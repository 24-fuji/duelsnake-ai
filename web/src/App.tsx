import { useCallback, useEffect, useRef, useState } from "react";
import { assertModelCompatible } from "./ai/agent";
import { BUNDLED_MODELS } from "./ai/catalog";
import { SnakeModel } from "./ai/model";
import { itemImageUrl } from "./components/draw";
import { GameScreen, type Layout, type Mode } from "./components/GameScreen";
import { RulesScreen } from "./components/RulesScreen";
import { formatTimeLimit, SoloScreen } from "./components/SoloScreen";
import {
  SOLO_DEFAULT_TIME_LIMIT,
  SOLO_POISON_GROW,
  SOLO_POISON_MULTIPLIER,
  SOLO_TIME_LIMITS,
} from "./game/solo";

/** 最初に選ぶ盤面。そのモデルが無ければ一覧の先頭を選ぶ */
const DEFAULT_BOARD = "16x16";
const DIFFICULTY_LABELS: Record<string, string> = { easy: "かんたん", medium: "ふつう", hard: "むずかしい" };
const SPEEDS = [1, 2, 4];
/** 対戦のモードと、AI のいない一人モード */
type PlayMode = Mode | "solo";
/** 表示の切り替えを覚えておく localStorage のキー */
const LAYOUT_KEY = "duelsnake.layout";
/** ルール画面を開いたときに積む履歴の印 (history.state に入れる) */
const RULES_STATE = { duelsnakeRules: true } as const;

function isRulesState(state: unknown): boolean {
  return typeof state === "object" && state !== null && "duelsnakeRules" in state;
}

/** 前に選んだ表示があればそれを、無ければ指で操作する端末かどうかで決める */
function initialLayout(): Layout {
  try {
    const saved = localStorage.getItem(LAYOUT_KEY);
    if (saved === "pc" || saved === "mobile") return saved;
  } catch {
    // 保存できない環境では毎回判定する
  }
  return window.matchMedia("(pointer: coarse)").matches ? "mobile" : "pc";
}

interface LoadedModel {
  model: SnakeModel;
  /** 読み込むたびに変わる値。ゲーム画面を作り直すのに使う */
  id: number;
}

let nextModelId = 1;

function loaded(model: SnakeModel): LoadedModel {
  assertModelCompatible(model);
  return { model, id: nextModelId++ };
}

