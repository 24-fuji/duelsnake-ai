import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ItemType } from "../game/field";
import type { Action } from "../game/game";
import { rulesFromConfig, type GameRules } from "../game/rules";
import { SoloEnv, type SoloEndReason } from "../game/solo";
import { Board } from "./Board";
import { itemImageUrl, PLAYER_PALETTES } from "./draw";
import { KEY_ACTIONS, type Layout } from "./GameScreen";
import { useGestures, type Gesture } from "./gestures";
import { PlayerPanel } from "./PlayerPanel";

type Status = "ready" | "running" | "paused" | "over";

const END_MESSAGES: Record<SoloEndReason, string> = {
  filled: "盤面をすべて埋めた",
  death: "衝突",
  time_up: "時間切れ",
};

/** 勝ったとき (盤面を埋めた) と負けたとき (衝突) に大きく出す文字。時間切れは相手がいないので勝敗を付けない */
const RESULT_WORDS: Partial<Record<SoloEndReason, "success" | "wasted">> = {
  filled: "success",
  death: "wasted",
};

/** クリアしたときに数を出すリンゴ */
const APPLE_LABELS: [ItemType, string][] = [
  ["normal_apple", "リンゴ"],
  ["gold_apple", "金のリンゴ"],
  ["poison_apple", "毒リンゴ"],
];

/** 秒数を「1 分 23.4 秒」のように表す (1 分未満は「23.4 秒」) */
function formatSeconds(seconds: number): string {
  const tenths = Math.round(seconds * 10);
  const [minutes, rest] = [Math.floor(tenths / 600), (tenths % 600) / 10];
  return minutes > 0 ? `${minutes} 分 ${rest.toFixed(1)} 秒` : `${rest.toFixed(1)} 秒`;
}

/** 制限時間の選択肢の表示。0 は「なし」、それ以外は「1 分 30 秒」のように表す */
export function formatTimeLimit(seconds: number): string {
  if (seconds === 0) return "なし";
  const [minutes, rest] = [Math.floor(seconds / 60), seconds % 60];
  return [minutes > 0 ? `${minutes} 分` : "", rest > 0 ? `${rest} 秒` : ""].filter(Boolean).join(" ");
}

interface Props {
  /** 対戦のルール (モデル JSON の game)。一人モードのルールに直して使う */
  game: GameRules;
  /** 制限時間 (秒)。0 なら制限なし */
  timeLimitSeconds: number;
  layout: Layout;
  /** false の間 (ルール画面を開いている間) は止めておき、キー入力も受け付けない */
  active: boolean;
}

