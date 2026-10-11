import { Field } from "../game/field";
import type { Action, GameEnv } from "../game/game";
import { DIRECTIONS, stepPosition, type Direction } from "../game/snake";

/** 人間の操作を AI が補助する範囲。衝突を選んでしまったとき、その範囲の衝突なら AI の判断に差し替える */
export type AssistLevel = "none" | "wall" | "all";

/** 難易度名ごとの補助の範囲 (かんたん: 壁・お邪魔ブロック・自分の体、ふつう: 外周の壁のみ) */
export function assistLevel(difficulty: string): AssistLevel {
  if (difficulty === "easy") return "all";
  if (difficulty === "medium") return "wall";
  return "none";
}

/** このティックの入力を反映したあとの進行方向。逆走の入力は無視される (Snake.turn と同じ) */
function directionAfter(env: GameEnv, player: number, actions: readonly Action[]): Direction {
  const snake = env.fields[player].snake;
  let dir = snake.dir;
  for (const action of actions) {
    if ((DIRECTIONS as readonly string[]).includes(action) && !isReverse(snake.lastMovedDir, action as Direction)) {
      dir = action as Direction;
    }
  }
  return dir;
}

function isReverse(a: Direction, b: Direction): boolean {
  return (
    (a === "up" && b === "down") || (a === "down" && b === "up") || (a === "left" && b === "right") || (a === "right" && b === "left")
  );
}

/** `actions` を入力したとき、次の移動が補助の対象になる衝突になるか */
export function collisionAhead(env: GameEnv, player: number, actions: readonly Action[], level: AssistLevel): boolean {
  if (level === "none" || !env.needsDecision(player)) return false;
  const field = env.fields[player];
  const next = stepPosition(field.snake.head, directionAfter(env, player, actions));
  if (!Field.inBounds(next, env.rules)) return true;
  if (level === "wall") return false;
  return field.hasObstacle(next) || field.snake.collidesWithSelf(next);
}
