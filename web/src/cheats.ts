/**
 * 動作確認用のチートコード。画面には何も出さず、キーボードで入力した文字が一致したときだけ動く。
 * コマンドを足すときは CHEAT_COMMANDS に1つ書き、必要なら CheatContext に画面が提供する操作を足す
 */

import { useEffect, useRef } from "react";

/** 画面が提供する操作。その画面で使えないものは渡さない (コマンドは何も起こさない) */
export interface CheatContext {
  /** 人間がスコアで AI に勝った状態にする (vs AI の対戦画面) */
  forceWin?: () => void;
  /** AI 同士の観戦モードに切り替える */
  aiVsAi?: () => void;
}

export interface CheatCommand {
  /** 打ち込む文字。大文字は Shift を押しながら、スペースは Shift なしで打つ */
  name: string;
  run: (ctx: CheatContext) => void;
}

/** `WIN FORCE` で作る結果のスコア。意味のある値ではない */
export const CHEAT_WIN_SCORES: readonly [number, number] = [24, 9];

export const CHEAT_COMMANDS: readonly CheatCommand[] = [
  { name: "WIN FORCE", run: (ctx) => ctx.forceWin?.() },
  { name: "ai vs ai", run: (ctx) => ctx.aiVsAi?.() },
];

/**
 * 入力済みの文字列に1文字足し、どのコマンドの頭にも合わなくなったら捨てる。
 * 戻り値の `matched` は、ちょうど一致したコマンド名
 */
export function advanceCheatBuffer(
  buffer: string,
  key: string,
  names: readonly string[],
): { buffer: string; matched: string | null } {
  const isPrefix = (text: string) => names.some((n) => n.startsWith(text));
  let next = buffer + key;
  if (!isPrefix(next)) next = isPrefix(key) ? key : "";
  const matched = names.find((n) => n === next) ?? null;
  return { buffer: matched ? "" : next, matched };
}

/** 入力の途切れでバッファを捨てるまでの時間 */
const IDLE_RESET_MS = 3000;

/** キー入力を見張り、チートコードが入力されたら実行する。`enabled` が false の間は何もしない */
export function useCheatCodes(ctx: CheatContext, enabled: boolean): void {
  const ctxRef = useRef(ctx);
  useEffect(() => {
    ctxRef.current = ctx;
  });

  useEffect(() => {
    if (!enabled) return;
    let buffer = "";
    let lastAt = 0;
    const names = CHEAT_COMMANDS.map((c) => c.name);
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;
      if (e.ctrlKey || e.metaKey || e.altKey || e.key.length !== 1) return;
      const now = performance.now();
      if (now - lastAt > IDLE_RESET_MS) buffer = "";
      lastAt = now;
      // スペースは Shift なしで打つ。Shift 付きのスペースは入力の誤りとして捨てる
      const key = e.key === " " && e.shiftKey ? "" : e.key;
      if (key === "") {
        buffer = "";
        return;
      }
      const next = advanceCheatBuffer(buffer, key, names);
      // 入力中のスペースでページがスクロールしないようにする
      if (next.buffer !== "" || next.matched) e.preventDefault();
      buffer = next.buffer;
      if (next.matched) CHEAT_COMMANDS.find((c) => c.name === next.matched)?.run(ctxRef.current);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [enabled]);
}
