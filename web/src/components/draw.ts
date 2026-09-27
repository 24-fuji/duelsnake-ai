/**
 * 盤面の描画。アイテムとお邪魔ブロックは src/assets/items/ の画像で、ヘビは図形で描く
 */

import blockClearUrl from "../assets/items/block_clear.png";
import blockJamUrl from "../assets/items/block_jam.png";
import goldAppleUrl from "../assets/items/gold_apple.png";
import normalAppleUrl from "../assets/items/normal_apple.png";
import obstacleUrl from "../assets/items/obstacle.png";
import poisonAppleUrl from "../assets/items/poison_apple.png";
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

type Sprite = ItemType | "obstacle";

/** 画像はどれも余白を削った正方形 */
const SPRITE_URLS: Record<Sprite, string> = {
  normal_apple: normalAppleUrl,
  gold_apple: goldAppleUrl,
  poison_apple: poisonAppleUrl,
  block_clear: blockClearUrl,
  block_jam: blockJamUrl,
  obstacle: obstacleUrl,
};

export function itemImageUrl(kind: ItemType): string {
  return SPRITE_URLS[kind];
}

const sprites = new Map<Sprite, HTMLImageElement>();
let spritesLoading: Promise<void> | undefined;

/** 盤面の画像を読み込む。読み込み終わるまでは画像を描かないので、終わったら描き直す */
export function loadBoardImages(): Promise<void> {
  spritesLoading ??= Promise.all(
    (Object.keys(SPRITE_URLS) as Sprite[]).map((sprite) => {
      const img = new Image();
      img.src = SPRITE_URLS[sprite];
      sprites.set(sprite, img);
      return img.decode().catch(() => undefined);
    }),
  ).then(() => undefined);
  return spritesLoading;
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

  // 大きな画像を縮めて描くので、粗くならないよう高品質に補間する
  ctx.imageSmoothingQuality = "high";
  for (const p of field.obstacles) {
    drawSprite(ctx, "obstacle", p.x * cell, p.y * cell, cell);
  }
  for (const item of field.items) {
    drawSprite(ctx, item.kind, item.pos.x * cell, item.pos.y * cell, cell);
  }

  drawSnake(ctx, field, cell, palette);
}

function line(ctx: CanvasRenderingContext2D, x0: number, y0: number, x1: number, y1: number): void {
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

/** マスの枠線にかからないよう 1px 内側に描く */
function drawSprite(ctx: CanvasRenderingContext2D, sprite: Sprite, x: number, y: number, cell: number): void {
  const img = sprites.get(sprite);
  if (!img?.complete || img.naturalWidth === 0) return;
  ctx.drawImage(img, x + 1, y + 1, cell - 2, cell - 2);
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
