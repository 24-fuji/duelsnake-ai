import type { Decision } from "../ai/agent";
import type { Field } from "../game/field";
import type { Action } from "../game/game";
import type { Rules } from "../game/rules";
import type { Palette } from "./draw";

export const ACTION_LABELS: Record<Action, string> = {
  up: "↑ 上",
  down: "↓ 下",
  left: "← 左",
  right: "→ 右",
  boost: "ブースト",
  use_item: "アイテム使用",
};

const HELD_LABELS = { block_clear: "ブロック消去", block_jam: "お邪魔" } as const;

interface Props {
  title: string;
  field: Field;
  rules: Rules;
  palette: Palette;
  /** AI の直近の判断。人間なら undefined */
  decision?: Decision | null;
  actions?: readonly Action[];
  /** 所持アイテムと飛来中のお邪魔の欄を出すか。所持アイテムが出ない一人モードでは出さない */
  heldItems?: boolean;
}

export function PlayerPanel({ title, field, rules, palette, decision, actions, heldItems = true }: Props) {
  const snake = field.snake;
  const seconds = (ticks: number) => (ticks * rules.tickSeconds).toFixed(1);
  const boost =
    snake.boostRemaining > 0
      ? `発動中 あと ${seconds(snake.boostRemaining)} 秒`
      : snake.boostCooldown > 0
        ? `待機 あと ${seconds(snake.boostCooldown)} 秒`
        : "使用可";

  return (
    <div className="panel">
      <h2>
        <span className="swatch" style={{ background: palette.head }} />
        {title}
        {!snake.alive && <span className="dead">衝突</span>}
      </h2>
      <dl>
        <dt>スコア</dt>
        <dd className="score">{snake.score}</dd>
        <dt>長さ</dt>
        <dd>{snake.length}</dd>
        {heldItems && (
          <>
            <dt>所持アイテム</dt>
            <dd>{snake.heldItem ? HELD_LABELS[snake.heldItem] : "なし"}</dd>
          </>
        )}
        <dt>ブースト</dt>
        <dd>{boost}</dd>
        {heldItems && (
          <>
            <dt>飛来中のお邪魔</dt>
            <dd>{field.incomingJams.length}</dd>
          </>
        )}
      </dl>
      {decision !== undefined && actions && <QValues decision={decision} actions={actions} />}
    </div>
  );
}

function QValues({ decision, actions }: { decision: Decision | null; actions: readonly Action[] }) {
  if (!decision) {
    return <p className="qvalues-empty">AI の判断はまだありません</p>;
  }
  const q = Array.from(decision.qValues);
  const min = Math.min(...q);
  const range = Math.max(...q) - min || 1;
  return (
    <div className="qvalues">
      <p>
        直近の判断: <strong>{ACTION_LABELS[decision.action]}</strong>
        {decision.random && <span className="random"> (ランダム行動)</span>}
      </p>
      <table>
        <tbody>
          {actions.map((action, i) => (
            <tr key={action} className={action === decision.action ? "chosen" : undefined}>
              <td>{ACTION_LABELS[action]}</td>
              <td className="bar-cell">
                <div className="bar" style={{ width: `${8 + (92 * (q[i] - min)) / range}%` }} />
              </td>
              <td className="num">{q[i].toFixed(3)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
