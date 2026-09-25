import { Field, randomPicker, type Picker } from "./field";
import type { Rules } from "./rules";

export const NUM_PLAYERS = 2;

/** プレイヤーの行動。並び順がモデル出力 (Q 値) の並びになる */
export type Action = "up" | "down" | "left" | "right" | "boost" | "use_item";

export const ACTIONS: readonly Action[] = ["up", "down", "left", "right", "boost", "use_item"];

export type EndReason = "death" | "time_up";

export interface GameResult {
  /** null は引き分け */
  winner: number | null;
  reason: EndReason;
}

export type PlayerInputs = readonly [readonly Action[], readonly Action[]];

/** 2人対戦1試合分の環境 (learn/src/env/game.rs と同じルール) */
export class GameEnv {
  readonly rules: Rules;
  fields: [Field, Field];
  tick = 0;
  result: GameResult | null = null;
  /** アイテムとお邪魔ブロックの出現位置の選び方 */
  pick: Picker;

  constructor(rules: Rules, pick: Picker = randomPicker) {
    this.rules = rules;
    this.pick = pick;
    this.fields = [new Field(rules, pick), new Field(rules, pick)];
  }

  get isOver(): boolean {
    return this.result !== null;
  }

  get remainingTicks(): number {
    return Math.max(0, this.rules.timeLimitTicks - this.tick);
  }

  /** このプレイヤーが次のティックの前に行動を決める必要があるか */
  needsDecision(player: number): boolean {
    return !this.isOver && this.fields[player].snake.movesNextTick;
  }

  /**
   * 1ティック進める。`inputs[p]` はこのティックに反映するプレイヤー p の入力で、押した順に並べる。
   * AI は行動を決めるティックに1つだけ、人間は前のティック以降に押したものをすべて渡す
   */
  step(inputs: PlayerInputs): void {
    if (this.isOver) return;
    this.tick += 1;

    for (const field of this.fields) {
      field.processPending(this.rules, this.pick);
    }
    inputs.forEach((actions, player) => {
      for (const action of actions) this.applyAction(player, action);
    });
    for (const field of this.fields) {
      field.updateSnake(this.rules);
    }

    this.result = this.judge();
  }

  private applyAction(player: number, action: Action): void {
    const field = this.fields[player];
    const snake = field.snake;
    switch (action) {
      case "up":
      case "down":
      case "left":
      case "right":
        snake.turn(action);
        break;
      case "boost":
        snake.tryBoost(this.rules);
        break;
      case "use_item": {
        const held = snake.heldItem;
        snake.heldItem = null;
        if (held === "block_clear") {
          field.obstacles = [];
        } else if (held === "block_jam") {
          this.fields[1 - player].incomingJams.push(this.rules.jamDelayTicks);
        }
        break;
      }
    }
  }

  private judge(): GameResult | null {
    const [a, b] = this.fields.map((f) => f.snake.alive);
    if (a && !b) return { winner: 0, reason: "death" };
    if (!a && b) return { winner: 1, reason: "death" };
    if (!a && !b) return { winner: this.leaderByScore(), reason: "death" };
    if (this.tick >= this.rules.timeLimitTicks) return { winner: this.leaderByScore(), reason: "time_up" };
    return null;
  }

  private leaderByScore(): number | null {
    const [a, b] = this.fields.map((f) => f.snake.score);
    if (a > b) return 0;
    if (a < b) return 1;
    return null;
  }
}
