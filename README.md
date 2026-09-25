# duelsnake-ai
Human vs AI リアルタイム対戦型スネークゲーム。

## ディレクトリ構成
- `learn/`: Rustによる自己対戦強化学習 (Double DQN) とゲームシミュレータ
  - `config.yaml`: ゲームルール・ネットワーク構造・学習パラメータ
  - `RULES.md`: ゲームルールと AI の入出力仕様 (Web 版もこれに従う)
- `model/`: 学習済みモデル (JSON)
  - `recent-model/snake-model.json`: 最新モデル。Web アプリが読み込む
  - `models/`: タイムスタンプ付きのバックアップ (git 管理外)
- `web/`: Vite + React + TypeScriptによるフロントエンドWebアプリ
  - `src/game/`: ゲームエンジン (learn と同じルール)
  - `src/ai/`: モデル JSON の読み込み・推論と、AI の入力の作成 (外部ライブラリ不要)
  - `src/components/`: 画面
  - `src/assets/`: UI 用の画像などを置くフォルダ

## 学習
```sh
just train                # Ctrl+C で中断すると model/ に保存して終了
just train --games 50000  # 指定した試合数で終了
just train-resume         # 最新モデルから再開
just train-watch          # 学習中のログを表示し続ける (別の端末で実行)
just test                 # ルールとモデル書き出しのテスト
```

学習中のコンソールには、最下行に概数の試合数などの状況だけを表示します。
詳しい様子はログファイルに出力されるので、別の端末で `just train-watch` を実行すると追えます。

- `log/train-<日時>.log`: 人が読む用。30 秒ごとの統計 (ε・loss・平均 Q 値・スコア・行動の内訳など) と、保存・中断などの出来事
- `log/train-<日時>.csv`: 同じ統計の表。グラフを描くときに使う

ログの間隔は `learn/config.yaml` の `train.log_interval_seconds` で変えられます。

## Web アプリ
公開版: https://24-fuji.github.io/duelsnake-ai/

main に push すると GitHub Actions ([.github/workflows/deploy-web.yml](.github/workflows/deploy-web.yml)) がテスト・ビルドして GitHub Pages に公開します。

ローカルで動かすには Node.js (v18 以降) が必要です。

```sh
just web-install  # 初回のみ
just web-dev      # http://localhost:3000 で起動
just web-test     # ルールと AI のテスト
```

「あなた vs AI」と「AI vs AI (観戦)」のモードがあり、AI の各行動の Q 値を見ながらモデルを確認できます。
画面の「別のモデルを読み込む」から `model/models/` のバックアップなど、他のモデル JSON も試せます。
