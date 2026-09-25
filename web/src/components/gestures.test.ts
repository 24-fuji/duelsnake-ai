import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { GestureRecognizer, type Gesture } from "./gestures";

let gestures: Gesture[];
let recognizer: GestureRecognizer;

beforeEach(() => {
  vi.useFakeTimers();
  gestures = [];
  recognizer = new GestureRecognizer((g) => gestures.push(g));
});

afterEach(() => {
  vi.useRealTimers();
});

/** 時刻 start から duration ms だけ動かさずに押して離す */
function tap(start: number, duration = 80): void {
  recognizer.down(1, 100, 100, start);
  vi.advanceTimersByTime(duration);
  recognizer.up(1, start + duration);
}

describe("タッチ操作", () => {
  it("スワイプした向きを伝え、指を離さずに曲がった向きも伝える", () => {
    recognizer.down(1, 0, 0, 0);
    recognizer.move(1, 10, 2);
    expect(gestures).toEqual([]);
    recognizer.move(1, 30, 4);
    recognizer.move(1, 60, 6); // 同じ向きのままなら繰り返さない
    recognizer.move(1, 62, -30);
    recognizer.up(1, 200);
    expect(gestures).toEqual([
      { kind: "swipe", direction: "right" },
      { kind: "swipe", direction: "up" },
    ]);
  });

  it("動かさずに押し続けると長押しになり、離してもタップにはならない", () => {
    recognizer.down(1, 0, 0, 0);
    vi.advanceTimersByTime(349);
    expect(gestures).toEqual([]);
    vi.advanceTimersByTime(1);
    recognizer.up(1, 500);
    expect(gestures).toEqual([{ kind: "long_press" }]);
  });

  it("長押しのあとにスワイプもできる", () => {
    recognizer.down(1, 0, 0, 0);
    vi.advanceTimersByTime(400);
    recognizer.move(1, 0, 30);
    expect(gestures).toEqual([{ kind: "long_press" }, { kind: "swipe", direction: "down" }]);
  });

  it("スワイプしたら長押しにならない", () => {
    recognizer.down(1, 0, 0, 0);
    recognizer.move(1, -30, 0);
    vi.advanceTimersByTime(1000);
    expect(gestures).toEqual([{ kind: "swipe", direction: "left" }]);
  });

  it("すぐに続けて2回タップするとダブルタップになる", () => {
    tap(0);
    tap(300);
    tap(1000);
    tap(1500);
    expect(gestures).toEqual([{ kind: "tap" }, { kind: "double_tap" }, { kind: "tap" }, { kind: "tap" }]);
  });

  it("少しの指のぶれはタップとみなす", () => {
    recognizer.down(1, 0, 0, 0);
    recognizer.move(1, 8, -6);
    recognizer.up(1, 100);
    expect(gestures).toEqual([{ kind: "tap" }]);
  });

  it("2本目の指は無視する", () => {
    recognizer.down(1, 0, 0, 0);
    recognizer.down(2, 50, 50, 10);
    recognizer.move(2, 100, 50);
    recognizer.up(2, 50);
    recognizer.move(1, 0, 30);
    recognizer.up(1, 100);
    expect(gestures).toEqual([{ kind: "swipe", direction: "down" }]);
  });
});