/** 一人モードの画面。AI の相手はおらず、自分の盤面だけで遊ぶ */
export function SoloScreen({ game, timeLimitSeconds, layout, active }: Props) {
  const rules = useMemo(() => rulesFromConfig(game), [game]);

  const envRef = useRef<SoloEnv | null>(null);
  if (envRef.current === null) envRef.current = new SoloEnv(rules, timeLimitSeconds);
  const inputsRef = useRef<Action[]>([]);
  const [status, setStatus] = useState<Status>("ready");
  const statusRef = useRef(status);
  const [version, setVersion] = useState(0);
  /** この画面で遊んだうちの最高スコア */
  const [best, setBest] = useState(0);

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const activeRef = useRef(active);
  useEffect(() => {
    activeRef.current = active;
    if (!active && statusRef.current === "running") setStatus("paused");
  }, [active]);

  const start = useCallback(() => {
    if (envRef.current?.isOver) envRef.current = new SoloEnv(rules, timeLimitSeconds);
    inputsRef.current = [];
    // 設定欄などにフォーカスが残っていると矢印キーやスペースを奪われるので外す
    (document.activeElement as HTMLElement | null)?.blur?.();
    setStatus("running");
    setVersion((v) => v + 1);
  }, [rules, timeLimitSeconds]);

  // ゲームループ: 経過時間に応じて決まった間隔でティックを進める
  useEffect(() => {
    if (status !== "running") return;
    const env = envRef.current!;
    const tickMs = env.rules.tickSeconds * 1000;
    let last = performance.now();
    let pending = 0;
    let frameId = 0;

    const frame = (now: number) => {
      pending += Math.min(now - last, 250);
      last = now;
      let ticked = false;
      while (pending >= tickMs && !env.isOver) {
        pending -= tickMs;
        env.step(inputsRef.current.splice(0));
        ticked = true;
      }
      if (ticked) setVersion((v) => v + 1);
      if (env.isOver) {
        setBest((b) => Math.max(b, env.field.snake.score));
        setStatus("over");
        return;
      }
      frameId = requestAnimationFrame(frame);
    };
    frameId = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(frameId);
  }, [status]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!activeRef.current) return;
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;
      const current = statusRef.current;
      if (e.code === "Enter") {
        e.preventDefault();
        if (current !== "running") start();
        return;
      }
      if (e.code === "KeyP" || e.code === "Escape") {
        if (current === "running") setStatus("paused");
        else if (current === "paused") setStatus("running");
        return;
      }
      const action = KEY_ACTIONS[e.code];
      // 所持アイテムが出ないので、アイテム使用のキーは受け付けない
      if (!action || action === "use_item") return;
      e.preventDefault();
      if (!e.repeat && current === "running") inputsRef.current.push(action);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [start]);

  // 携帯版: スワイプで向きを変え、長押しでブーストする。止まっているときはタップで開始する
  const onGesture = useCallback(
    (gesture: Gesture) => {
      if (statusRef.current !== "running") {
        if (gesture.kind === "tap") start();
        return;
      }
      if (gesture.kind === "swipe") inputsRef.current.push(gesture.direction);
      else if (gesture.kind === "long_press") inputsRef.current.push("boost");
    },
    [start],
  );
  const gestures = useGestures(onGesture);

  const env = envRef.current;
  const score = env.field.snake.score;
  const remainingTicks = env.remainingTicks;
  const timer =
    remainingTicks === null
      ? `経過 ${formatSeconds(env.elapsedSeconds)} (制限なし)`
      : `残り ${formatSeconds(remainingTicks * env.rules.tickSeconds)}`;
  const board = (
    <Board
      field={env.field}
      rules={env.rules}
      palette={PLAYER_PALETTES[0]}
      version={version}
      fill={layout === "mobile"}
    />
  );
  const panel = (
    <PlayerPanel title="あなた" field={env.field} rules={env.rules} palette={PLAYER_PALETTES[0]} heldItems={false} />
  );
  const resultWord = status === "over" && env.result ? RESULT_WORDS[env.result] : undefined;
  const overlay = status !== "running" && (
    <div className="overlay">
      {resultWord && <p className={`solo-result ${resultWord}`}>{resultWord}</p>}
      <p>{overlayMessage(status, env, layout)}</p>
      {status === "over" && env.result === "filled" && (
        <>
          <p className="solo-detail">クリア時間 {formatSeconds(env.elapsedSeconds)}</p>
          <p className="solo-detail">
            取ったリンゴ
            {APPLE_LABELS.map(([kind, label]) => (
              <span key={kind} className="solo-apple">
                <img className="item-icon" src={itemImageUrl(kind)} alt="" />
                {label} {env.eatenCounts[kind]} 個
              </span>
            ))}
          </p>
        </>
      )}
      {status === "over" && <p className="hint">最高スコア {best}</p>}
      {layout === "mobile" && status === "over" && <p className="hint">タップでもう一度</p>}
    </div>
  );

  if (layout === "mobile") {
    return (
      <div className="game mobile">
        <div className="mobile-top">
          <div className="mobile-status">
            <span className="timer">{timer}</span>
            <span>スコア {score}</span>
            <span className="tally">最高スコア {best}</span>
            {status === "running" ? (
              <button onClick={() => setStatus("paused")}>一時停止</button>
            ) : (
              <button onClick={start}>{status === "over" ? "もう一度" : status === "paused" ? "再開" : "開始"}</button>
            )}
          </div>
        </div>
        <div className="stage" {...gestures}>
          {board}
          {overlay}
        </div>
        {panel}
      </div>
    );
  }

  return (
    <div className="game">
      <div className="topbar">
        <span className="timer">{timer}</span>
        <span className="tally">最高スコア {best}</span>
        {status === "running" ? (
          <button onClick={() => setStatus("paused")}>一時停止 (P)</button>
        ) : (
          <button onClick={start}>{status === "over" ? "もう一度 (Enter)" : "開始 (Enter)"}</button>
        )}
      </div>
      <div className="boards">
        <div className="side">
          {board}
          {panel}
        </div>
        {overlay}
      </div>
    </div>
  );
}

function overlayMessage(status: Status, env: SoloEnv, layout: Layout): string {
  const touch = layout === "mobile";
  if (status === "ready") return touch ? "タップで開始" : "Enter キーで開始";
  if (status === "paused") return touch ? "一時停止中 (タップで再開)" : "一時停止中 (P で再開)";
  if (!env.result) return "";
  return `${END_MESSAGES[env.result]} スコア ${env.field.snake.score}`;
}
