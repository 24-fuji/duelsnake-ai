/** AI の入力 (観測) の作り方。learn/src/env/observation.rs と同じ手順 (learn/RULES.md 10章) */

import type { Field, ItemType } from "../game/field";
import type { GameEnv } from "../game/game";
import type { Rules } from "../game/rules";
import { DIRECTIONS, type Position } from "../game/snake";
import type { ObservationSpec } from "./model";

/**
 * 盤面チャネル。値は [チャネル][y][x] の順に並べる。
 * 0〜8 が自分のフィールド、9〜16 が相手のフィールド (自分と同じ並び)。
 * 相手の分は後から末尾に足したので、先頭 9 チャネルだけを使う古いモデルもそのまま動かせる (inputSizes)
 */
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
  "opp_head",
  "opp_body",
  "opp_normal_apple",
  "opp_gold_apple",
  "opp_poison_apple",
  "opp_block_clear_item",
  "opp_block_jam_item",
  "opp_obstacle",
] as const;

/** 状態ベクトル。0〜12 が自分の状態と相手との比較、13〜21 が相手の状態 (後から末尾に足した) */
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
  "opp_dir_up",
  "opp_dir_down",
  "opp_dir_left",
  "opp_dir_right",
  "opp_pending_growth",
  "opp_held_block_clear",
  "opp_boost_remaining",
  "opp_boost_ready",
  "opp_incoming_jams",
] as const;

/** 自分のフィールドの最初のチャネル (own_head) と、相手のフィールドの最初のチャネル (opp_head) */
const OWN_FIELD_CHANNEL = 0;
const OPP_FIELD_CHANNEL = 9;
/** すべてのマスが 1 のチャネル */
const IN_BOUNDS_CHANNEL = 8;

/** フィールドの最初のチャネル (頭) から数えた、各アイテムのチャネルの位置 */
const ITEM_CHANNEL: Record<ItemType, number> = {
  normal_apple: 2,
  gold_apple: 3,
  poison_apple: 4,
  block_clear: 5,
  block_jam: 6,
};

/** `encodeObservation` が書く観測全体の長さ (盤面, 状態ベクトル) */
export function fullObservationLengths(rules: Rules): { grid: number; vector: number } {
  return { grid: GRID_CHANNEL_NAMES.length * rules.width * rules.height, vector: VECTOR_FEATURE_NAMES.length };
}

function isPrefix(names: readonly string[], all: readonly string[]): boolean {
  return names.length > 0 && names.length <= all.length && names.every((n, i) => n === all[i]);
}

/**
 * モデルの観測の名前の並びが、この観測の先頭部分と一致すれば、そのチャネル数と状態ベクトルの長さを返す。
 * 観測全体を書いたあと、盤面は先頭のチャネル数分、状態ベクトルは先頭の長さ分をモデルに渡せばよい
 */
export function inputSizes(spec: ObservationSpec): { gridChannels: number; vectorFeatures: number } | null {
  if (!isPrefix(spec.grid_channels, GRID_CHANNEL_NAMES) || !isPrefix(spec.vector_features, VECTOR_FEATURE_NAMES)) {
    return null;
  }
  return { gridChannels: spec.grid_channels.length, vectorFeatures: spec.vector_features.length };
}

/** モデルの観測仕様がこの実装・ルールと一致するか確かめる。一致しなければ例外 */
export function checkObservationSpec(spec: ObservationSpec, rules: Rules): void {
  if (inputSizes(spec) === null) {
    throw new Error("モデルの観測仕様がこの Web 版と一致しません (learn と web のバージョンを揃えてください)");
  }
  if (spec.grid_width !== rules.width || spec.grid_height !== rules.height) {
    throw new Error(`モデルの盤面サイズ ${spec.grid_width}x${spec.grid_height} がルールと一致しません`);
  }
}

/** 1つのフィールドを `first` チャネルから始まる 8 チャネル (頭・胴体・アイテム 5 種・お邪魔ブロック) に書き込む */
function encodeField(field: Field, width: number, plane: number, first: number, grid: Float32Array): void {
  const cell = (channel: number, p: Position) => (first + channel) * plane + p.y * width + p.x;
  const snake = field.snake;
  grid[cell(0, snake.head)] = 1;
  // 胴体は首が 1、尻尾に向かって 1 / (長さ - 1) まで小さくなる
  const len = snake.length;
  for (let i = 1; i < len; i++) {
    grid[cell(1, snake.body[i])] = (len - i) / (len - 1);
  }
  for (const item of field.items) {
    grid[cell(ITEM_CHANNEL[item.kind], item.pos)] = 1;
  }
  for (const p of field.obstacles) {
    grid[cell(7, p)] = 1;
  }
}

/**
 * `player` から見た観測全体を書き込む。`grid` と `vector` の長さは fullObservationLengths のとおり。
 * 古いモデルには、このうち先頭部分 (inputSizes) だけを渡す
 */
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
  const full = fullObservationLengths(rules);
  if (grid.length !== full.grid || vector.length !== full.vector) {
    throw new Error(`観測の配列の長さが違います: grid ${grid.length} (期待 ${full.grid}), vector ${vector.length} (期待 ${full.vector})`);
  }
  const field = env.fields[player];
  const opponent = env.fields[1 - player];

  grid.fill(0);
  grid.fill(1, IN_BOUNDS_CHANNEL * plane, (IN_BOUNDS_CHANNEL + 1) * plane);
  encodeField(field, w, plane, OWN_FIELD_CHANNEL, grid);
  encodeField(opponent, w, plane, OPP_FIELD_CHANNEL, grid);

  const snake = field.snake;
  const opp = opponent.snake;
  const flag = (b: boolean) => (b ? 1 : 0);
  vector.fill(0);
  vector[DIRECTIONS.indexOf(snake.lastMovedDir)] = 1;
  vector[4] = Math.min(snake.pendingGrowth / spec.pending_growth_scale, 1);
  vector[5] = flag(snake.heldItem === "block_clear");
  vector[6] = flag(snake.heldItem === "block_jam");
  vector[7] = snake.boostRemaining / rules.boostDurationTicks;
  vector[8] = flag(snake.boostReady);
  vector[9] = env.remainingTicks / rules.timeLimitTicks;
  vector[10] = Math.min(Math.max((snake.score - opp.score) / spec.score_diff_scale, -1), 1);
  vector[11] = flag(opp.heldItem === "block_jam");
  vector[12] = Math.min(field.incomingJams.length / spec.incoming_jams_scale, 1);

  // 相手の状態。相手がお邪魔を持っているかは 11 番にあるので、ここには入れない
  vector[13 + DIRECTIONS.indexOf(opp.lastMovedDir)] = 1;
  vector[17] = Math.min(opp.pendingGrowth / spec.pending_growth_scale, 1);
  vector[18] = flag(opp.heldItem === "block_clear");
  vector[19] = opp.boostRemaining / rules.boostDurationTicks;
  vector[20] = flag(opp.boostReady);
  vector[21] = Math.min(opponent.incomingJams.length / spec.incoming_jams_scale, 1);
}
