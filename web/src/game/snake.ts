import type { Rules } from "./rules";

/** 盤面上の座標。左上が (0, 0) で、x は右、y は下に向かって増える */
export interface Position {
  x: number;
  y: number;
}

export type Direction = "up" | "down" | "left" | "right";

/** 観測ベクトルの one-hot で使う並び */
export const DIRECTIONS: readonly Direction[] = ["up", "down", "left", "right"];

const DELTA: Record<Direction, Position> = {
  up: { x: 0, y: -1 },
  down: { x: 0, y: 1 },
  left: { x: -1, y: 0 },
  right: { x: 1, y: 0 },
};

const OPPOSITE: Record<Direction, Direction> = {
  up: "down",
  down: "up",
  left: "right",
  right: "left",
};

export function stepPosition(pos: Position, dir: Direction): Position {
  const d = DELTA[dir];
  return { x: pos.x + d.x, y: pos.y + d.y };
}

export function samePosition(a: Position, b: Position): boolean {
  return a.x === b.x && a.y === b.y;
}

export function manhattan(a: Position, b: Position): number {
  return Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
}

export type HeldItem = "block_clear" | "block_jam";

export class Snake {
  /** 先頭が頭 */
  body: Position[];
  /** 次の移動で進む方向 */
  dir: Direction = "up";
  /** 直近の移動で実際に進んだ方向。逆走の判定はこちらで行う */
  lastMovedDir: Direction = "up";
  alive = true;
  score = 0;
  heldItem: HeldItem | null = null;
  /** これから伸びる残りマス数。0 より大きい間は移動しても尻尾が縮まない */
  pendingGrowth = 0;
  /** 次の移動までの残りティック数。1 なら次のティックで移動する */
  moveCooldown: number;
  boostRemaining = 0;
  boostCooldown = 0;

  /** 頭を `head` に置き、胴体を下向きに伸ばした状態で上向きに生成する */
  constructor(head: Position, rules: Rules) {
    this.body = Array.from({ length: rules.initialLength }, (_, i) => ({ x: head.x, y: head.y + i }));
    this.moveCooldown = rules.normalIntervalTicks;
  }

  get head(): Position {
    return this.body[0];
  }

  get length(): number {
    return this.body.length;
  }

  /** 直前の進行方向と真逆の入力は無視する */
  turn(dir: Direction): void {
    if (dir !== OPPOSITE[this.lastMovedDir]) {
      this.dir = dir;
    }
  }

  get boostReady(): boolean {
    return this.boostRemaining === 0 && this.boostCooldown === 0;
  }

  tryBoost(rules: Rules): void {
    if (this.boostReady) {
      this.boostRemaining = rules.boostDurationTicks;
    }
  }

  /** 次のティックで移動するか (= AI が行動を決めるタイミングか) */
  get movesNextTick(): boolean {
    return this.alive && this.moveCooldown === 1;
  }

  /** ブーストと移動のタイマーを1ティック進め、このティックで移動するなら true を返す */
  advanceTimers(rules: Rules): boolean {
    if (this.boostRemaining > 0) {
      this.boostRemaining -= 1;
      if (this.boostRemaining === 0) {
        this.boostCooldown = rules.boostCooldownTicks;
      }
    } else if (this.boostCooldown > 0) {
      this.boostCooldown -= 1;
    }

    this.moveCooldown -= 1;
    if (this.moveCooldown > 0) {
      return false;
    }
    this.moveCooldown = this.boostRemaining > 0 ? rules.boostIntervalTicks : rules.normalIntervalTicks;
    return true;
  }

  nextHead(): Position {
    return stepPosition(this.head, this.dir);
  }

  /** `pos` に進むと自分の体にぶつかるか。尻尾はこの移動で退くので伸びている途中でなければ除外する */
  collidesWithSelf(pos: Position): boolean {
    const checked = this.pendingGrowth === 0 ? this.body.length - 1 : this.body.length;
    for (let i = 0; i < checked; i++) {
      if (samePosition(this.body[i], pos)) return true;
    }
    return false;
  }

  advance(next: Position): void {
    this.body.unshift(next);
    if (this.pendingGrowth > 0) {
      this.pendingGrowth -= 1;
    } else {
      this.body.pop();
    }
    this.lastMovedDir = this.dir;
  }

  /** 正なら以降の移動で伸び、負なら伸びる予定を打ち消したうえで尻尾を即座に削る (最短1マス) */
  applyGrowth(grow: number): void {
    if (grow >= 0) {
      this.pendingGrowth += grow;
      return;
    }
    let shrink = -grow;
    const cancelled = Math.min(shrink, this.pendingGrowth);
    this.pendingGrowth -= cancelled;
    shrink -= cancelled;
    for (let i = 0; i < shrink && this.body.length > 1; i++) {
      this.body.pop();
    }
  }
}
