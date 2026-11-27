import type { Action, GameEnv } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import type { SnakeModel } from "./model";
import { checkObservationSpec, encodeObservation, fullObservationLengths } from "./observation";

export interface Decision {
  action: Action;
  /** 行動ごとの Q 値 (並びは model.info.actions) */
  qValues: Float32Array;
  /** 難易度によるランダム行動だったか */
  random: boolean;
}

/** モデルがこの Web 版のルール・観測仕様で使えるか確かめる。使えなければ例外 */
export function assertModelCompatible(model: SnakeModel): void {
  checkObservationSpec(model.info.observation, rulesFromConfig(model.info.game));
}

/** 学習済みモデルで動く AI プレイヤー */
export class AiPlayer {
  private readonly model: SnakeModel;
  private readonly randomActionRate: number;
  /** 観測全体を書く配列 */
  private readonly grid: Float32Array;
  private readonly vector: Float32Array;
  /** grid・vector のうちモデルに渡す先頭部分 (古いモデルは相手の盤面・状態を使わない) */
  private readonly gridInput: Float32Array;
  private readonly vectorInput: Float32Array;

  constructor(model: SnakeModel, randomActionRate: number) {
    this.model = model;
    this.randomActionRate = randomActionRate;
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

    if (Math.random() < this.randomActionRate) {
      return { action: actions[Math.floor(Math.random() * actions.length)], qValues, random: true };
    }
    let best = 0;
    for (let i = 1; i < qValues.length; i++) {
      if (qValues[i] > qValues[best]) best = i;
    }
    return { action: actions[best], qValues, random: false };
  }
}
