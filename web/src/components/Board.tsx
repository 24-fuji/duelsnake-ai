import { useEffect, useRef } from "react";
import type { Field } from "../game/field";
import type { Rules } from "../game/rules";
import { drawField, type Palette } from "./draw";

/** 盤面の幅と高さがおよそこのピクセル数に収まるようにマスの大きさを決める (16 マスなら 24px) */
const BOARD_PIXELS = 384;

function defaultCellSize(rules: Rules): number {
  return Math.min(32, Math.max(8, Math.floor(BOARD_PIXELS / Math.max(rules.width, rules.height))));
}

interface Props {
  field: Field;
  rules: Rules;
  palette: Palette;
  /** ゲームが進むたびに増える値。変わったら描き直す */
  version: number;
  cellSize?: number;
}

export function Board({ field, rules, palette, version, cellSize = defaultCellSize(rules) }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (ctx) drawField(ctx, field, rules, cellSize, palette);
  }, [field, rules, palette, version, cellSize]);

  return (
    <canvas
      ref={canvasRef}
      className="board"
      width={rules.width * cellSize}
      height={rules.height * cellSize}
    />
  );
}
