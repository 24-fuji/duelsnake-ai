# duelsnake-ai
Human vs AI リアルタイム対戦型スネークゲーム。

## ディレクトリ構成
- `learn/`: Rustによる自己対戦強化学習 (Double DQN) とゲームシミュレータ
  - `config.yaml`: ゲームルール・ネットワーク構造・学習パラメータ
  - `RULES.md`: ゲームルールと AI の入出力仕様 (Web 版もこれに従う)
  - `RUST_TUTORIAL.md`: learn/ のコードを教材にした Rust 入門 (文法・所有権・メモリ管理・強化学習の実装)
- `model/`: 学習済みモデル (JSON)
  - `recent-model/snake-model-<幅>x<高さ>.json`: 盤面サイズごとの最新モデル。Web アプリはここにある盤面サイズから選べる
  - `models/`: 盤面サイズとタイムスタンプ付きのバックアップ (git 管理外)
- `web/`: Vite + React + TypeScriptによるフロントエンドWebアプリ
  - `src/game/`: ゲームエンジン (learn と同じルール)
  - `src/ai/`: モデル JSON の読み込み・推論と、AI の入力の作成 (外部ライブラリ不要)
  - `src/components/`: 画面
  - `src/assets/`: UI 用の画像などを置くフォルダ

## 学習
```sh
just train                # Ctrl+C で中断すると model/ に保存して終了
just train --games 50000  # 指定した試合数で終了
just train --commit-on-interrupt  # Ctrl+C で中断したら、保存した最新モデルをコミットしてプッシュする
just train-resume         # 同じ盤面サイズの最新モデルから再開 (--commit-on-interrupt も使える)
just train-watch          # 学習中のログを表示し続ける (別の端末で実行)
just test                 # ルールとモデル書き出しのテスト
```

盤面サイズは `learn/config.yaml` の `game.grid` で決まり、モデルは盤面サイズごとに `model/recent-model/snake-model-<幅>x<高さ>.json` に保存します。
同じ盤面サイズのモデルがあれば上書きし、ほかの盤面サイズのモデルはそのまま残ります。
`just train-resume` は config.yaml と同じ盤面サイズのモデルから再開し、そのモデルが無ければ警告を出して新しく学習を始めます。

`just train` と `just train-resume` は、モデルの通算試合数が 10 万の倍数を越えるごとに、最新モデルを保存して `model update` というメッセージで git にコミットし、プッシュします。
`--commit-on-interrupt` を付けると、Ctrl+C で中断して保存したあとにも同じようにコミットしてプッシュします (`--games` で指定した試合数を終えて止まったときはコミットしません)。
コミットに入るのはそのモデルのファイルだけで、ほかにステージしてある変更は含めません。
通算試合数はモデルごとに数え、`just train-resume` では再開元から続けて、`just train` では 0 から数えます。
間隔は `learn/config.yaml` の `train.commit_interval_games` で変えられます。コミットやプッシュに失敗しても学習は止めず、ログに警告を残します。

学習を助けるため、リンゴには頭を引き寄せる引力 (盤面全体に届く)、毒リンゴとお邪魔ブロックには遠ざける斥力 (隣のマスだけに働く) を報酬として与えています。
強さと届く距離は `learn/config.yaml` の `reward.force` で変えられます (詳しくは [learn/RULES.md](learn/RULES.md) の 12 章)。

学習中のコンソールには、最下行に概数の試合数などの状況だけを表示します。
詳しい様子はログファイルに出力されるので、別の端末で `just train-watch` を実行すると追えます。

- `log/train-<日時>.log`: 人が読む用。30 秒ごとの統計 (ε・loss・平均 Q 値・スコア・引力斥力の報酬・行動の内訳など) と、保存・中断などの出来事
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
「一人モード」では AI の相手なしで遊べます。相手がいないのでブロック消去とお邪魔は出ず、毒リンゴが対戦の 3 倍出て、取ると 4 マス縮みます。
制限時間は、制限なしと 30 秒から 5 分まで (30 秒刻み) から選べます。盤面をすべて埋めてクリアすると、クリア時間と取ったリンゴの数を表示します。
一人モードは Web 版だけのルールで、学習には使いません。
盤面サイズは、`model/recent-model/` に学習済みモデルがあるものから選べます。

指で操作する端末では携帯版、それ以外ではパソコン版で表示し、画面の「表示」でいつでも切り替えられます (選んだ表示はブラウザに保存)。

- パソコン版: キーボードで操作し、両方の盤面を並べる
- 携帯版: スワイプで移動、長押しでブースト、ダブルタップでアイテム使用、タップで開始。相手の盤面は右上のワイプに小さく映す。観戦中はワイプをタップすると、大きく映す AI が入れ替わる
画面には、使っているモデルの最終更新日時 (モデルを書き出した日時) だけを表示します。

画面下の「ルールを確認する」ボタンで、遊ぶ人向けのルール画面 ([web/src/components/RulesScreen.tsx](web/src/components/RulesScreen.tsx)) を開けます。learn/RULES.md から学習や実装の話を除き、選んでいる盤面の数値で説明しています。
ルール画面は「戻る」ボタンでも、ブラウザの戻る操作 (iPhone の横スワイプなど) でも閉じられます。プレー中のスワイプで開いてしまわないよう、ブラウザの進む操作では開かず、開けるのはボタンだけです。開いている間、ゲームは一時停止し、戻ると続きから遊べます。
