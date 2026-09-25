/**
 * 盤面の描画。今は図形で描いている。
 * 画像に差し替えるときは src/assets/ に PNG を置いて import し、各関数を drawImage に置き換える
 */

import type { Field, ItemType } from "../game/field";
import type { Rules } from "../game/rules";
import type { Direction } from "../game/snake";

export interface Palette {
  head: string;
  body: string;
}

export const PLAYER_PALETTES: readonly [Palette, Palette] = [
  { head: "#4fc3f7", body: "#0288d1" },
  { head: "#ffb74d", body: "#ef6c00" },
];

const BACKGROUND = "#1b1f24";
const GRID_LINE = "#262b33";
const OBSTACLE = "#78909c";

const ITEM_STYLE: Record<ItemType, { color: string; label?: string }> = {
  normal_apple: { color: "#e53935" },
  gold_apple: { color: "#fdd835" },
  poison_apple: { color: "#8e24aa" },
  block_clear: { color: "#26c6da", label: "C" },
  block_jam: { color: "#ec407a", label: "J" },
};

export function itemColor(kind: ItemType): string {
  return ITEM_STYLE[kind].color;
}

export function drawField(
  ctx: CanvasRenderingContext2D,
  field: Field,
  rules: Rules,
  cell: number,
  palette: Palette,
): void {
  const w = rules.width * cell;
  const h = rules.height * cell;
  ctx.fillStyle = BACKGROUND;
  ctx.fillRect(0, 0, w, h);
  ctx.strokeStyle = GRID_LINE;
  ctx.lineWidth = 1;
  for (let i = 1; i < rules.width; i++) line(ctx, i * cell + 0.5, 0, i * cell + 0.5, h);
  for (let i = 1; i < rules.height; i++) line(ctx, 0, i * cell + 0.5, w, i * cell + 0.5);

  for (const p of field.obstacles) {
    ctx.fillStyle = OBSTACLE;
    ctx.fillRect(p.x * cell + 1, p.y * cell + 1, cell - 2, cell - 2);
    ctx.strokeStyle = "#37474f";
    ctx.lineWidth = 2;
    line(ctx, p.x * cell + 4, p.y * cell + 4, (p.x + 1) * cell - 4, (p.y + 1) * cell - 4);
    line(ctx, (p.x + 1) * cell - 4, p.y * cell + 4, p.x * cell + 4, (p.y + 1) * cell - 4);
  }

  for (const item of field.items) {
    drawItem(ctx, item.kind, item.pos.x * cell, item.pos.y * cell, cell);
  }

  drawSnake(ctx, field, cell, palette);
}

function line(ctx: CanvasRenderingContext2D, x0: number, y0: number, x1: number, y1: number): void {
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

function drawItem(ctx: CanvasRenderingContext2D, kind: ItemType, x: number, y: number, cell: number): void {
  const style = ITEM_STYLE[kind];
  ctx.fillStyle = style.color;
  if (style.label) {
    ctx.fillRect(x + 3, y + 3, cell - 6, cell - 6);
    ctx.fillStyle = "#102027";
    ctx.font = `bold ${Math.floor(cell * 0.6)}px sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(style.label, x + cell / 2, y + cell / 2 + 1);
  } else {
    ctx.beginPath();
    ctx.arc(x + cell / 2, y + cell / 2, cell * 0.38, 0, Math.PI * 2);
    ctx.fill();
  }
}

const EYE_OFFSET: Record<Direction, [number, number]> = {
  up: [0, -1],
  down: [0, 1],
  left: [-1, 0],
  right: [1, 0],
};

function drawSnake(ctx: CanvasRenderingContext2D, field: Field, cell: number, palette: Palette): void {
  const snake = field.snake;
  const len = snake.length;
  for (let i = len - 1; i >= 1; i--) {
    const p = snake.body[i];
    ctx.globalAlpha = 1 - (0.5 * (i - 1)) / Math.max(1, len - 1);
    ctx.fillStyle = palette.body;
    ctx.fillRect(p.x * cell + 2, p.y * cell + 2, cell - 4, cell - 4);
  }
  ctx.globalAlpha = 1;

  const head = snake.head;
  ctx.fillStyle = snake.alive ? palette.head : "#d32f2f";
  ctx.fillRect(head.x * cell + 1, head.y * cell + 1, cell - 2, cell - 2);
  // 進行方向に目を描く
  const [dx, dy] = EYE_OFFSET[snake.alive ? snake.dir : snake.lastMovedDir];
  const cx = head.x * cell + cell / 2 + dx * cell * 0.22;
  const cy = head.y * cell + cell / 2 + dy * cell * 0.22;
  ctx.fillStyle = "#102027";
  ctx.beginPath();
  ctx.arc(cx, cy, cell * 0.12, 0, Math.PI * 2);
  ctx.fill();
}
