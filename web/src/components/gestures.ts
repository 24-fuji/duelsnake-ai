/**
 * 携帯版のタッチ操作。1本の指の動きから、スワイプ・長押し・タップ・ダブルタップを見分ける。
 * Pointer Events で受けるので、マウスでも同じように操作できる
 */

import { useEffect, useMemo, useRef, type PointerEvent } from "react";
import type { Direction } from "../game/snake";

export type Gesture =
  | { kind: "swipe"; direction: Direction }
  | { kind: "long_press" }
  | { kind: "tap" }
  | { kind: "double_tap" };

export interface GestureOptions {
  /** 指がこの距離 (px) 動くごとに、動いた向きのスワイプとみなす */
  swipeDistance: number;
  /** 指を動かさずにこの時間 (ms) 押し続けたら長押し */
  longPressMs: number;
  /** タップを離してからこの時間 (ms) 以内に次のタップを始めたらダブルタップ */
  doubleTapMs: number;
}

export const DEFAULT_GESTURE_OPTIONS: GestureOptions = { swipeDistance: 24, longPressMs: 350, doubleTapMs: 300 };

interface Touch {
  id: number;
  /** スワイプの向きを測る基準の位置 */
  x: number;
  y: number;
  swiped: boolean;
  longPressed: boolean;
  lastDirection: Direction | null;
  /** 直前のタップのすぐ後に始まったか */
  followsTap: boolean;
}

export class GestureRecognizer {
  private touch: Touch | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private lastTapEnd = -Infinity;
  private readonly emit: (gesture: Gesture) => void;
  private readonly options: GestureOptions;

  constructor(emit: (gesture: Gesture) => void, options: GestureOptions = DEFAULT_GESTURE_OPTIONS) {
    this.emit = emit;
    this.options = options;
  }

  down(id: number, x: number, y: number, time: number): void {
    // 2本目以降の指は無視する
    if (this.touch) return;
    const touch: Touch = {
      id,
      x,
      y,
      swiped: false,
      longPressed: false,
      lastDirection: null,
      followsTap: time - this.lastTapEnd <= this.options.doubleTapMs,
    };
    this.touch = touch;
    this.timer = setTimeout(() => {
      if (this.touch !== touch || touch.swiped) return;
      touch.longPressed = true;
      this.emit({ kind: "long_press" });
    }, this.options.longPressMs);
  }

  move(id: number, x: number, y: number): void {
    const touch = this.touch;
    if (!touch || touch.id !== id) return;
    const dx = x - touch.x;
    const dy = y - touch.y;
    if (Math.max(Math.abs(dx), Math.abs(dy)) < this.options.swipeDistance) return;

    const direction: Direction = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? "right" : "left") : dy > 0 ? "down" : "up";
    touch.swiped = true;
    clearTimeout(this.timer);
    // 指を離さずに続けて曲がれるよう、基準を今の位置に移す
    touch.x = x;
    touch.y = y;
    if (direction !== touch.lastDirection) {
      touch.lastDirection = direction;
      this.emit({ kind: "swipe", direction });
    }
  }

  up(id: number, time: number): void {
    const touch = this.touch;
    if (!touch || touch.id !== id) return;
    this.release();
    if (touch.swiped || touch.longPressed) {
      this.lastTapEnd = -Infinity;
    } else if (touch.followsTap) {
      this.lastTapEnd = -Infinity;
      this.emit({ kind: "double_tap" });
    } else {
      this.lastTapEnd = time;
      this.emit({ kind: "tap" });
    }
  }

  cancel(id: number): void {
    if (this.touch?.id !== id) return;
    this.release();
    this.lastTapEnd = -Infinity;
  }

  /** 押している途中の指を忘れ、長押しのタイマーを止める */
  release(): void {
    clearTimeout(this.timer);
    this.touch = null;
  }
}

/** 要素に付けるポインターイベントのハンドラー。見分けたジェスチャーを onGesture に渡す */
export function useGestures(onGesture: (gesture: Gesture) => void) {
  const handler = useRef(onGesture);
  useEffect(() => {
    handler.current = onGesture;
  }, [onGesture]);
  const recognizer = useMemo(() => new GestureRecognizer((g) => handler.current(g)), []);
  useEffect(() => () => recognizer.release(), [recognizer]);

  return {
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      // 指が要素の外に出ても動きを追えるようにする
      e.currentTarget.setPointerCapture(e.pointerId);
      recognizer.down(e.pointerId, e.clientX, e.clientY, e.timeStamp);
    },
    onPointerMove: (e: PointerEvent<HTMLElement>) => recognizer.move(e.pointerId, e.clientX, e.clientY),
    onPointerUp: (e: PointerEvent<HTMLElement>) => recognizer.up(e.pointerId, e.timeStamp),
    onPointerCancel: (e: PointerEvent<HTMLElement>) => recognizer.cancel(e.pointerId),
    // 長押しでメニューが出ないようにする
    onContextMenu: (e: { preventDefault(): void }) => e.preventDefault(),
  };
}
