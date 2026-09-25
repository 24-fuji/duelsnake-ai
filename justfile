default:
    @just --list

# --- Rust (learn) 指令 ---
build-learn:
    cd learn && cargo build --release

# 自己対戦で学習する。Ctrl+C で中断するとモデルを保存 (オプションは `just train --help`)
train *ARGS:
    cd learn && cargo run --release -- {{ARGS}}

# model/recent-model/snake-model.json から学習を再開する
train-resume *ARGS:
    cd learn && cargo run --release -- --resume {{ARGS}}

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