export function App() {
  const [current, setCurrent] = useState<LoadedModel | null>(null);
  /** 選んでいる同梱モデルの盤面 ("16x16" など)。同梱モデルが無ければ null */
  const [board, setBoard] = useState<string | null>(
    () => (BUNDLED_MODELS.find((m) => m.key === DEFAULT_BOARD) ?? BUNDLED_MODELS[0])?.key ?? null,
  );
  const [error, setError] = useState<string | null>(
    BUNDLED_MODELS.length === 0 ? "model/recent-model/ に学習済みモデルがありません" : null,
  );
  const [mode, setMode] = useState<PlayMode>("human_vs_ai");
  const solo = mode === "solo";
  /** 一人モードの制限時間 (秒)。0 は制限なし */
  const [soloTimeLimit, setSoloTimeLimit] = useState(SOLO_DEFAULT_TIME_LIMIT);
  const [difficulty, setDifficulty] = useState("hard");
  const [speed, setSpeed] = useState(1);
  const [autoRestart, setAutoRestart] = useState(true);
  const [layout, setLayout] = useState(initialLayout);
  /** ルール画面を開いているか。開くのはボタンを押したときだけ */
  const [showRules, setShowRules] = useState(false);
  /** ルール画面を開く前のスクロール位置。戻ったときに元の位置に戻す */
  const scrollRef = useRef(0);

  const changeLayout = (next: Layout) => {
    setLayout(next);
    try {
      localStorage.setItem(LAYOUT_KEY, next);
    } catch {
      // 保存できなくても、この画面では切り替わる
    }
  };

  // ルール画面は履歴を1つ積んで開くので、ブラウザの戻る (iPhone の横スワイプなど) でも閉じられる。
  // 逆に、進む操作でルール画面の履歴に入ったときは開かずに戻す。プレー中のスワイプでルール画面が出ないようにするため
  useEffect(() => {
    // ルール画面のまま再読み込みしたときは、ゲーム画面で始める
    if (isRulesState(history.state)) history.replaceState(null, "");
    const onPopState = (e: PopStateEvent) => {
      if (isRulesState(e.state)) {
        history.back();
        return;
      }
      setShowRules(false);
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  useEffect(() => {
    if (showRules) window.scrollTo(0, 0);
    else window.scrollTo(0, scrollRef.current);
  }, [showRules]);

  const openRules = () => {
    // ボタンにフォーカスが残ると、戻ったあとの Enter やスペースでボタンが押されてしまうので外す
    (document.activeElement as HTMLElement | null)?.blur?.();
    scrollRef.current = window.scrollY;
    history.pushState(RULES_STATE, "");
    setShowRules(true);
  };

  const closeRules = useCallback(() => {
    // 開いたときに積んだ履歴を戻る。popstate で閉じる
    if (isRulesState(history.state)) history.back();
    else setShowRules(false);
  }, []);

  useEffect(() => {
    const entry = BUNDLED_MODELS.find((m) => m.key === board);
    if (!entry) return;
    // 読み込み中に別の盤面を選んだら、古いほうの結果は捨てる
    let cancelled = false;
    SnakeModel.load(entry.url)
      .then((model) => {
        if (cancelled) return;
        setCurrent(loaded(model));
        setError(null);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(`${entry.path} を読み込めません: ${String(e)}`);
      });
    return () => {
      cancelled = true;
    };
  }, [board]);

  const info = current?.model.info;
  const difficulties = info?.difficulty ?? [];
  const randomActionRate = current?.model.randomActionRate(difficulty) ?? 0;

  return (
    <div className={`app ${layout}`}>
      {showRules && current && <RulesScreen game={current.model.info.game} onBack={closeRules} />}
      {/* ルール画面の間もゲーム画面は消さずに隠し、戻ったら続きから遊べるようにする */}
      <div hidden={showRules}>
        <header>
          <h1>DuelSnake AI</h1>
          <div className="settings">
            <label>
              盤面
              <select value={board ?? ""} onChange={(e) => setBoard(e.target.value)}>
                {BUNDLED_MODELS.map((m) => (
                  <option key={m.key} value={m.key}>
                    {m.width} × {m.height}
                  </option>
                ))}
              </select>
            </label>
            <label>
              モード
              <select value={mode} onChange={(e) => setMode(e.target.value as PlayMode)}>
                <option value="human_vs_ai">あなた vs AI</option>
                <option value="ai_vs_ai">AI vs AI (観戦)</option>
                <option value="solo">一人モード</option>
              </select>
            </label>
            {solo && (
              <label>
                制限時間
                <input
                  type="range"
                  min={0}
                  max={SOLO_TIME_LIMITS.length - 1}
                  step={1}
                  list="solo-time-limits"
                  value={SOLO_TIME_LIMITS.indexOf(soloTimeLimit)}
                  onChange={(e) => setSoloTimeLimit(SOLO_TIME_LIMITS[Number(e.target.value)])}
                  // マウスで動かしたあともフォーカスが残ると、Enter や矢印キーをゲームに渡せないので外す
                  onPointerUp={(e) => e.currentTarget.blur()}
                />
                <datalist id="solo-time-limits">
                  {SOLO_TIME_LIMITS.map((_, i) => (
                    <option key={i} value={i} />
                  ))}
                </datalist>
                <span className="time-limit-value">{formatTimeLimit(soloTimeLimit)}</span>
              </label>
            )}
            <label>
              難易度
              <select value={difficulty} disabled={solo} onChange={(e) => setDifficulty(e.target.value)}>
                {difficulties.map((d) => (
                  <option key={d.name} value={d.name}>
                    {DIFFICULTY_LABELS[d.name] ?? d.name} (ランダム {Math.round(d.random_action_rate * 100)}%)
                  </option>
                ))}
              </select>
            </label>
            <label>
              速度
              <select
                value={mode === "ai_vs_ai" ? speed : 1}
                disabled={mode !== "ai_vs_ai"}
                onChange={(e) => setSpeed(Number(e.target.value))}
              >
                {SPEEDS.map((s) => (
                  <option key={s} value={s}>
                    x{s}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <input
                type="checkbox"
                checked={autoRestart}
                disabled={mode !== "ai_vs_ai"}
                onChange={(e) => setAutoRestart(e.target.checked)}
              />
              自動で次の試合
            </label>
            <label>
              表示
              <select value={layout} onChange={(e) => changeLayout(e.target.value as Layout)}>
                <option value="pc">パソコン版</option>
                <option value="mobile">携帯版 (タッチ操作)</option>
              </select>
            </label>
          </div>
          {info && <p className="model-info">最終更新: {new Date(info.created_at).toLocaleString()}</p>}
          {error && <p className="error">{error}</p>}
        </header>

        {!current ? (
          !error && <p>モデルを読み込み中...</p>
        ) : mode === "solo" ? (
          <SoloScreen
            key={`${current.id}-solo-${soloTimeLimit}`}
            game={current.model.info.game}
            timeLimitSeconds={soloTimeLimit}
            layout={layout}
            active={!showRules}
          />
        ) : (
          <GameScreen
            key={`${current.id}-${mode}-${difficulty}`}
            model={current.model}
            mode={mode}
            randomActionRate={randomActionRate}
            speed={mode === "ai_vs_ai" ? speed : 1}
            autoRestart={autoRestart}
            layout={layout}
            active={!showRules}
          />
        )}

        <section className="help">
          <h2>操作</h2>
          {layout === "mobile" ? (
            <ul>
              <li>移動: 盤面をスワイプ (指を離さずに続けて曲がれる)</li>
              <li>ブースト: 盤面を長押し</li>
              {!solo && <li>アイテム使用: 盤面をダブルタップ</li>}
              <li>開始・もう一度・再開: 盤面をタップ</li>
              {!solo && <li>観戦中: 右上の小さい盤面をタップすると、大きく映す AI を入れ替える</li>}
            </ul>
          ) : (
            <ul>
              <li>移動: 矢印キー / WASD</li>
              <li>ブースト: スペース</li>
              {!solo && <li>アイテム使用: Shift / E</li>}
              <li>開始・もう一度: Enter、一時停止: P / Esc</li>
            </ul>
          )}
          <p>
            アイテム: <img className="item-icon" src={itemImageUrl("normal_apple")} alt="" />
            リンゴ (+1) <img className="item-icon" src={itemImageUrl("gold_apple")} alt="" />
            金のリンゴ (+3) <img className="item-icon" src={itemImageUrl("poison_apple")} alt="" />
            {solo ? (
              <>毒リンゴ (-1, {-SOLO_POISON_GROW} マス縮む)</>
            ) : (
              <>
                毒リンゴ (-1, 縮む) <img className="item-icon" src={itemImageUrl("block_clear")} alt="" />
                ブロック消去 <img className="item-icon" src={itemImageUrl("block_jam")} alt="" />
                お邪魔 (相手にブロックを送る)
              </>
            )}
          </p>
          {solo && (
            <p>
              一人モード: ブロック消去とお邪魔は出ず、毒リンゴが対戦の {SOLO_POISON_MULTIPLIER} 倍出ます。
              盤面をすべて埋めるとクリアです。制限時間は「制限時間」のバーで選べます (左端は制限なし)。
            </p>
          )}
          <p>
            <button onClick={openRules} disabled={!current}>
              ルールを確認する
            </button>
          </p>
        </section>
      </div>
    </div>
  );
}
