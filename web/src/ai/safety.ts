/**
 * 推論時の安全装置 (docs/training/endgame.md の提案 1)。learn/src/safety.rs と learn/src/trap.rs と同じ手順。
 *
 * 行動を選ぶ前に、行動ごとに次の移動の行き先を調べて 4 段階に分け、いちばん良い段階の中から Q 値が最大の行動を選ぶ。
 * 1. 助かる: 今の体がすべて退くまで動き続けられる経路がある
 * 2. 判定できず: 探索が上限 (SAFETY_BUDGET) を越えた
 * 3. 詰み: 動き続けられる経路が無い。この中では、行けるマスが多い手を選ぶ (相手が先に死ぬまで粘る)
 * 4. 衝突: 次の一手で衝突する
 *
 * ブースト・アイテム使用は向きを変えないので、今の向きに進む手として調べる。
 * ブロック消去を持っているときのアイテム使用は、お邪魔ブロックを消してから進む手として調べる。
 * 経路の途中でアイテムを取ったときの効果と、これから置かれるお邪魔ブロックは考えない。
 */

import { Field } from "../game/field";
import type { Action } from "../game/game";
import type { Rules } from "../game/rules";
import { DIRECTIONS, stepPosition, type Direction, type Position } from "../game/snake";

/** 1 つの手を調べる局面の数の上限 (learn の trap::SAFETY_BUDGET と同じ) */
export const SAFETY_BUDGET = 2000;

/** 行き先の段階。小さいほど良い */
export const Class = { Open: 0, Unknown: 1, Trapped: 2, Collide: 3 } as const;
export type Class = (typeof Class)[keyof typeof Class];

export interface Verdict {
  class: Class;
  /** 進んだ先から、お邪魔ブロックと (次の移動で退く尻尾を除く) 体を避けて行けるマスの数。詰みの手どうしを比べるのに使う */
  room: number;
}

export interface Choice {
  /** 選んだ行動の番号 (Q 値の並び) */
  index: number;
  /** Q 値が最大の行動を差し替えたか */
  overridden: boolean;
}

const OPPOSITE: Record<Direction, Direction> = { up: "down", down: "up", left: "right", right: "left" };

/** 安全装置の探索に使う、フィールドの様子 */
interface Snapshot {
  /** 先頭が頭 */
  body: Position[];
  pendingGrowth: number;
  lastMovedDir: Direction;
  obstacles: Position[];
}

/** この行動で次に進む向きと、お邪魔ブロックを消してから進むか */
export function moveOf(field: Field, action: Action): { dir: Direction; cleared: boolean } {
  const snake = field.snake;
  switch (action) {
    case "up":
    case "down":
    case "left":
    case "right":
      // 直前に進んだ方向の真逆は無視され、今の向きのまま進む
      return { dir: action === OPPOSITE[snake.lastMovedDir] ? snake.dir : action, cleared: false };
    case "boost":
      return { dir: snake.dir, cleared: false };
    case "use_item":
      return { dir: snake.dir, cleared: snake.heldItem === "block_clear" && field.obstacles.length > 0 };
  }
}

/** `dir` に進んだときの行き先を調べる。`cleared` ならお邪魔ブロックを消してから進む */
export function judge(field: Field, rules: Rules, dir: Direction, cleared: boolean): Verdict {
  const snake = field.snake;
  const next = stepPosition(snake.head, dir);
  if (!Field.inBounds(next, rules) || (!cleared && field.hasObstacle(next)) || snake.collidesWithSelf(next)) {
    return { class: Class.Collide, room: 0 };
  }
  const snapshot: Snapshot = {
    body: snake.body,
    pendingGrowth: snake.pendingGrowth,
    lastMovedDir: snake.lastMovedDir,
    obstacles: cleared ? [] : field.obstacles,
  };
  return { class: escapeBy(snapshot, rules.width, rules.height, dir, SAFETY_BUDGET), room: room(snapshot, rules, next) };
}

