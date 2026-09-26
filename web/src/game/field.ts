import type { Rules } from "./rules";
import { manhattan, samePosition, Snake, type Position } from "./snake";

export type ItemType = "normal_apple" | "gold_apple" | "poison_apple" | "block_clear" | "block_jam";

export interface Item {
  pos: Position;
  kind: ItemType;
}

export interface PendingSpawn {
  kind: ItemType;
  ticks: number;
}

/** 出現位置の選び方。通常はランダムに1つ選ぶ (テストでは決まった位置を返す) */
export type Picker = (candidates: Position[], what: ItemType | "obstacle") => Position | undefined;

export const randomPicker: Picker = (candidates) =>
  candidates.length > 0 ? candidates[Math.floor(Math.random() * candidates.length)] : undefined;

export function respawnTicks(rules: Rules, kind: ItemType): number {
  return kind === "normal_apple" || kind === "poison_apple" ? rules.appleRespawnTicks : rules.specialRespawnTicks;
}

/** プレイヤー1人分の盤面 */
export class Field {
  snake: Snake;
  items: Item[] = [];
  obstacles: Position[] = [];
  /** 食べられたアイテムの再出現待ち */
  pendingSpawns: PendingSpawn[] = [];
  /** 相手から送られたお邪魔ブロックの出現待ち (残りティック数) */
  incomingJams: number[] = [];

  /** `heldItems` が false なら、所持アイテム (ブロック消去・お邪魔) を置かずリンゴだけにする */
  constructor(rules: Rules, pick: Picker, heldItems = true) {
    this.snake = new Snake({ x: Math.floor(rules.width / 2), y: Math.floor(rules.height / 2) }, rules);
    const initial: ItemType[] = [
      ...Array<ItemType>(rules.normalApples).fill("normal_apple"),
      ...Array<ItemType>(rules.poisonApples).fill("poison_apple"),
      "gold_apple",
      ...(heldItems ? (["block_clear", "block_jam"] as const) : []),
    ];
    for (const kind of initial) {
      this.spawnItem(kind, rules, pick);
    }
  }

  /** 生きているヘビが、お邪魔ブロック以外のすべてのマスを埋めているか */
  isFilled(rules: Rules): boolean {
    return this.snake.alive && this.snake.length + this.obstacles.length === rules.width * rules.height;
  }

  static inBounds(pos: Position, rules: Rules): boolean {
    return pos.x >= 0 && pos.x < rules.width && pos.y >= 0 && pos.y < rules.height;
  }

  hasObstacle(pos: Position): boolean {
    return this.obstacles.some((p) => samePosition(p, pos));
  }

  /** ヘビ・アイテム・ブロックのどれも無いマスを列挙する */
  emptyCells(rules: Rules): Position[] {
    const w = rules.width;
    const occupied = new Uint8Array(w * rules.height);
    for (const p of this.snake.body) occupied[p.y * w + p.x] = 1;
    for (const item of this.items) occupied[item.pos.y * w + item.pos.x] = 1;
    for (const p of this.obstacles) occupied[p.y * w + p.x] = 1;

    const cells: Position[] = [];
    for (let i = 0; i < occupied.length; i++) {
      if (!occupied[i]) cells.push({ x: i % w, y: Math.floor(i / w) });
    }
    return cells;
  }

  /** 空きマスにアイテムを出す。空きが無ければ false */
  spawnItem(kind: ItemType, rules: Rules, pick: Picker): boolean {
    const pos = pick(this.emptyCells(rules), kind);
    if (!pos) return false;
    this.items.push({ pos, kind });
    return true;
  }

  /** 頭から安全距離より離れた空きマスにお邪魔ブロックを出す。置ける場所が無ければ false */
  spawnObstacle(rules: Rules, pick: Picker): boolean {
    const head = this.snake.head;
    const candidates = this.emptyCells(rules).filter((p) => manhattan(p, head) > rules.jamSafeDistance);
    const pos = pick(candidates, "obstacle");
    if (!pos) return false;
    this.obstacles.push(pos);
    return true;
  }

  /** 出現待ちのタイマーを進め、時間になったものを出現させる */
  processPending(rules: Rules, pick: Picker): void {
    const dueItems: ItemType[] = [];
    this.pendingSpawns = this.pendingSpawns.filter((p) => {
      p.ticks = Math.max(0, p.ticks - 1);
      if (p.ticks === 0) dueItems.push(p.kind);
      return p.ticks > 0;
    });
    for (const kind of dueItems) {
      if (!this.spawnItem(kind, rules, pick)) {
        // 盤面が埋まっていたら次のティックで再挑戦
        this.pendingSpawns.push({ kind, ticks: 1 });
      }
    }

    let dueJams = 0;
    this.incomingJams = this.incomingJams
      .map((t) => Math.max(0, t - 1))
      .filter((t) => {
        if (t === 0) dueJams += 1;
        return t > 0;
      });
    for (let i = 0; i < dueJams; i++) {
      // 置き場所が無いお邪魔ブロックは消滅する
      this.spawnObstacle(rules, pick);
    }
  }

  /** ヘビのタイマーを進め、移動するティックなら移動・衝突判定・アイテム取得まで行う */
  updateSnake(rules: Rules): void {
    const snake = this.snake;
    if (!snake.alive || !snake.advanceTimers(rules)) return;

    const next = snake.nextHead();
    if (!Field.inBounds(next, rules) || this.hasObstacle(next) || snake.collidesWithSelf(next)) {
      snake.alive = false;
      return;
    }
    snake.advance(next);

    const idx = this.items.findIndex((i) => samePosition(i.pos, next));
    if (idx >= 0) {
      const [item] = this.items.splice(idx, 1);
      this.consume(item.kind, rules);
    }
  }

  private consume(kind: ItemType, rules: Rules): void {
    const snake = this.snake;
    const apple =
      kind === "normal_apple"
        ? rules.normalApple
        : kind === "gold_apple"
          ? rules.goldApple
          : kind === "poison_apple"
            ? rules.poisonApple
            : null;
    if (apple) {
      snake.score = Math.max(0, snake.score + apple.score);
      snake.applyGrowth(apple.grow);
    } else {
      snake.heldItem = kind === "block_clear" ? "block_clear" : "block_jam";
    }
    this.pendingSpawns.push({ kind, ticks: respawnTicks(rules, kind) });
  }
}
