import { useEffect, useRef } from "react";
import type { Field } from "../game/field";
import type { Rules } from "../game/rules";
import { drawField, type Palette } from "./draw";

interface Props {
  field: Field;
  rules: Rules;
  palette: Palette;
  /** ゲームが進むたびに増える値。変わったら描き直す */
  version: number;
  cellSize?: number;
}

export function Board({ field, rules, palette, version, cellSize = 24 }: Props) {
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
