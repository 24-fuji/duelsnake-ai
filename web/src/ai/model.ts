/**
 * learn/ が書き出すモデル JSON (model/recent-model/snake-model-<幅>x<高さ>.json) を読み込んで推論する。
 * 観測 (入力) の作り方と順伝播の定義は learn/RULES.md の 10〜11 章を参照。
 */

import type { Action } from "../game/game";
import type { GameRules } from "../game/rules";

export const MODEL_FORMAT = "duelsnake-dqn";
export const MODEL_FORMAT_VERSION = 1;

export interface ObservationSpec {
  grid_channels: string[];
  grid_height: number;
  grid_width: number;
  vector_features: string[];
  pending_growth_scale: number;
  score_diff_scale: number;
  incoming_jams_scale: number;
}

export interface ConvLayer {
  in_channels: number;
  out_channels: number;
  kernel_size: number;
  stride: number;
  padding: number;
  /** [out_channels][in_channels][kernel_size][kernel_size] を平坦化したもの */
  weight: number[];
  bias: number[];
}

export interface DenseLayer {
  in_features: number;
  out_features: number;
  /** [out_features][in_features] を平坦化したもの */
  weight: number[];
  bias: number[];
}

export interface Difficulty {
  name: string;
  random_action_rate: number;
}

export interface ModelFile {
  format: string;
  format_version: number;
  created_at: string;
  training: { games: number; decisions: number; updates: number };
  game: GameRules;
  observation: ObservationSpec;
  actions: Action[];
  difficulty: Difficulty[];
  network: { conv: ConvLayer[]; dense: DenseLayer[] };
}

export type ModelInfo = Omit<ModelFile, "network">;

interface Conv {
  inChannels: number;
  outChannels: number;
  kernel: number;
  stride: number;
  padding: number;
  weight: Float32Array;
  bias: Float32Array;
}

interface Dense {
  inFeatures: number;
  outFeatures: number;
  weight: Float32Array;
  bias: Float32Array;
}

export class SnakeModel {
  /** 重み以外の情報 (ルール・観測仕様・行動・難易度など) */
  readonly info: ModelInfo;
  private readonly conv: Conv[];
  private readonly dense: Dense[];

  static async load(url: string): Promise<SnakeModel> {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`モデルを読み込めません: ${url} (${response.status})`);
    }
    return new SnakeModel((await response.json()) as ModelFile);
  }

  constructor(file: ModelFile) {
    if (file.format !== MODEL_FORMAT || file.format_version !== MODEL_FORMAT_VERSION) {
      throw new Error(`未対応のモデル形式です: ${file.format} v${file.format_version}`);
    }
    const { network, ...info } = file;
    this.info = info;

    this.conv = network.conv.map((layer, i) => {
      const k = layer.kernel_size;
      expectLength(`conv[${i}].weight`, layer.weight, layer.out_channels * layer.in_channels * k * k);
      expectLength(`conv[${i}].bias`, layer.bias, layer.out_channels);
      return {
        inChannels: layer.in_channels,
        outChannels: layer.out_channels,
        kernel: k,
        stride: layer.stride,
        padding: layer.padding,
        weight: Float32Array.from(layer.weight),
        bias: Float32Array.from(layer.bias),
      };
    });
    this.dense = network.dense.map((layer, i) => {
      expectLength(`dense[${i}].weight`, layer.weight, layer.out_features * layer.in_features);
      expectLength(`dense[${i}].bias`, layer.bias, layer.out_features);
      return {
        inFeatures: layer.in_features,
        outFeatures: layer.out_features,
        weight: Float32Array.from(layer.weight),
        bias: Float32Array.from(layer.bias),
      };
    });
  }

  /** 盤面入力の長さ (チャネル数 × 高さ × 幅) */
  get gridLength(): number {
    const obs = this.info.observation;
    return obs.grid_channels.length * obs.grid_height * obs.grid_width;
  }

  /** 状態ベクトルの長さ */
  get vectorLength(): number {
    return this.info.observation.vector_features.length;
  }

  /** 行動ごとの Q 値を返す。並びは info.actions と同じ */
  predict(grid: Float32Array, vector: Float32Array): Float32Array {
    if (grid.length !== this.gridLength || vector.length !== this.vectorLength) {
      throw new Error(
        `入力の長さが違います: grid ${grid.length} (期待 ${this.gridLength}), vector ${vector.length} (期待 ${this.vectorLength})`,
      );
    }
    let x = grid;
    let h = this.info.observation.grid_height;
    let w = this.info.observation.grid_width;
    for (const layer of this.conv) {
      [x, h, w] = conv2dRelu(layer, x, h, w);
    }

    const flat = new Float32Array(x.length + vector.length);
    flat.set(x);
    flat.set(vector, x.length);
    x = flat;

    this.dense.forEach((layer, i) => {
      x = linear(layer, x, i < this.dense.length - 1);
    });
    return x;
  }

  /** Q 値が最大の行動を選ぶ。randomActionRate の確率でランダムな行動にする */
  selectAction(grid: Float32Array, vector: Float32Array, randomActionRate = 0): Action {
    const actions = this.info.actions;
    if (Math.random() < randomActionRate) {
      return actions[Math.floor(Math.random() * actions.length)];
    }
    const q = this.predict(grid, vector);
    let best = 0;
    for (let i = 1; i < q.length; i++) {
      if (q[i] > q[best]) best = i;
    }
    return actions[best];
  }

  /** 難易度名からランダム行動率を引く。見つからなければ 0 (最も強い) */
  randomActionRate(difficulty: string): number {
    return this.info.difficulty.find((d) => d.name === difficulty)?.random_action_rate ?? 0;
  }
}

