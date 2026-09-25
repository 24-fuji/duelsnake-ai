// learn/src/env/tests.rs と同じ内容のテスト。両方の実装が同じルールで動くことを確かめる

import { describe, expect, it } from "vitest";
import modelFile from "../../../model/recent-model/snake-model.json";
import { GameEnv, type Action } from "./game";
import { rulesFromConfig, type GameRules, type Rules } from "./rules";
import { manhattan, type Direction, type Position } from "./snake";
import { encodeObservation } from "../ai/observation";
import type { ObservationSpec } from "../ai/model";

const baseRules = (): Rules => rulesFromConfig(modelFile.game as GameRules);
const spec = modelFile.observation as ObservationSpec;

/** アイテムの無い盤面で始める */
function emptyEnv(rules: Rules = baseRules()): GameEnv {
  const env = new GameEnv(rules);
  for (const field of env.fields) field.items = [];
  return env;
}

const pos = (x: number, y: number): Position => ({ x, y });
const none: [Action[], Action[]] = [[], []];

/** プレイヤー0が次に移動するティックまで進め、そのティックでは `action` を入力する */
function moveOnce(env: GameEnv, action?: Action): void {
  while (!env.needsDecision(0)) env.step(none);
  env.step([action ? [action] : [], []]);
}

function setBody(env: GameEnv, player: number, body: Position[], dir: Direction): void {
  const snake = env.fields[player].snake;
  snake.body = body.map((p) => ({ ...p }));
  snake.dir = dir;
  snake.lastMovedDir = dir;
}

