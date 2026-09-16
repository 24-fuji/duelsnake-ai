# デフォルト（just とだけ打った場合）はヘルプを表示
default:
    @just --list

# ==========================================
# Rust (learn) 関連コマンド
# ==========================================

# Rust側シミュレータのビルド
build-learn:
    cd learn && cargo build --release

# RL学習の開始（引数を渡すことも可能：例 just train 5000）
train episodes="10000":
    cd learn && cargo run --release -- --episodes {{episodes}}

# Rustのコードチェック・フォーマット
check:
    cd learn && cargo check && cargo fmt --check

# ==========================================
# Web (web) 関連コマンド
# ==========================================

# フロントエンドの依存パッケージインストール
web-install:
    cd web && npm install

# Webアプリの開発サーバー起動
web-dev:
    cd web && npm run dev

# Webアプリのビルド
build-web:
    cd web && npm run build

# ==========================================
# 統合・全般コマンド
# ==========================================

# 全体のビルド（Rust & Web）
build-all: build-learn build-web

# 一時出力ファイルやビルド成果物のクリーンアップ
clean:
    cd learn && cargo clean
    cd web && rm -rf dist node_modules