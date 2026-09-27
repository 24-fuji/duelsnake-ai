import { useEffect, useRef, useState } from "react";
import type { Field } from "../game/field";
import type { Rules } from "../game/rules";
import { drawField, loadBoardImages, type Palette } from "./draw";

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
  /** 1マスの大きさ (CSS ピクセル)。省略すると盤面のマス数から決める */
  cellSize?: number;
  /** 親要素の幅いっぱいに広げる (cellSize は使わない) */
  fill?: boolean;
}

export function Board({ field, rules, palette, version, cellSize, fill = false }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [fillWidth, setFillWidth] = useState(0);
  const [imagesLoaded, setImagesLoaded] = useState(false);

  useEffect(() => {
    let active = true;
    void loadBoardImages().then(() => active && setImagesLoaded(true));
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!fill || !canvas) return;
    const observer = new ResizeObserver(([entry]) => setFillWidth(entry.contentRect.width));
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [fill]);

  const cell = fill ? fillWidth / rules.width : (cellSize ?? defaultCellSize(rules));
  // 高解像度の画面でもぼやけないよう、画面の画素数に合わせて描く
  const scale = window.devicePixelRatio || 1;

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx || cell <= 0) return;
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    drawField(ctx, field, rules, cell, palette);
  }, [field, rules, palette, version, cell, scale, imagesLoaded]);

  return (
    <canvas
      ref={canvasRef}
      className={fill ? "board fill" : "board"}
      width={Math.round(rules.width * cell * scale)}
      height={Math.round(rules.height * cell * scale)}
      style={
        fill
          ? { aspectRatio: `${rules.width} / ${rules.height}` }
          : { width: rules.width * cell, height: rules.height * cell }
      }
    />
  );
}