/** Q 値 `q` (並びは `actions`) から、安全装置を通して行動を選ぶ */
export function chooseSafely(field: Field, rules: Rules, actions: readonly Action[], q: ArrayLike<number>): Choice {
  const greedy = argmax(q);
  // 同じ行き先は 1 回だけ調べる (向き 4 つ × お邪魔ブロックを消すかどうか)
  const cache = new Map<string, Verdict>();
  const verdicts = actions.map((action) => {
    const { dir, cleared } = moveOf(field, action);
    const key = `${dir}:${cleared}`;
    let v = cache.get(key);
    if (!v) {
      v = judge(field, rules, dir, cleared);
      cache.set(key, v);
    }
    return v;
  });
  let best = greedy;
  for (let a = 0; a < actions.length; a++) {
    if (better(verdicts[a], q[a], verdicts[best], q[best])) best = a;
  }
  return { index: best, overridden: best !== greedy };
}

function better(va: Verdict, qa: number, vb: Verdict, qb: number): boolean {
  if (va.class !== vb.class) return va.class < vb.class;
  if (va.class === Class.Trapped && va.room !== vb.room) return va.room > vb.room;
  return qa > qb;
}

/** Q 値が最大の行動の番号。同じ値なら先のもの */
export function argmax(q: ArrayLike<number>): number {
  let best = 0;
  for (let i = 1; i < q.length; i++) {
    if (q[i] > q[best]) best = i;
  }
  return best;
}

/** `start` から、お邪魔ブロックと体 (次の移動で退く尻尾を除く) を避けて行けるマスの数 (`start` を含む) */
function room(snapshot: Snapshot, rules: Rules, start: Position): number {
  const { width: w, height: h } = rules;
  const blocked = new Uint8Array(w * h);
  for (const p of snapshot.obstacles) blocked[p.y * w + p.x] = 1;
  const kept = snapshot.body.length - (snapshot.pendingGrowth === 0 ? 1 : 0);
  for (let i = 0; i < kept; i++) blocked[snapshot.body[i].y * w + snapshot.body[i].x] = 1;
  blocked[start.y * w + start.x] = 1;
  const stack = [start];
  let count = 1;
  while (stack.length > 0) {
    const p = stack.pop()!;
    for (const dir of DIRECTIONS) {
      const n = stepPosition(p, dir);
      if (n.x < 0 || n.x >= w || n.y < 0 || n.y >= h || blocked[n.y * w + n.x]) continue;
      blocked[n.y * w + n.x] = 1;
      count += 1;
      stack.push(n);
    }
  }
  return count;
}

/**
 * 最初の一手を `first` に進んだうえで、今の体がすべて退くまで (長さ + 伸びる予定の手数だけ) 動き続けられる経路があるか。
 * 体は尻尾から順に空いていくので、その順番も考えて探す。伸びている途中で盤面が埋まる経路も助かる道に数える
 */
export function escapeBy(snapshot: Snapshot, width: number, height: number, first: Direction, budget: number): Class {
  const search = new Search(snapshot, width, height, budget);
  if (first === OPPOSITE[snapshot.lastMovedDir]) return Class.Trapped;
  const next = search.neighbor(search.body[0], first);
  if (next < 0 || !search.canEnter(next, 1)) return Class.Trapped;
  if (search.tracesBody(next)) return Class.Open;
  search.onPath[next] = 1;
  const found = search.run(next, first, 1);
  return found === null ? Class.Unknown : found ? Class.Open : Class.Trapped;
}

/** 体でないマスの bodyIndex */
const NOT_BODY = -1;

/** 経路の探索。マスは y * 幅 + x の番号で扱う */
class Search {
  private readonly width: number;
  private readonly height: number;
  private readonly goal: number;
  /** freeAt[i] は、マス i に入れるようになる移動の番号 (1 手目が 1)。0 は最初から空き */
  private readonly freeAt: Float64Array;
  /** 今の体のマスの番号 (先頭が頭) */
  readonly body: Int32Array;
  /** bodyIndex[i] は、マス i が頭から何番目の体か。体でなければ NOT_BODY */
  private readonly bodyIndex: Int32Array;
  /** 探索中の経路で通ったマス。経路は新しい体になり、goal 手までは退かない */
  readonly onPath: Uint8Array;
  /** 行けるマスを数えるときに、数えたマスへ付ける印。数えるたびに stamp を 1 増やすので、消さずに使い回せる */
  private readonly seen: Uint32Array;
  private stamp = 0;
  private readonly stack: Int32Array;
  private readonly budget: number;
  private nodes = 0;

