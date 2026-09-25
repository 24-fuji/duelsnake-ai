import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AiPlayer, type Decision } from "../ai/agent";
import type { SnakeModel } from "../ai/model";
import { GameEnv, type Action } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import { Board } from "./Board";
import { PLAYER_PALETTES } from "./draw";
import { PlayerPanel } from "./PlayerPanel";

export type Mode = "human_vs_ai" | "ai_vs_ai";
type Status = "ready" | "running" | "paused" | "over";

const KEY_ACTIONS: Record<string, Action> = {
  ArrowUp: "up",
  KeyW: "up",
  ArrowDown: "down",
  KeyS: "down",
  ArrowLeft: "left",
  KeyA: "left",
  ArrowRight: "right",
  KeyD: "right",
  Space: "boost",
  ShiftLeft: "use_item",
  ShiftRight: "use_item",
  KeyE: "use_item",
};

const AUTO_RESTART_DELAY_MS = 1500;

interface Props {
  model: SnakeModel;
  mode: Mode;
  randomActionRate: number;
  /** ゲームの進行速度の倍率 */
  speed: number;
  /** 試合が終わったら自動で次の試合を始める (AI 同士のみ) */
  autoRestart: boolean;
}

export function GameScreen({ model, mode, randomActionRate, speed, autoRestart }: Props) {
  const rules = useMemo(() => rulesFromConfig(model.info.game), [model]);
  const ais = useMemo(
    () =>
      mode === "human_vs_ai"
        ? [null, new AiPlayer(model, randomActionRate)]
        : [new AiPlayer(model, randomActionRate), new AiPlayer(model, randomActionRate)],
    [model, mode, randomActionRate],
  );
  const names = mode === "human_vs_ai" ? ["あなた", "AI"] : ["AI 1", "AI 2"];

  const envRef = useRef<GameEnv | null>(null);
  if (envRef.current === null) envRef.current = new GameEnv(rules);
  const decisionsRef = useRef<(Decision | null)[]>([null, null]);
  const inputsRef = useRef<Action[]>([]);
  const [status, setStatus] = useState<Status>("ready");
  const statusRef = useRef(status);
  const [version, setVersion] = useState(0);
  const [tally, setTally] = useState({ wins: [0, 0], draws: 0 });

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const start = useCallback(() => {
    if (envRef.current?.isOver) {
      envRef.current = new GameEnv(rules);
      decisionsRef.current = [null, null];
    }
    inputsRef.current = [];
    // 設定欄などにフォーカスが残っていると矢印キーやスペースを奪われるので外す
    (document.activeElement as HTMLElement | null)?.blur?.();
    setStatus("running");
    setVersion((v) => v + 1);
  }, [rules]);

  const tick = useCallback(() => {
    const env = envRef.current!;
    const inputs: [Action[], Action[]] = [[], []];
    ais.forEach((ai, player) => {
      if (ai && env.needsDecision(player)) {
        const decision = ai.decide(env, player);
        decisionsRef.current[player] = decision;
        inputs[player].push(decision.action);
      }
    });
    if (!ais[0]) inputs[0] = inputsRef.current.splice(0);
    env.step(inputs);
  }, [ais]);

  // ゲームループ: 経過時間に応じて決まった間隔でティックを進める
  useEffect(() => {
    if (status !== "running") return;
    const tickMs = (rules.tickSeconds * 1000) / speed;
    let last = performance.now();
    let pending = 0;
    let frameId = 0;

    const frame = (now: number) => {
      pending += Math.min(now - last, 250);
      last = now;
      const env = envRef.current!;
      let ticked = false;
      while (pending >= tickMs && !env.isOver) {
        pending -= tickMs;
        tick();
        ticked = true;
      }
      if (ticked) setVersion((v) => v + 1);
      if (env.result) {
        const { winner } = env.result;
        setTally((t) =>
          winner === null
            ? { ...t, draws: t.draws + 1 }
            : { ...t, wins: t.wins.map((n, p) => (p === winner ? n + 1 : n)) },
        );
        setStatus("over");
        return;
      }
      frameId = requestAnimationFrame(frame);
    };
    frameId = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(frameId);
  }, [status, speed, rules, tick]);

  useEffect(() => {
    if (status !== "over" || !autoRestart || mode !== "ai_vs_ai") return;
    const timer = setTimeout(start, AUTO_RESTART_DELAY_MS);
    return () => clearTimeout(timer);
  }, [status, autoRestart, mode, start]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
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
      if (!action || ais[0]) return;
      e.preventDefault();
      if (!e.repeat && current === "running") inputsRef.current.push(action);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [start, ais]);

  const env = envRef.current;
  const remaining = (env.remainingTicks * rules.tickSeconds).toFixed(1);

  return (
    <div className="game">
      <div className="topbar">
        <span className="timer">残り {remaining} 秒</span>
        <span className="tally">
          {names[0]} {tally.wins[0]} 勝 / {names[1]} {tally.wins[1]} 勝 / 引き分け {tally.draws}
        </span>
        {status === "running" ? (
          <button onClick={() => setStatus("paused")}>一時停止 (P)</button>
        ) : (
          <button onClick={start}>{status === "over" ? "もう一度 (Enter)" : "開始 (Enter)"}</button>
        )}
      </div>
      <div className="boards">
        {[0, 1].map((player) => (
          <div className="side" key={player}>
            <Board field={env.fields[player]} rules={rules} palette={PLAYER_PALETTES[player]} version={version} />
            <PlayerPanel
              title={names[player]}
              field={env.fields[player]}
              rules={rules}
              palette={PLAYER_PALETTES[player]}
              decision={ais[player] ? decisionsRef.current[player] : undefined}
              actions={model.info.actions}
            />
          </div>
        ))}
        {status !== "running" && (
          <div className="overlay">
            <p>{overlayMessage(status, env, names)}</p>
          </div>
        )}
      </div>
    </div>
  );
}

function overlayMessage(status: Status, env: GameEnv, names: string[]): string {
  if (status === "ready") return "Enter キーで開始";
  if (status === "paused") return "一時停止中 (P で再開)";
  const result = env.result;
  if (!result) return "";
  const outcome = result.winner === null ? "引き分け" : `${names[result.winner]}の勝ち`;
  let reason: string;
  if (result.reason === "time_up") {
    reason = "時間切れ・スコア判定";
  } else if (env.fields.every((f) => !f.snake.alive)) {
    reason = "両者衝突・スコア判定";
  } else {
    reason = `${names[1 - result.winner!]}が衝突`;
  }
  return `${outcome} (${reason})`;
}
