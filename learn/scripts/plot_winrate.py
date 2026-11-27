"""学習の経過 (旧モデルとの対戦の勝率) の CSV から PNG のグラフを描く。

使い方: python3 plot_winrate.py <winrate-WxH.csv> <winrate-WxH.png>

CSV は学習 (learn/src/progress.rs) がモデルを保存するたびに 1 行ずつ足す。
グラフの文字は、日本語のフォントが無い環境でも崩れないよう英語にしている。
"""

import csv
import re
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.ticker import FuncFormatter  # noqa: E402

NEW_COLOR = "#2a6fdb"
OLD_COLOR = "#9a9a9a"


def read_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as f:
        rows = [r for r in csv.DictReader(f) if r.get("games")]
    return sorted(rows, key=lambda r: int(r["games"]))


def thousands(x: float, _pos: int = 0) -> str:
    return f"{x / 1000:,.0f}k" if x >= 1000 else f"{x:,.0f}"


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("usage: plot_winrate.py <csv> <png>")
    csv_path, png_path = Path(sys.argv[1]), Path(sys.argv[2])
    rows = read_rows(csv_path)
    if not rows:
        sys.exit(f"{csv_path} に記録がありません")

    games = [int(r["games"]) for r in rows]
    rate = [float(r["win_rate"]) * 100 for r in rows]
    ci = [float(r["win_rate_ci95"]) * 100 for r in rows]
    new_death = [float(r["new_death_rate"]) * 100 for r in rows]
    old_death = [float(r["old_death_rate"]) * 100 for r in rows]
    match = re.search(r"winrate-(\d+x\d+)", csv_path.name)
    size = match.group(1) if match else csv_path.stem
    eval_games = int(rows[-1]["eval_games"])
    old_games = int(rows[-1]["old_model_games"])

    fig, (ax_rate, ax_death) = plt.subplots(
        2, 1, figsize=(9, 6.5), sharex=True, height_ratios=[3, 2]
    )
    fig.suptitle(
        f"DuelSnake {size}: new model vs old model (main, {old_games:,} games trained)",
        fontsize=12,
    )

    ax_rate.axhline(50, color=OLD_COLOR, linestyle="--", linewidth=1)
    ax_rate.fill_between(
        games,
        [r - c for r, c in zip(rate, ci)],
        [r + c for r, c in zip(rate, ci)],
        color=NEW_COLOR,
        alpha=0.18,
        linewidth=0,
        label="95% confidence interval",
    )
    ax_rate.plot(games, rate, color=NEW_COLOR, marker="o", markersize=3, label="win rate of new model")
    ax_rate.annotate(
        f"{rate[-1]:.1f}%",
        (games[-1], rate[-1]),
        textcoords="offset points",
        xytext=(6, 0),
        va="center",
        color=NEW_COLOR,
        fontsize=9,
    )
    low = min(min(r - c for r, c in zip(rate, ci)), 40)
    high = max(max(r + c for r, c in zip(rate, ci)), 60)
    ax_rate.set_ylim(max(0, low - 5), min(100, high + 5))
    ax_rate.set_ylabel(f"win rate (%)\n{eval_games:,} games each, draw = 0.5")
    ax_rate.legend(loc="best", fontsize=9)
    ax_rate.grid(alpha=0.3)

    ax_death.plot(games, new_death, color=NEW_COLOR, marker="o", markersize=3, label="new model")
    ax_death.plot(games, old_death, color=OLD_COLOR, marker="o", markersize=3, label="old model")
    ax_death.set_ylim(0, 100)
    ax_death.set_ylabel("games with a\ncollision (%)")
    ax_death.set_xlabel("games trained (new model)")
    ax_death.legend(loc="best", fontsize=9)
    # 右端の数値のラベルが切れないよう、右に余白を取る
    ax_death.set_xlim(0, games[-1] * 1.06)
    ax_death.grid(alpha=0.3)
    ax_death.xaxis.set_major_formatter(FuncFormatter(thousands))

    # 旧モデル (main のモデル) が途中で変わったところに縦線を引く
    for prev, cur in zip(rows, rows[1:]):
        if prev["old_model_created_at"] != cur["old_model_created_at"]:
            for ax in (ax_rate, ax_death):
                ax.axvline(int(cur["games"]), color="#d08a2a", linestyle=":", linewidth=1)
            ax_rate.annotate(
                "old model changed",
                (int(cur["games"]), ax_rate.get_ylim()[1]),
                textcoords="offset points",
                xytext=(3, -12),
                fontsize=8,
                color="#d08a2a",
            )

    fig.tight_layout()
    png_path.parent.mkdir(parents=True, exist_ok=True)
    tmp = png_path.with_name(png_path.name + ".tmp.png")
    fig.savefig(tmp, dpi=110)
    tmp.replace(png_path)


if __name__ == "__main__":
    main()
