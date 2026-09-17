default:
    @just --list

# --- Rust (learn) 指令 ---
build-learn:
    cd learn && cargo build --release

# 1エピソード（シングル環境）で実行
train-single:
    cd learn && cargo run --release -- --mode single

# 利用可能メモリの半分（または最低1環境分）に律速して並列学習
train-half:
    cd learn && cargo run --release -- --mode half-memory

# CPUフル活用で最速並列学習
train-max:
    cd learn && cargo run --release -- --mode max

check:
    cd learn && cargo check && cargo fmt --check

clean:
    cd learn && cargo clean