  constructor(snapshot: Snapshot, width: number, height: number, budget: number) {
    const cells = width * height;
    const len = snapshot.body.length;
    const growth = snapshot.pendingGrowth;
    this.width = width;
    this.height = height;
    this.budget = budget;
    this.freeAt = new Float64Array(cells);
    for (const p of snapshot.obstacles) this.freeAt[p.y * width + p.x] = Infinity;
    // 頭から i 番目の体は、伸びる予定の分だけ尻尾が止まったあと、尻尾から順に退く
    this.body = Int32Array.from(snapshot.body, (p) => p.y * width + p.x);
    this.bodyIndex = new Int32Array(cells).fill(NOT_BODY);
    this.body.forEach((c, i) => {
      this.freeAt[c] = growth + len - i;
      this.bodyIndex[c] = i;
    });
    let goal = growth + len;
    // 伸びている途中で盤面が埋まれば勝ちなので、そこまで動ければよい
    const fillAt = Math.max(0, cells - snapshot.obstacles.length - len);
    if (fillAt <= growth) goal = Math.min(goal, fillAt);
    this.goal = goal;
    this.onPath = new Uint8Array(cells);
    this.seen = new Uint32Array(cells);
    this.stack = new Int32Array(cells + 1);
  }

  /** マス `i` の `dir` 側の隣。盤の外なら -1 */
  neighbor(i: number, dir: Direction): number {
    const x = i % this.width;
    const y = (i - x) / this.width;
    switch (dir) {
      case "up":
        return y > 0 ? i - this.width : -1;
      case "down":
        return y < this.height - 1 ? i + this.width : -1;
      case "left":
        return x > 0 ? i - 1 : -1;
      case "right":
        return x < this.width - 1 ? i + 1 : -1;
    }
  }

  /** `moves` 手目の移動でマス `i` に入れるか */
  canEnter(i: number, moves: number): boolean {
    return !this.onPath[i] && this.freeAt[i] <= moves;
  }

  /**
   * 入ったマス `i` が今の体のマスなら、そこから体を頭までなぞれば goal 手まで動き続けられるか。
   * 頭から b 番目の体のマスに、そのマスが空いた手以降に入れたなら、b - 1 番目、b - 2 番目 … もちょうど空いた手に入れる。
   * なぞるマスがまだ経路で通っていなければ、頭のマスに着く手は goal 以上になる
   */
  tracesBody(i: number): boolean {
    const b = this.bodyIndex[i];
    if (b === NOT_BODY) return false;
    for (let j = 0; j < b; j++) {
      if (this.onPath[this.body[j]]) return false;
    }
    return true;
  }

  /** `moves` 手動いて頭がマス `head` にあるところから、goal 手まで動き続けられるか。上限を越えたら null */
  run(head: number, lastDir: Direction, moves: number): boolean | null {
    if (moves >= this.goal) return true;
    this.nodes += 1;
    if (this.nodes > this.budget) return null;
    // 経路は同じマスを通れないので、行けるマスが残りの手数より少なければ届かない
    if (!this.hasRoom(head, this.goal - moves)) return false;
    for (const dir of DIRECTIONS) {
      // 直前に進んだ方向の真逆は無視される
      if (dir === OPPOSITE[lastDir]) continue;
      const next = this.neighbor(head, dir);
      if (next < 0 || !this.canEnter(next, moves + 1)) continue;
      if (this.tracesBody(next)) return true;
      this.onPath[next] = 1;
      const found = this.run(next, dir, moves + 1);
      this.onPath[next] = 0;
      if (found !== false) return found;
    }
    return false;
  }

  /**
   * `head` から、経路とお邪魔ブロックを避けて行けるマス (goal 手までに空かない体のマスも除く) が `need` 以上あるか。
   * `need` に届いた時点で数えるのをやめる
   */
  private hasRoom(head: number, need: number): boolean {
    if (need <= 0) return true;
    this.stamp += 1;
    let top = 0;
    this.stack[top++] = head;
    let count = 0;
    while (top > 0) {
      const i = this.stack[--top];
      for (const dir of DIRECTIONS) {
        const next = this.neighbor(i, dir);
        if (next < 0 || this.seen[next] === this.stamp || this.onPath[next] || this.freeAt[next] > this.goal) continue;
        this.seen[next] = this.stamp;
        count += 1;
        if (count >= need) return true;
        this.stack[top++] = next;
      }
    }
    return false;
  }
}
