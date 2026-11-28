// learn/src/safety.rs と learn/src/trap.rs と同じ内容のテスト。両方の安全装置が同じ手を選ぶことを確かめる

import { describe, expect, it } from "vitest";
import modelFile from "../../../model/recent-model/snake-model-16x16.json";
import type { Field } from "../game/field";
import { ACTIONS, GameEnv } from "../game/game";
import { rulesFromConfig, type GameRules } from "../game/rules";
import type { Direction } from "../game/snake";
import { argmax, chooseSafely, Class, escapeBy, judge, moveOf } from "./safety";

const rules = rulesFromConfig(modelFile.game as GameRules);

/** アイテムの無いフィールドに、`body` (先頭が頭) のヘビを `dir` 向きで置く */
function field(body: [number, number][], dir: Direction, jams: [number, number][] = []): Field {
  const f = new GameEnv(rules).fields[0];
  f.items = [];
  f.snake.body = body.map(([x, y]) => ({ x, y }));
  f.snake.dir = dir;
  f.snake.lastMovedDir = dir;
  f.obstacles = jams.map(([x, y]) => ({ x, y }));
  return f;
}

/** Q 値を行動の並び (up, down, left, right, boost, use_item) で作る */
const q = (values: number[]) => Float32Array.from(values);

describe("安全装置", () => {
  it("Q 値が最大の行動が安全なら、そのまま選ぶ", () => {
    const f = field([[4, 4], [4, 5], [4, 6]], "up");
    expect(chooseSafely(f, rules, ACTIONS, q([1, 0, 0, 0, 0, 0]))).toEqual({ index: 0, overridden: false });
  });

  it("次の一手で衝突する手を避ける", () => {
    // 上端で上を向いている。上・ブースト・アイテム使用は壁にぶつかり、下は逆走で無視されて上に進む
    const f = field([[4, 0], [4, 1], [4, 2]], "up");
    expect(chooseSafely(f, rules, ACTIONS, q([5, 9, 1, 2, 4, 3]))).toEqual({ index: 3, overridden: true });
  });

  it("体より狭い袋小路に入る手を避ける", () => {
    const f = field([[0, 2], [0, 3], [0, 4], [1, 4], [2, 4]], "up", [[1, 1], [1, 0]]);
    expect(judge(f, rules, "up", false)).toEqual({ class: Class.Trapped, room: 2 });
    expect(judge(f, rules, "right", false).class).toBe(Class.Open);
    expect(chooseSafely(f, rules, ACTIONS, q([9, 0, 0, 1, 0, 0]))).toEqual({ index: 3, overridden: true });
  });

  it("どの手も詰みなら、行けるマスが多い手を選ぶ", () => {
    const f = field(
      [[2, 1], [2, 2], [2, 3], [2, 4], [3, 4], [4, 4]],
      "up",
      [[1, 0], [2, 0], [3, 0], [4, 0], [0, 1], [5, 1], [1, 2], [4, 2], [3, 3]],
    );
    expect(judge(f, rules, "up", false).class).toBe(Class.Collide);
    expect(judge(f, rules, "left", false)).toEqual({ class: Class.Trapped, room: 1 });
    expect(judge(f, rules, "right", false)).toEqual({ class: Class.Trapped, room: 3 });
    expect(chooseSafely(f, rules, ACTIONS, q([0, 0, 9, 1, 0, 0]))).toEqual({ index: 3, overridden: true });
  });

  it("ブロック消去を持っていれば、アイテム使用はブロックを消してから進む手として調べる", () => {
    const f = field([[4, 4], [4, 5], [4, 6]], "up", [[4, 3]]);
    f.snake.heldItem = "block_clear";
    expect(moveOf(f, "use_item")).toEqual({ dir: "up", cleared: true });
    expect(judge(f, rules, "up", false).class).toBe(Class.Collide);
    expect(judge(f, rules, "up", true).class).toBe(Class.Open);
    expect(chooseSafely(f, rules, ACTIONS, q([2, 0, 0, 0, 9, 5]))).toEqual({ index: 5, overridden: true });
    f.snake.heldItem = "block_jam";
    expect(moveOf(f, "use_item")).toEqual({ dir: "up", cleared: false });
  });

  it("探索が上限を越えたら判定できず", () => {
    const snapshot = {
      body: [{ x: 0, y: 2 }, { x: 0, y: 3 }, { x: 0, y: 4 }, { x: 1, y: 4 }, { x: 2, y: 4 }],
      pendingGrowth: 0,
      lastMovedDir: "up" as Direction,
      obstacles: [{ x: 1, y: 1 }, { x: 1, y: 0 }],
    };
    expect(escapeBy(snapshot, 5, 5, "up", 1000)).toBe(Class.Trapped);
    expect(escapeBy(snapshot, 5, 5, "right", 1000)).toBe(Class.Open);
    expect(escapeBy(snapshot, 5, 5, "left", 1000)).toBe(Class.Trapped);
    expect(escapeBy(snapshot, 5, 5, "down", 1000)).toBe(Class.Trapped);
    expect(escapeBy(snapshot, 5, 5, "right", 1)).toBe(Class.Unknown);
  });

  it("伸びている途中で最後の空きマスに入れば助かる", () => {
    // 2x2 の盤面で空きは 1 マス。伸びる予定があれば、そこに入って盤面が埋まる
    const snapshot = {
      body: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }],
      pendingGrowth: 1,
      lastMovedDir: "left" as Direction,
      obstacles: [],
    };
    expect(escapeBy(snapshot, 2, 2, "down", 1000)).toBe(Class.Open);
  });

  it("小さい盤面で、枝刈りも近道もしない総当たりと同じ判定をする", () => {
    // 乱数で歩かせたヘビと、空いたマスに置いたお邪魔ブロックで調べる (learn の trap.rs のテストと同じ考え方)
    let seed = 42;
    const rand = (n: number) => {
      seed = (seed * 1103515245 + 12345) % 2147483648;
      return Math.floor((seed / 2147483648) * n);
    };
    const opposite: Record<Direction, Direction> = { up: "down", down: "up", left: "right", right: "left" };
    const delta: Record<Direction, [number, number]> = { up: [0, -1], down: [0, 1], left: [-1, 0], right: [1, 0] };
    const dirs: Direction[] = ["up", "down", "left", "right"];
    let [open, trapped] = [0, 0];
    for (let t = 0; t < 2000; t++) {
      const [w, h] = [3 + rand(3), 3 + rand(3)];
      const len = 2 + rand(w * h - 3);
      const walk = [{ x: rand(w), y: rand(h) }];
      let last: Direction | null = null;
      while (walk.length < len) {
        const p = walk[walk.length - 1];
        const options = dirs.filter((d) => {
          const [nx, ny] = [p.x + delta[d][0], p.y + delta[d][1]];
          return nx >= 0 && nx < w && ny >= 0 && ny < h && !walk.some((q) => q.x === nx && q.y === ny);
        });
        if (options.length === 0) break;
        last = options[rand(options.length)];
        walk.push({ x: p.x + delta[last][0], y: p.y + delta[last][1] });
      }
      if (last === null) continue;
      walk.reverse();
      const obstacles: { x: number; y: number }[] = [];
      for (let k = rand(3); k > 0; k--) {
        const p = { x: rand(w), y: rand(h) };
        if (![...walk, ...obstacles].some((q) => q.x === p.x && q.y === p.y)) obstacles.push(p);
      }
      if (walk.length + obstacles.length >= w * h) continue;
      const snapshot = { body: walk, pendingGrowth: rand(3), lastMovedDir: last, obstacles };

      // 総当たり: 体のマスは尻尾から順に空き、経路のマスは goal 手まで空かない
      const len0 = walk.length;
      const growth = snapshot.pendingGrowth;
      const freeAt = new Map<string, number>();
      obstacles.forEach((p) => freeAt.set(`${p.x},${p.y}`, Infinity));
      walk.forEach((p, i) => freeAt.set(`${p.x},${p.y}`, growth + len0 - i));
      const fillAt = w * h - obstacles.length - len0;
      const goal = fillAt <= growth ? Math.min(growth + len0, fillAt) : growth + len0;
      const onPath = new Set<string>();
      const go = (x: number, y: number, lastDir: Direction, moves: number, only: Direction | null): boolean => {
        if (moves >= goal) return true;
        for (const d of dirs) {
          if (d === opposite[lastDir] || (only !== null && d !== only)) continue;
          const [nx, ny] = [x + delta[d][0], y + delta[d][1]];
          const key = `${nx},${ny}`;
          if (nx < 0 || nx >= w || ny < 0 || ny >= h || onPath.has(key) || (freeAt.get(key) ?? 0) > moves + 1) continue;
          onPath.add(key);
          const found = go(nx, ny, d, moves + 1, null);
          onPath.delete(key);
          if (found) return true;
        }
        return false;
      };
      for (const d of dirs) {
        const expected = go(walk[0].x, walk[0].y, last, 0, d);
        expect(escapeBy(snapshot, w, h, d, Infinity) === Class.Open, JSON.stringify({ snapshot, w, h, d })).toBe(expected);
        if (expected) open++;
        else trapped++;
      }
    }
    expect(open).toBeGreaterThan(300);
    expect(trapped).toBeGreaterThan(300);
  });

  it("argmax は同じ値なら先の行動を選ぶ", () => {
    expect(argmax([1, 3, 3, 0])).toBe(1);
  });
});