describe("ルール", () => {
  it("秒数をティック数に四捨五入する", () => {
    const r = baseRules();
    expect(r.timeLimitTicks).toBe(300);
    expect(r.normalIntervalTicks).toBe(2);
    expect(r.boostIntervalTicks).toBe(1);
    expect(r.boostDurationTicks).toBe(50);
    expect(r.boostCooldownTicks).toBe(75);
    expect(r.appleRespawnTicks).toBe(6);
    expect(r.specialRespawnTicks).toBe(10);
    expect(r.jamDelayTicks).toBe(2);
  });

  it("初期状態", () => {
    const env = new GameEnv(baseRules());
    for (const field of env.fields) {
      expect(field.snake.body).toEqual([pos(8, 8), pos(8, 9), pos(8, 10)]);
      expect(field.snake.dir).toBe("up");
      const count = (kind: string) => field.items.filter((i) => i.kind === kind).length;
      expect([count("normal_apple"), count("poison_apple"), count("gold_apple")]).toEqual([3, 2, 1]);
      expect([count("block_clear"), count("block_jam")]).toEqual([1, 1]);
      for (const item of field.items) {
        expect(field.snake.body).not.toContainEqual(item.pos);
      }
    }
  });

  it("通常は2ティックに1回動き、その直前に行動を決める", () => {
    const env = emptyEnv();
    expect(env.needsDecision(0)).toBe(false);
    env.step(none);
    expect(env.fields[0].snake.head).toEqual(pos(8, 8));
    expect(env.needsDecision(0)).toBe(true);
    env.step(none);
    expect(env.fields[0].snake.head).toEqual(pos(8, 7));
    expect(env.fields[0].snake.length).toBe(3);
  });

  it("直前の移動と逆向きの入力は無視する", () => {
    const env = emptyEnv();
    env.step([["left"], []]);
    env.step([["down"], []]);
    expect(env.fields[0].snake.alive).toBe(true);
    expect(env.fields[0].snake.head).toEqual(pos(7, 8));
  });

  it("同じティックの複数入力は順に反映する", () => {
    const env = emptyEnv();
    env.step(none);
    env.step([["left", "down", "boost"], []]);
    expect(env.fields[0].snake.head).toEqual(pos(7, 8));
    expect(env.fields[0].snake.boostRemaining).toBeGreaterThan(0);
  });

  it("普通のリンゴでスコアが増え、次の移動から伸びる", () => {
    const env = emptyEnv();
    env.fields[0].items.push({ pos: pos(8, 7), kind: "normal_apple" });
    moveOnce(env);
    const snake = env.fields[0].snake;
    expect([snake.score, snake.length, snake.pendingGrowth]).toEqual([1, 3, 1]);
    expect(env.fields[0].items).toEqual([]);

    moveOnce(env);
    expect(env.fields[0].snake.length).toBe(4);
    for (let i = 0; i < 3; i++) env.step(none);
    expect(env.fields[0].items).toEqual([]);
    env.step(none);
    expect(env.fields[0].items.length).toBe(1);
  });

  it("金のリンゴで3マス伸びる", () => {
    const env = emptyEnv();
    env.fields[0].items.push({ pos: pos(8, 7), kind: "gold_apple" });
    for (let i = 0; i < 4; i++) moveOnce(env);
    const snake = env.fields[0].snake;
    expect([snake.score, snake.length]).toEqual([3, 6]);
  });

  it("毒リンゴで即座に縮み、スコアは負にならない", () => {
    const env = emptyEnv();
    env.fields[0].items.push({ pos: pos(8, 7), kind: "poison_apple" });
    moveOnce(env);
    const snake = env.fields[0].snake;
    expect([snake.score, snake.length, snake.alive]).toEqual([0, 1, true]);
  });

  it("毒リンゴは先に伸びる予定を打ち消す", () => {
    const env = emptyEnv();
    env.fields[0].snake.pendingGrowth = 3;
    env.fields[0].items.push({ pos: pos(8, 7), kind: "poison_apple" });
    moveOnce(env);
    const snake = env.fields[0].snake;
    expect([snake.length, snake.pendingGrowth]).toEqual([4, 0]);
  });

  it("伸びている途中でなければ尻尾のマスに進める", () => {
    const body = [pos(8, 8), pos(9, 8), pos(9, 9), pos(8, 9)];
    for (const [growth, alive] of [
      [0, true],
      [1, false],
    ] as const) {
      const env = emptyEnv();
      setBody(env, 0, body, "left");
      env.fields[0].snake.dir = "down";
      env.fields[0].snake.pendingGrowth = growth;
      moveOnce(env);
      expect(env.fields[0].snake.alive).toBe(alive);
    }
  });

  it("壁にぶつかると即敗北", () => {
    const env = emptyEnv();
    setBody(env, 0, [pos(8, 0), pos(8, 1), pos(8, 2)], "up");
    env.fields[0].snake.score = 10;
    moveOnce(env);
    expect(env.result).toEqual({ winner: 1, reason: "death" });
  });

  it("お邪魔ブロックにぶつかると即敗北", () => {
    const env = emptyEnv();
    env.fields[1].obstacles.push(pos(8, 7));
    moveOnce(env);
    expect(env.result?.winner).toBe(0);
  });

  it("同時に死んだらスコアで決める", () => {
    for (const [scores, winner] of [
      [[2, 5], 1],
      [[4, 4], null],
    ] as const) {
      const env = emptyEnv();
      for (const p of [0, 1]) {
        setBody(env, p, [pos(3, 0), pos(3, 1)], "up");
        env.fields[p].snake.score = scores[p];
      }
      moveOnce(env);
      expect(env.result).toEqual({ winner, reason: "death" });
    }
  });

  it("時間切れはスコアで決める", () => {
    const env = emptyEnv({ ...baseRules(), normalIntervalTicks: 10_000 });
    env.fields[0].snake.score = 3;
    for (let i = 0; i < 299; i++) env.step(none);
    expect(env.isOver).toBe(false);
    env.step(none);
    expect(env.result).toEqual({ winner: 0, reason: "time_up" });
  });

  it("お邪魔ブロックは遅れて相手の頭から離れた位置に出る", () => {
    for (let n = 0; n < 20; n++) {
      const env = new GameEnv(baseRules());
      env.fields[0].snake.heldItem = "block_jam";
      env.step([["use_item"], []]);
      expect(env.fields[0].snake.heldItem).toBeNull();
      env.step(none);
      expect(env.fields[1].obstacles).toEqual([]);
      env.step(none);
      expect(env.fields[1].obstacles.length).toBe(1);
      const block = env.fields[1].obstacles[0];
      expect(manhattan(block, env.fields[1].snake.head)).toBeGreaterThan(2);
      expect(env.fields[1].items.map((i) => i.pos)).not.toContainEqual(block);
    }
  });

  it("ブロック消去は自分のブロックだけ消す", () => {
    const env = emptyEnv();
    env.fields[0].obstacles = [pos(0, 0), pos(1, 1)];
    env.fields[1].obstacles = [pos(0, 0)];
    env.fields[0].snake.heldItem = "block_clear";
    env.step([["use_item"], []]);
    expect(env.fields[0].obstacles).toEqual([]);
    expect(env.fields[1].obstacles.length).toBe(1);
  });

  it("ブースト中は毎ティック動き、終了後はクールダウンに入る", () => {
    const env = emptyEnv({ ...baseRules(), boostDurationTicks: 6, boostCooldownTicks: 4 });
    setBody(env, 0, [pos(0, 15)], "right");
    moveOnce(env, "boost");
    expect(env.fields[0].snake.head).toEqual(pos(1, 15));
    const xs: number[] = [];
    for (let i = 0; i < 8; i++) {
      env.step(none);
      xs.push(env.fields[0].snake.head.x);
    }
    expect(xs).toEqual([2, 3, 4, 5, 6, 6, 7, 7]);
    expect(env.fields[0].snake.boostReady).toBe(false);
    env.step(none);
    expect(env.fields[0].snake.boostReady).toBe(true);
  });
});

describe("観測", () => {
  it("自分のフィールドから見た観測を作る", () => {
    const env = emptyEnv();
    env.fields[0].items.push({ pos: pos(2, 3), kind: "gold_apple" });
    env.fields[0].snake.score = 5;
    env.fields[1].snake.heldItem = "block_jam";

    const plane = 16 * 16;
    const grid = new Float32Array(9 * plane);
    const vector = new Float32Array(13);
    encodeObservation(env, 0, spec, grid, vector);
    const at = (c: number, x: number, y: number) => grid[c * plane + y * 16 + x];

    expect(at(0, 8, 8)).toBe(1);
    expect(at(1, 8, 9)).toBe(1);
    expect(at(1, 8, 10)).toBe(0.5);
    expect(at(3, 2, 3)).toBe(1);
    expect(grid.subarray(8 * plane).every((v) => v === 1)).toBe(true);
    expect(Array.from(vector.subarray(0, 4))).toEqual([1, 0, 0, 0]);
    expect([vector[8], vector[9], vector[10], vector[11]]).toEqual([1, 1, 0.5, 1]);

    encodeObservation(env, 1, spec, grid, vector);
    expect([vector[10], vector[11]]).toEqual([-0.5, 0]);
  });
});
