import type { Action, GameEnv } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import type { SnakeModel } from "./model";
import { checkObservationSpec, encodeObservation, fullObservationLengths } from "./observation";
import { argmax, chooseSafely } from "./safety";

export interface Decision {
  action: Action;
  /** 行動ごとの Q 値 (並びは model.info.actions) */
  qValues: Float32Array;
  /** 難易度によるランダム行動だったか */
  random: boolean;
  /** 安全装置が、Q 値が最大の行動 (`greedyAction`) を差し替えたか */
  overridden: boolean;
  /** Q 値が最大の行動 */
  greedyAction: Action;
}

/** モデルがこの Web 版のルール・観測仕様で使えるか確かめる。使えなければ例外 */
export function assertModelCompatible(model: SnakeModel): void {
  checkObservationSpec(model.info.observation, rulesFromConfig(model.info.game));
}

/** 学習済みモデルで動く AI プレイヤー */
export class AiPlayer {
  private readonly model: SnakeModel;
  private readonly randomActionRate: number;
  /** 推論時の安全装置 (safety.ts) を通して行動を選ぶか */
  private readonly safety: boolean;
  /** 観測全体を書く配列 */
  private readonly grid: Float32Array;
  private readonly vector: Float32Array;
  /** grid・vector のうちモデルに渡す先頭部分 (古いモデルは相手の盤面・状態を使わない) */
  private readonly gridInput: Float32Array;
  private readonly vectorInput: Float32Array;

  constructor(model: SnakeModel, randomActionRate: number, safety = true) {
    this.model = model;
    this.randomActionRate = randomActionRate;
    this.safety = safety;
    const full = fullObservationLengths(rulesFromConfig(model.info.game));
    this.grid = new Float32Array(full.grid);
    this.vector = new Float32Array(full.vector);
    this.gridInput = this.grid.subarray(0, model.gridLength);
    this.vectorInput = this.vector.subarray(0, model.vectorLength);
  }

  /** 行動を決める。env.needsDecision(player) が true のティックで呼ぶ */
  decide(env: GameEnv, player: number): Decision {
    encodeObservation(env, player, this.model.info.observation, this.grid, this.vector);
    const qValues = this.model.predict(this.gridInput, this.vectorInput);
    const actions = this.model.info.actions;

    const greedyAction = actions[argmax(qValues)];
    if (Math.random() < this.randomActionRate) {
      const action = actions[Math.floor(Math.random() * actions.length)];
      return { action, qValues, random: true, overridden: false, greedyAction };
    }
    if (!this.safety) {
      return { action: greedyAction, qValues, random: false, overridden: false, greedyAction };
    }
    const choice = chooseSafely(env.fields[player], env.rules, actions, qValues);
    return { action: actions[choice.index], qValues, random: false, overridden: choice.overridden, greedyAction };
  }
}
