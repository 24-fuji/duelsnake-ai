import { Field, randomPicker, type ItemType, type Picker } from "./field";
import type { Action } from "./game";
import type { Rules } from "./rules";

/** 一人モードで毒リンゴを何倍に増やすか */
export const SOLO_POISON_MULTIPLIER = 3;
/** 一人モードで毒リンゴを取ったときの長さの増減 */
export const SOLO_POISON_GROW = -4;
/** 一人モードで選べる制限時間 (秒)。0 は制限なしで、そこから 30 秒刻みで 5 分まで */
export const SOLO_TIME_LIMITS: readonly number[] = Array.from({ length: 11 }, (_, i) => i * 30);
/** 一人モードの制限時間の既定値 (秒)。対戦と同じにする */
export const SOLO_DEFAULT_TIME_LIMIT = 30;

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
 * 制限時間は選べる (制限なしも選べる)。それ以外は対戦 (game.ts) と同じルールで進む
 */
export class SoloEnv {
  readonly rules: Rules;
  /** 制限時間のティック数。null は制限なし。rules.timeLimitTicks (対戦の制限時間) は使わない */
  readonly timeLimitTicks: number | null;
  field: Field;
  tick = 0;
  result: SoloEndReason | null = null;
  /** これまでに取ったアイテムの種類ごとの数 */
  readonly eatenCounts: Record<ItemType, number> = {
    normal_apple: 0,
    gold_apple: 0,
    poison_apple: 0,
    block_clear: 0,
    block_jam: 0,
  };
  /** リンゴの出現位置の選び方 */
  pick: Picker;

  /**
   * `rules` は対戦のルール。一人モードのルール (soloRules) に直して使う。
   * `timeLimitSeconds` は制限時間 (秒) で、0 なら制限なし
   */
  constructor(rules: Rules, timeLimitSeconds = SOLO_DEFAULT_TIME_LIMIT, pick: Picker = randomPicker) {
    this.rules = soloRules(rules);
    this.timeLimitTicks = timeLimitSeconds > 0 ? Math.max(1, Math.round(timeLimitSeconds / rules.tickSeconds)) : null;
    this.pick = pick;
    this.field = new Field(this.rules, pick, false);
  }

  get isOver(): boolean {
    return this.result !== null;
  }

  /** 残りのティック数。制限なしなら null */
  get remainingTicks(): number | null {
    return this.timeLimitTicks === null ? null : Math.max(0, this.timeLimitTicks - this.tick);
  }

  /** 始めてからの経過秒数 */
  get elapsedSeconds(): number {
    return this.tick * this.rules.tickSeconds;
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
    if (this.field.eaten) this.eatenCounts[this.field.eaten] += 1;

    this.result = this.judge();
  }

  private judge(): SoloEndReason | null {
    if (this.field.isFilled(this.rules)) return "filled";
    if (!this.field.snake.alive) return "death";
    if (this.timeLimitTicks !== null && this.tick >= this.timeLimitTicks) return "time_up";
    return null;
  }
}
