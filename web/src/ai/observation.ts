/** AI の入力 (観測) の作り方。learn/src/env/observation.rs と同じ手順 (learn/RULES.md 10章) */

import type { ItemType } from "../game/field";
import type { GameEnv } from "../game/game";
import type { Rules } from "../game/rules";
import { DIRECTIONS } from "../game/snake";
import type { ObservationSpec } from "./model";

/** 盤面チャネル。値は [チャネル][y][x] の順に並べる */
export const GRID_CHANNEL_NAMES = [
  "own_head",
  "own_body",
  "normal_apple",
  "gold_apple",
  "poison_apple",
  "block_clear_item",
  "block_jam_item",
  "obstacle",
  "in_bounds",
] as const;

export const VECTOR_FEATURE_NAMES = [
  "dir_up",
  "dir_down",
  "dir_left",
  "dir_right",
  "pending_growth",
  "held_block_clear",
  "held_block_jam",
  "boost_remaining",
  "boost_ready",
  "time_remaining",
  "score_diff",
  "opponent_holds_jam",
  "incoming_jams",
] as const;

const ITEM_CHANNEL: Record<ItemType, number> = {
  normal_apple: 2,
  gold_apple: 3,
  poison_apple: 4,
  block_clear: 5,
  block_jam: 6,
};

function sameNames(names: readonly string[], expected: readonly string[]): boolean {
  return names.length === expected.length && names.every((n, i) => n === expected[i]);
}

/** モデルの観測仕様がこの実装・ルールと一致するか確かめる。一致しなければ例外 */
export function checkObservationSpec(spec: ObservationSpec, rules: Rules): void {
  if (!sameNames(spec.grid_channels, GRID_CHANNEL_NAMES) || !sameNames(spec.vector_features, VECTOR_FEATURE_NAMES)) {
    throw new Error("モデルの観測仕様がこの Web 版と一致しません (learn と web のバージョンを揃えてください)");
  }
  if (spec.grid_width !== rules.width || spec.grid_height !== rules.height) {
    throw new Error(`モデルの盤面サイズ ${spec.grid_width}x${spec.grid_height} がルールと一致しません`);
  }
}

/** `player` から見た観測を書き込む */
export function encodeObservation(
  env: GameEnv,
  player: number,
  spec: ObservationSpec,
  grid: Float32Array,
  vector: Float32Array,
): void {
  const rules = env.rules;
  const w = rules.width;
  const plane = w * rules.height;
  const field = env.fields[player];
  const opponent = env.fields[1 - player];
  const snake = field.snake;
  const cell = (channel: number, x: number, y: number) => channel * plane + y * w + x;

  grid.fill(0);
  grid.fill(1, 8 * plane, 9 * plane);

  grid[cell(0, snake.head.x, snake.head.y)] = 1;
  // 胴体は首が 1、尻尾に向かって 1 / (長さ - 1) まで小さくなる
  const len = snake.length;
  for (let i = 1; i < len; i++) {
    const p = snake.body[i];
    grid[cell(1, p.x, p.y)] = (len - i) / (len - 1);
  }
  for (const item of field.items) {
    grid[cell(ITEM_CHANNEL[item.kind], item.pos.x, item.pos.y)] = 1;
  }
  for (const p of field.obstacles) {
    grid[cell(7, p.x, p.y)] = 1;
  }

  const flag = (b: boolean) => (b ? 1 : 0);
  vector.fill(0);
  vector[DIRECTIONS.indexOf(snake.lastMovedDir)] = 1;
  vector[4] = Math.min(snake.pendingGrowth / spec.pending_growth_scale, 1);
  vector[5] = flag(snake.heldItem === "block_clear");
  vector[6] = flag(snake.heldItem === "block_jam");
  vector[7] = snake.boostRemaining / rules.boostDurationTicks;
  vector[8] = flag(snake.boostReady);
  vector[9] = env.remainingTicks / rules.timeLimitTicks;
  vector[10] = Math.min(Math.max((snake.score - opponent.snake.score) / spec.score_diff_scale, -1), 1);
  vector[11] = flag(opponent.snake.heldItem === "block_jam");
  vector[12] = Math.min(field.incomingJams.length / spec.incoming_jams_scale, 1);
}
