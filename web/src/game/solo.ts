import { Field, randomPicker, type Picker } from "./field";
import type { Action } from "./game";
import type { Rules } from "./rules";

/** 一人モードで毒リンゴを何倍に増やすか */
export const SOLO_POISON_MULTIPLIER = 3;
/** 一人モードで毒リンゴを取ったときの長さの増減 */
export const SOLO_POISON_GROW = -4;

/** 一人モードのルール。対戦のルールから、毒リンゴの数を 3 倍にし、取ると長さ -4 にする */
export function soloRules(rules: Rules): Rules {
  return {
    ...rules,
    poisonApples: rules.poisonApples * SOLO_POISON_MULTIPLIER,
    poisonApple: { ...rules.poisonApple, grow: SOLO_POISON_GROW },
  };
}

/** filled はすべてのマスを埋めたこと (クリア) */
export type SoloEndReason = "filled" | "death" | "time_up";

/**
 * 一人モードの 1 回分の環境。相手がいないので、所持アイテム (ブロック消去・お邪魔) は出さずリンゴだけにする。
 * それ以外は対戦 (game.ts) と同じルールで進む
 */
export class SoloEnv {
  readonly rules: Rules;
  field: Field;
  tick = 0;
  result: SoloEndReason | null = null;
  /** リンゴの出現位置の選び方 */
  pick: Picker;

  /** `rules` は対戦のルール。一人モードのルール (soloRules) に直して使う */
  constructor(rules: Rules, pick: Picker = randomPicker) {
    this.rules = soloRules(rules);
    this.pick = pick;
    this.field = new Field(this.rules, pick, false);
  }

  get isOver(): boolean {
    return this.result !== null;
  }

  get remainingTicks(): number {
    return Math.max(0, this.rules.timeLimitTicks - this.tick);
  }

  /** 1ティック進める。`actions` はこのティックに反映する入力で、押した順に並べる */
  step(actions: readonly Action[]): void {
    if (this.isOver) return;
    this.tick += 1;

    this.field.processPending(this.rules, this.pick);
    const snake = this.field.snake;
    for (const action of actions) {
      if (action === "boost") snake.tryBoost(this.rules);
      // 所持アイテムが出ないので use_item では何も起きない
      else if (action !== "use_item") snake.turn(action);
    }
    this.field.updateSnake(this.rules);

    this.result = this.judge();
  }

  private judge(): SoloEndReason | null {
    if (this.field.isFilled(this.rules)) return "filled";
    if (!this.field.snake.alive) return "death";
    if (this.tick >= this.rules.timeLimitTicks) return "time_up";
    return null;
  }
}