function expectLength(name: string, values: number[], expected: number): void {
  if (values.length !== expected) {
    throw new Error(`${name} の要素数が違います: ${values.length} (期待 ${expected})`);
  }
}

function conv2dRelu(
  layer: Conv,
  input: Float32Array,
  h: number,
  w: number,
): [Float32Array, number, number] {
  const { inChannels, outChannels, kernel: k, stride: s, padding: p, weight, bias } = layer;
  const oh = Math.floor((h + 2 * p - k) / s) + 1;
  const ow = Math.floor((w + 2 * p - k) / s) + 1;
  const plane = oh * ow;
  const out = new Float32Array(outChannels * plane);

  for (let oc = 0; oc < outChannels; oc++) {
    const outBase = oc * plane;
    out.fill(bias[oc], outBase, outBase + plane);
    for (let ic = 0; ic < inChannels; ic++) {
      const inBase = ic * h * w;
      for (let ky = 0; ky < k; ky++) {
        // 入力の行 iy = oy * s + ky - p が盤内に収まる oy の範囲
        const oyStart = Math.max(0, Math.ceil((p - ky) / s));
        const oyEnd = Math.min(oh - 1, Math.floor((h - 1 + p - ky) / s));
        for (let kx = 0; kx < k; kx++) {
          const wv = weight[((oc * inChannels + ic) * k + ky) * k + kx];
          const oxStart = Math.max(0, Math.ceil((p - kx) / s));
          const oxEnd = Math.min(ow - 1, Math.floor((w - 1 + p - kx) / s));
          for (let oy = oyStart; oy <= oyEnd; oy++) {
            const inRow = inBase + (oy * s + ky - p) * w + kx - p;
            const outRow = outBase + oy * ow;
            for (let ox = oxStart; ox <= oxEnd; ox++) {
              out[outRow + ox] += wv * input[inRow + ox * s];
            }
          }
        }
      }
    }
  }
  for (let i = 0; i < out.length; i++) {
    if (out[i] < 0) out[i] = 0;
  }
  return [out, oh, ow];
}

function linear(layer: Dense, input: Float32Array, relu: boolean): Float32Array {
  const { inFeatures, outFeatures, weight, bias } = layer;
  const out = new Float32Array(outFeatures);
  for (let o = 0; o < outFeatures; o++) {
    let sum = bias[o];
    const row = o * inFeatures;
    for (let i = 0; i < inFeatures; i++) {
      sum += weight[row + i] * input[i];
    }
    out[o] = relu && sum < 0 ? 0 : sum;
  }
  return out;
}
