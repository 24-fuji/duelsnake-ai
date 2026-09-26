import { describe, expect, it } from "vitest";
// 盤面の座標などは 16x16 のルールで書いている
import modelFile from "../../../model/recent-model/snake-model-16x16.json";
import type { Action } from "./game";
import { rulesFromConfig, type GameRules, type Rules } from "./rules";
import type { Direction, Position } from "./snake";
import { SoloEnv } from "./solo";

const baseRules = (): Rules => rulesFromConfig(modelFile.game as GameRules);

/** リンゴの無い盤面で始める */
function emptySolo(rules: Rules = baseRules()): SoloEnv {
  const env = new SoloEnv(rules);
  env.field.items = [];
  return env;
}

const pos = (x: number, y: number): Position => ({ x, y });

/** 次に移動するティックまで進め、そのティックでは `action` を入力する */
function moveOnce(env: SoloEnv, action?: Action): void {
  while (!env.field.snake.movesNextTick) env.step([]);
  env.step(action ? [action] : []);
}

function setBody(env: SoloEnv, body: Position[], dir: Direction): void {
  const snake = env.field.snake;
  snake.body = body.map((p) => ({ ...p }));
  snake.dir = dir;
  snake.lastMovedDir = dir;
}

describe("一人モード", () => {
  it("所持アイテムは置かず、毒リンゴを 3 倍に増やす", () => {
    const env = new SoloEnv(baseRules());
    const count = (kind: string) => env.field.items.filter((i) => i.kind === kind).length;
    expect([count("normal_apple"), count("gold_apple"), count("poison_apple")]).toEqual([3, 1, 6]);
    expect(env.field.items.length).toBe(10);
  });

  it("毒リンゴで長さが 4 縮み、スコアが 1 減る", () => {
    const env = emptySolo();
    setBody(env, [8, 9, 10, 11, 12, 13, 14].map((y) => pos(8, y)), "up");
    env.field.snake.score = 2;
    env.field.items.push({ pos: pos(8, 7), kind: "poison_apple" });
    moveOnce(env);
    const snake = env.field.snake;
    expect([snake.score, snake.length, snake.alive]).toEqual([1, 3, true]);
  });

  it("アイテム使用では何も起きない", () => {
    const env = emptySolo();
    moveOnce(env, "use_item");
    expect(env.field.snake.heldItem).toBeNull();
    expect(env.field.snake.head).toEqual(pos(8, 7));
  });

  it("ぶつかると終わる", () => {
    const env = emptySolo();
    setBody(env, [pos(8, 0), pos(8, 1), pos(8, 2)], "up");
    moveOnce(env);
    expect(env.result).toBe("death");
  });

  it("すべてのマスを埋めたらクリア", () => {
    const env = emptySolo({ ...baseRules(), width: 5, height: 5 });
    // 左上から 1 行ごとに折り返してたどり、最後のマスだけを空ける
    const path = Array.from({ length: 25 }, (_, i) => {
      const [y, k] = [Math.floor(i / 5), i % 5];
      return pos(y % 2 === 0 ? k : 4 - k, y);
    });
    setBody(env, path.slice(0, 24).reverse(), "right");
    env.field.snake.pendingGrowth = 1;
    moveOnce(env);
    expect(env.result).toBe("filled");
  });

  it("制限時間で終わる", () => {
    const env = emptySolo({ ...baseRules(), normalIntervalTicks: 10_000 });
    for (let i = 0; i < 299; i++) env.step([]);
    expect(env.isOver).toBe(false);
    env.step([]);
    expect(env.result).toBe("time_up");
  });
});
