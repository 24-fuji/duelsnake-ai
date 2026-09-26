default:
    @just --list

# --- Rust (learn) 指令 ---
build-learn:
    cd learn && cargo build --release

# 自己対戦で学習する。Ctrl+C で中断するとモデルを保存 (オプションは `just train --help`)
# 通算 10 万試合 (train.commit_interval_games) ごとに最新モデルを "model update" でコミットしてプッシュする
# `just train --commit-on-interrupt` とすると、Ctrl+C で中断したときも保存した最新モデルをコミットしてプッシュする
train *ARGS:
    cd learn && cargo run --release -- --auto-commit {{ARGS}}

# config.yaml の盤面サイズのモデル (model/recent-model/snake-model-<幅>x<高さ>.json) から学習を再開する。無ければ警告を出して新しく学習する
# train と同じく、通算 10 万試合ごとに最新モデルをコミットしてプッシュする。--commit-on-interrupt も使える
train-resume *ARGS:
    cd learn && cargo run --release -- --resume --auto-commit {{ARGS}}

# 学習中の最新ログを表示し続ける (学習とは別の端末で実行する)
train-watch:
    tail -n 20 -f "$(ls -t log/train-*.log | head -n 1)"

# 動作確認用の短い学習。model/ は上書きせず /tmp/duelsnake-smoke に出力する
train-smoke:
    cd learn && cargo run --release -- --games 1000 --envs 16 --out-dir /tmp/duelsnake-smoke

test:
    cd learn && cargo test --release

check:
    cd learn && cargo check && cargo fmt --check

clean:
    cd learn && cargo clean

# --- Web ---

# 依存パッケージをインストールする (初回のみ)
web-install:
    cd web && npm install

# 開発サーバーを http://localhost:3000 で起動する
web-dev:
    cd web && npm run dev

web-build:
    cd web && npm run build

web-test:
    cd web && npm test
