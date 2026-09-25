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
just test                 # ルールとモデル書き出しのテスト
```

学習ログは `log/train-*.csv` に出力されます。

## Web アプリ
Node.js (v18 以降) が必要です。

```sh
just web-install  # 初回のみ
just web-dev      # http://localhost:3000 で起動
just web-test     # ルールと AI のテスト
```

「あなた vs AI」と「AI vs AI (観戦)」のモードがあり、AI の各行動の Q 値を見ながらモデルを確認できます。
画面の「別のモデルを読み込む」から `model/models/` のバックアップなど、他のモデル JSON も試せます。
