import type { Action, GameEnv } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import type { SnakeModel } from "./model";
import { checkObservationSpec, encodeObservation } from "./observation";

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
  private readonly grid: Float32Array;
  private readonly vector: Float32Array;

  constructor(model: SnakeModel, randomActionRate: number) {
    this.model = model;
    this.randomActionRate = randomActionRate;
    this.grid = new Float32Array(model.gridLength);
    this.vector = new Float32Array(model.vectorLength);
  }

  /** 行動を決める。env.needsDecision(player) が true のティックで呼ぶ */
  decide(env: GameEnv, player: number): Decision {
    encodeObservation(env, player, this.model.info.observation, this.grid, this.vector);
    const qValues = this.model.predict(this.grid, this.vector);
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
