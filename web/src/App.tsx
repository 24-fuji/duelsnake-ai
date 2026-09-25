import { useEffect, useState } from "react";
import { assertModelCompatible } from "./ai/agent";
import { BUNDLED_MODELS } from "./ai/catalog";
import { SnakeModel, type ModelFile } from "./ai/model";
import { itemColor } from "./components/draw";
import { GameScreen, type Layout, type Mode } from "./components/GameScreen";

/** 最初に選ぶ盤面。そのモデルが無ければ一覧の先頭を選ぶ */
const DEFAULT_BOARD = "16x16";
const DIFFICULTY_LABELS: Record<string, string> = { easy: "かんたん", medium: "ふつう", hard: "むずかしい" };
const SPEEDS = [1, 2, 4];
/** 表示の切り替えを覚えておく localStorage のキー */
const LAYOUT_KEY = "duelsnake.layout";

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
  name: string;
  /** 読み込むたびに変わる値。ゲーム画面を作り直すのに使う */
  id: number;
}

let nextModelId = 1;

function loaded(model: SnakeModel, name: string): LoadedModel {
  assertModelCompatible(model);
  return { model, name, id: nextModelId++ };
}

export function App() {
  const [current, setCurrent] = useState<LoadedModel | null>(null);
  /** 選んでいる同梱モデルの盤面 ("16x16" など)。ファイルから読み込んだモデルを使っている間は null */
  const [board, setBoard] = useState<string | null>(
    () => (BUNDLED_MODELS.find((m) => m.key === DEFAULT_BOARD) ?? BUNDLED_MODELS[0])?.key ?? null,
  );
  const [error, setError] = useState<string | null>(
    BUNDLED_MODELS.length === 0 ? "model/recent-model/ に学習済みモデルがありません" : null,
  );
  const [mode, setMode] = useState<Mode>("human_vs_ai");
  const [difficulty, setDifficulty] = useState("hard");
  const [speed, setSpeed] = useState(1);
  const [autoRestart, setAutoRestart] = useState(true);
  const [layout, setLayout] = useState(initialLayout);

  const changeLayout = (next: Layout) => {
    setLayout(next);
    try {
      localStorage.setItem(LAYOUT_KEY, next);
    } catch {
      // 保存できなくても、この画面では切り替わる
    }
  };

  useEffect(() => {
    const entry = BUNDLED_MODELS.find((m) => m.key === board);
    if (!entry) return;
    // 読み込み中に別の盤面を選んだら、古いほうの結果は捨てる
    let cancelled = false;
    SnakeModel.load(entry.url)
      .then((model) => {
        if (cancelled) return;
        setCurrent(loaded(model, entry.path));
        setError(null);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(`${entry.path} を読み込めません: ${String(e)}`);
      });
    return () => {
      cancelled = true;
    };
  }, [board]);

  const loadFile = async (file: File) => {
    try {
      const model = new SnakeModel(JSON.parse(await file.text()) as ModelFile);
      setCurrent(loaded(model, file.name));
      setBoard(null);
      setError(null);
    } catch (e) {
      setError(`${file.name} を読み込めません: ${String(e)}`);
    }
  };

  const info = current?.model.info;
  const difficulties = info?.difficulty ?? [];
  const randomActionRate = current?.model.randomActionRate(difficulty) ?? 0;

  return (
    <div className={`app ${layout}`}>
      <header>
        <h1>DuelSnake AI</h1>
        <div className="settings">
          <label>
            盤面
            <select value={board ?? ""} onChange={(e) => setBoard(e.target.value)}>
              {board === null && (
                <option value="" disabled>
                  {current ? `${boardLabel(current.model)} (読み込んだファイル)` : "なし"}
                </option>
              )}
              {BUNDLED_MODELS.map((m) => (
                <option key={m.key} value={m.key}>
                  {m.width} × {m.height}
                </option>
              ))}
            </select>
          </label>
          <label>
            モード
            <select value={mode} onChange={(e) => setMode(e.target.value as Mode)}>
              <option value="human_vs_ai">あなた vs AI</option>
              <option value="ai_vs_ai">AI vs AI (観戦)</option>
            </select>
          </label>
          <label>
            難易度
            <select value={difficulty} onChange={(e) => setDifficulty(e.target.value)}>
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
          <label className="file">
            別のモデルを読み込む
            <input
              type="file"
              accept=".json,application/json"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) void loadFile(file);
                e.target.value = "";
              }}
            />
          </label>
        </div>
        {current && info && (
          <p className="model-info">
            モデル: {current.name} / 盤面 {boardLabel(current.model)} / 学習 {info.training.games.toLocaleString()} 試合 /
            作成 {new Date(info.created_at).toLocaleString()}
          </p>
        )}
        {error && <p className="error">{error}</p>}
      </header>

      {current ? (
        <GameScreen
          key={`${current.id}-${mode}-${difficulty}`}
          model={current.model}
          mode={mode}
          randomActionRate={randomActionRate}
          speed={mode === "ai_vs_ai" ? speed : 1}
          autoRestart={autoRestart}
          layout={layout}
        />
      ) : (
        !error && <p>モデルを読み込み中...</p>
      )}

      <section className="help">
        <h2>操作</h2>
        {layout === "mobile" ? (
          <ul>
            <li>移動: 盤面をスワイプ (指を離さずに続けて曲がれる)</li>
            <li>ブースト: 盤面を長押し</li>
            <li>アイテム使用: 盤面をダブルタップ</li>
            <li>開始・もう一度・再開: 盤面をタップ</li>
            <li>観戦中: 右上の小さい盤面をタップすると、大きく映す AI を入れ替える</li>
          </ul>
        ) : (
          <ul>
            <li>移動: 矢印キー / WASD</li>
            <li>ブースト: スペース</li>
            <li>アイテム使用: Shift / E</li>
            <li>開始・もう一度: Enter、一時停止: P / Esc</li>
          </ul>
        )}
        <p>
          アイテム: <span className="dot" style={{ background: itemColor("normal_apple") }} />
          リンゴ (+1) <span className="dot" style={{ background: itemColor("gold_apple") }} />
          金のリンゴ (+3) <span className="dot" style={{ background: itemColor("poison_apple") }} />
          毒リンゴ (-1, 縮む) <b>C</b> ブロック消去 <b>J</b> お邪魔 (相手にブロックを送る)
        </p>
        <p>
          ルールの詳細は <code>learn/RULES.md</code> を参照してください。
        </p>
      </section>
    </div>
  );
}

function boardLabel(model: SnakeModel): string {
  const { width, height } = model.info.game.grid;
  return `${width} × ${height}`;
}
