# Rust 入門: duelsnake-ai の学習コードを教材にして

この文書は、`learn/` にある Rust のコード (自己対戦による強化学習と、ゲームのシミュレータ) を教材にして、Rust の書き方・考え方・慣例を学ぶためのものです。
Rust をほとんど書いたことのない人が、読み終えたときに「同じ規模・同じ水準のコードを自分で書ける」ようになることを目標にしています。

このプロジェクトの核は「強化学習」と「メモリ管理」を Rust でやってみることでした。
そこで前半 (1〜11 章) で Rust の文法と考え方を一通り押さえ、後半 (12〜14 章) でメモリ管理と強化学習のコードを詳しく読みます。

## 目次

- [0. この教材の使い方](#0-この教材の使い方)
- [1. 全体の地図](#1-全体の地図)
- [2. Cargo とクレート](#2-cargo-とクレート)
- [3. モジュールと可視性](#3-モジュールと可視性)
- [4. 基本の文法: 変数・型・式](#4-基本の文法-変数型式)
- [5. 構造体と impl](#5-構造体と-impl)
- [6. enum と match](#6-enum-と-match)
- [7. 所有権: Rust の核心](#7-所有権-rust-の核心)
- [8. 借用: 所有権を渡さずに使う](#8-借用-所有権を渡さずに使う)
- [9. エラー処理](#9-エラー処理)
- [10. トレイトと derive](#10-トレイトと-derive)
- [11. クロージャとイテレータ](#11-クロージャとイテレータ)
- [12. メモリ管理: このプロジェクトの核](#12-メモリ管理-このプロジェクトの核)
- [13. 外部クレートの使い方](#13-外部クレートの使い方)
- [14. 強化学習のコードを読む](#14-強化学習のコードを読む)
- [15. テストの書き方](#15-テストの書き方)
- [16. Rust らしい書き方 (慣例)](#16-rust-らしい書き方-慣例)
- [17. よくあるコンパイルエラーと直し方](#17-よくあるコンパイルエラーと直し方)
- [18. ゼロから同じ構成のプロジェクトを作る](#18-ゼロから同じ構成のプロジェクトを作る)
- [19. 練習問題](#19-練習問題)
- [20. 次に読むもの](#20-次に読むもの)
- [付録: 用語集](#付録-用語集)

---

## 0. この教材の使い方

### 0.1 読み方

- コードを引用するときは、直前に「出典:」としてファイルと行番号のリンクを付けています。VS Code や GitHub で開くと、その行に飛べます。
- 行番号は 2026-09-28 時点のものです。コードを直すとずれるので、ずれていたら関数名で検索してください。
- 引用の途中を省いたところは `...` と書いています。
- 文中の「(実際に試した結果)」と書いたエラーメッセージは、このコードを実際に書き換えてコンパイルしたときの出力です。

### 0.2 手を動かしながら読む

Rust は「コンパイラに叱られながら覚える」言語です。読むだけでなく、コードを少し壊してエラーを見るのがいちばんの近道です。

```sh
cd learn
cargo check                # 型と借用の検査だけ (速い)。まずはこれを繰り返す
cargo test --release       # テストを実行する (just test と同じ)
cargo fmt                  # 書式を整える
cargo clippy               # よくない書き方を指摘してもらう
cargo doc --open           # 依存クレートも含めたドキュメントをブラウザで開く
rustc --explain E0502      # エラー番号の詳しい説明を読む
```

壊して試したあとは `git restore src` で元に戻せます。

VS Code を使うなら、拡張機能 rust-analyzer を入れてください。
変数にカーソルを当てると型が表示され、推論された型が薄い文字でコードの中に表示されます (inlay hints)。
Rust は型を書かずに済む場面が多いぶん、「今この変数は何型か」を見られることが理解の大きな助けになります。

### 0.3 TypeScript 版と比べる

`web/src/game/` には、同じルールを TypeScript で書いたゲームがあります (Web で遊ぶため)。
同じ処理を2つの言語で書いているので、TypeScript を読める人は「TS ならこう、Rust ならこう」と比べると理解が早くなります。本文でもいくつか並べて比べます。

| Rust (`learn/src/env/`) | TypeScript (`web/src/`) |
|---|---|
| `snake.rs` | `game/snake.ts` |
| `field.rs` | `game/field.ts` |
| `game.rs` | `game/game.ts` |
| `rules.rs` | `game/rules.ts` |
| `observation.rs` | `ai/observation.ts` |

---

## 1. 全体の地図

### 1.1 ファイル構成

```
learn/
├── Cargo.toml        … パッケージ名と依存クレート (npm の package.json に当たる)
├── Cargo.lock        … 依存の正確なバージョン (package-lock.json に当たる)
├── config.yaml       … ゲームルール・ネットワーク・学習の設定
└── src/
    ├── main.rs       … 入口。コマンドライン引数を読み、Trainer を動かす
    ├── config.rs     … config.yaml を構造体に読み込み、値を検査する
    ├── env/          … ゲームそのもの (強化学習でいう「環境」)
    │   ├── mod.rs          … env モジュールの目次
    │   ├── rules.rs        … 秒をティックに直したルールの値
    │   ├── snake.rs        … 座標・向き・ヘビ
    │   ├── field.rs        … 1人分の盤面 (ヘビ・アイテム・ブロック)
    │   ├── game.rs         … 2人対戦の1試合。行動を受け取って1ティック進める
    │   ├── observation.rs  … AI に渡す入力 (観測) を作る
    │   └── tests.rs        … ルールのテスト
    ├── model.rs      … ニューラルネットワーク (Q ネットワーク)
    ├── replay.rs     … 経験を貯めておくリプレイバッファ
    ├── force.rs      … 引力・斥力による報酬の補助
    ├── trainer.rs    … 学習ループ (自己対戦の Double DQN)
    ├── export.rs     … モデルを JSON で読み書きする
    ├── monitor.rs    … ログとコンソールの表示
    └── git.rs        … モデルを git にコミットしてプッシュする
```

### 1.2 データの流れ

```
main()
 ├─ Config::load()          config.yaml → Config 構造体
 ├─ Trainer::new()          ネットワーク・リプレイバッファ・64 個の対戦 (Slot) を用意する
 └─ Trainer::run()          Ctrl+C か、指定した試合数まで繰り返す
     ├─ tick()              全部の対戦を1ティック進める
     │   ├─ observation::encode()   行動を決めるプレイヤーの観測を作る
     │   ├─ select_actions()        ε-greedy で行動を決める (まとめて推論する)
     │   ├─ GameEnv::step()         ゲームを1ティック進める
     │   └─ NStepBuilder::push()    経験をリプレイバッファに入れる
     ├─ train_if_due()      経験が溜まったらネットワークを更新する
     │   └─ train_step()    Double DQN の1回分の更新
     └─ save()              ModelFile として JSON に書き出す
```

この教材では、下の層 (ゲームのルール) から上の層 (学習ループ) に向かって、Rust の文法を1つずつ拾っていきます。

---

## 2. Cargo とクレート

### 2.1 Cargo.toml

Rust では、ひとまとまりのプログラムやライブラリを「クレート (crate)」と呼び、Cargo というツールで管理します。
Cargo はビルド・テスト・依存の管理をまとめて受け持つ、npm と pip を合わせたような道具です。

出典: [Cargo.toml](Cargo.toml)
```toml
[package]
name = "duelsnake-learn"
version = "0.2.0"
edition = "2021"

[dependencies]
chrono = "0.4"
clap = { version = "4", features = ["derive"] }
ctrlc = "3.4"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0", features = ["raw_value"] }
serde_yaml = "0.9"
tch = { version = "0.14", features = ["download-libtorch"] } # PyTorch (libtorch) Rustバインディング
```

- `edition` は言語の版です。2015 / 2018 / 2021 / 2024 があり、版によって少しずつ文法が違います。
- `[dependencies]` に使うクレートを書きます。`"0.8"` は「0.8 系の中で最新のもの」という意味です。実際に使うバージョンは `Cargo.lock` に記録されます。
- `features` は、クレートの追加機能を有効にするスイッチです。たとえば `serde` の `derive` を有効にすると、`#[derive(Serialize, Deserialize)]` が使えるようになります (13.1)。
- 依存は `cargo add rand` のようにコマンドでも追加できます。

| クレート | 使いみち | 主な使用箇所 |
|---|---|---|
| chrono | 日時 | ログの時刻、モデルの `created_at` |
| clap | コマンドライン引数の解析 | main.rs の `Cli` |
| ctrlc | Ctrl+C の検知 | main.rs |
| rand | 乱数 | アイテムの出現位置、ε-greedy |
| serde / serde_json / serde_yaml | 構造体と JSON・YAML の相互変換 | config.rs、export.rs |
| tch | PyTorch (libtorch) を Rust から使う | model.rs、trainer.rs |

### 2.2 debug ビルドと release ビルド

```sh
cargo run                              # debug ビルドで実行する
cargo run --release                    # release ビルドで実行する (just train はこちら)
cargo run --release -- --games 1000    # -- の後ろは、自分のプログラムへの引数
```

| | debug | release |
|---|---|---|
| コンパイル | 速い | 遅い (最適化するため) |
| 実行 | 遅い (数倍〜数十倍) | 速い |
| 整数のオーバーフロー | panic して止まる | 黙って折り返す |
| `debug_assert!` | 検査する | 消える |

学習は release で、書きながらの確認は `cargo check` (実行ファイルを作らないので最も速い) で行うのが基本です。
整数のオーバーフローの違いは 12.8 で扱います。

### 2.3 コンパイル時に値を埋め込む `env!`

出典: [src/main.rs:24](src/main.rs#L24)
```rust
const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");
```

`env!` は、コンパイルするときに環境変数を読み、その値を文字列としてプログラムに埋め込みます。
`CARGO_MANIFEST_DIR` は Cargo が設定する「Cargo.toml のあるディレクトリ」なので、どのディレクトリから実行しても `learn/config.yaml` を見つけられます。

名前の後ろに `!` が付くものは「マクロ」です (`println!`、`format!`、`vec!`、`assert_eq!` など)。
関数とは違い、引数の数が自由だったり、コンパイル時に処理されたりします。最初は「`!` の付いた特別な関数」と思っておけば十分です。

---

## 3. モジュールと可視性

### 3.1 `mod` でファイルをつなぐ

Rust では、ファイルを置いただけではコンパイルされません。`mod` で宣言して初めてモジュールになります。

出典: [src/main.rs:1-9](src/main.rs#L1-L9)
```rust
mod config;
mod env;
mod export;
mod force;
mod git;
mod model;
mod monitor;
mod replay;
mod trainer;
```

`mod config;` は「`config.rs` (または `config/mod.rs`) を `config` モジュールとして読み込む」という意味です。
`env` はディレクトリなので、`env/mod.rs` がその目次になります。

出典: [src/env/mod.rs:1-8](src/env/mod.rs#L1-L8)
```rust
pub mod field;
pub mod game;
pub mod observation;
pub mod rules;
pub mod snake;

#[cfg(test)]
mod tests;
```

- `pub mod` は、外 (親の main.rs 側) から `env::game` のように使えるようにします。
- `#[cfg(test)]` は「テストのときだけコンパイルする」という属性です。`cargo build` では tests.rs は無視されます。

できあがるモジュールの木はこうなります。

```
crate (main.rs)
├── config
├── env
│   ├── field
│   ├── game
│   ├── observation
│   ├── rules
│   ├── snake
│   └── tests      (テストのときだけ)
├── export
├── force
├── git
├── model
├── monitor
├── replay
└── trainer
```

### 3.2 `use` でパスを短くする

出典: [src/trainer.rs:3-6](src/trainer.rs#L3-L6)
```rust
use crate::config::Config;
use crate::env::game::{Action, EndReason, GameEnv, GameResult, NUM_PLAYERS};
use crate::env::observation::{self, GRID_CHANNELS, VECTOR_FEATURES};
use crate::env::rules::Rules;
```

- `crate::` は、クレートの根 (main.rs) から数えた絶対パスです。
- `super::` は、親モジュールからの相対パスです。env の中では `use super::rules::Rules;` ([src/env/snake.rs:1](src/env/snake.rs#L1)) のように兄弟のモジュールを参照しています。
- `{self, GRID_CHANNELS, ...}` の `self` は、モジュールそのものも取り込む書き方です。これで `observation::encode(...)` と書けるようになります ([src/trainer.rs:355](src/trainer.rs#L355))。
- `std::` は標準ライブラリ、`rand::` や `tch::` は外部クレートです。

### 3.3 `pub` と非公開

Rust では、何も付けなければ非公開 (private) です。`pub` を付けたものだけが外から見えます。
これは構造体のフィールドにも1つずつ適用されます。

出典: [src/env/game.rs:62-69](src/env/game.rs#L62-L69)
```rust
/// 2人対戦1試合分の環境
pub struct GameEnv {
    pub fields: [Field; NUM_PLAYERS],
    pub tick: u32,
    pub result: Option<GameResult>,
    rules: Rules,
    rng: StdRng,
}
```

`rules` と `rng` には `pub` がありません。外からは直接触れず、ルールを読むときは次のメソッドを使います。

出典: [src/env/game.rs:84-86](src/env/game.rs#L84-L86)
```rust
    pub fn rules(&self) -> &Rules {
        &self.rules
    }
```

こうしておくと、「試合の途中で外からルールを書き換える」ような誤りが、そもそも書けなくなります。
一方 `fields` は `pub` なので、テストでは盤面を直接いじって状況を作れます ([src/env/tests.rs:109-112](src/env/tests.rs#L109-L112))。

非公開の項目は、そのモジュールの中と、その子モジュールからは見えます。
ファイルの末尾に書く `mod tests` (15 章) は子モジュールなので、`pub` でない関数もテストできます。

### 3.4 コメントの3種類

```rust
//! モジュール全体の説明。ファイルの先頭に書く       (例: src/trainer.rs の1行目)
/// 直後の項目 (関数・構造体・フィールドなど) の説明  (例: src/env/snake.rs の4行目)
// ふつうのコメント
```

`///` と `//!` はドキュメントコメントです。`cargo doc` で HTML になり、rust-analyzer ではカーソルを当てたときに表示されます。
clap は `///` を `--help` の説明文としても使います (13.2)。

---

## 4. 基本の文法: 変数・型・式

### 4.1 `let` と `mut`

Rust の変数は、何も付けなければ書き換えられません (immutable)。書き換えるなら `mut` を付けます。

出典: [src/main.rs:68-74](src/main.rs#L68-L74)
```rust
    let mut config = Config::load(&config_path)?;
    let size = export::size_label(&config.game.grid);
    let recent_dir = out_dir.join("recent-model");
    let recent_model = recent_dir.join(export::model_file_name(&config.game.grid));
    if let Some(envs) = cli.envs {
        config.train.num_envs = envs.max(1);
    }
```

`config` は後で `num_envs` を書き換えるので `mut` を付け、`size` などは書き換えないので付けていません。
`mut` の無い変数を書き換えようとするとコンパイルエラー (E0594 や E0384) になり、`mut` があるのに書き換えないと警告が出ます。
そのため、「どの変数が途中で変わりうるか」がコードを読むだけで分かります。

TypeScript の `const` / `let` に似ていますが、Rust のほうが厳しく、`let` で作った構造体はフィールドも書き換えられません。
TS の `const obj = { x: 0 }; obj.x = 1;` は通りますが、Rust で `let config = ...; config.train.num_envs = 1;` と書くとエラーです。

### 4.2 シャドーイング

同じ名前でもう一度 `let` すると、前の変数を隠して新しい変数を作ります。これをシャドーイングと呼びます。

出典: [src/trainer.rs:462-463](src/trainer.rs#L462-L463)
```rust
        let best = tch::no_grad(|| self.online.forward(&grid, &vector).argmax(1, false));
        let best = Vec::<i64>::try_from(best.to(Device::Cpu)).expect("行動の取り出しに失敗");
```

1行目の `best` は `Tensor` (テンソル)、2行目の `best` は `Vec<i64>` で、型まで違います。
「同じものを、形を変えて持ち直す」ときに、`best_tensor` と `best_vec` のような名前を考えずに済みます。

### 4.3 数値の型と `as`

Rust は、数値の型を暗黙に変換しません。

| 型 | 意味 | このコードでの主な用途 |
|---|---|---|
| `i32` | 符号付き 32 bit 整数 | 座標 `x`・`y`、スコア |
| `u32` | 符号なし 32 bit 整数 | ティック数、タイマー |
| `u64` | 符号なし 64 bit 整数 | 通算の試合数・判断数 |
| `usize` | 符号なし整数。幅はポインタと同じ (64 bit 環境なら 64 bit) | 配列の添字、長さ |
| `i64` | 符号付き 64 bit 整数 | tch (PyTorch) のテンソルの形 |
| `f32` / `f64` | 浮動小数点数 | 観測・報酬 / 学習率・ε |

座標が `i32` (負になれる型) なのは、盤面の外 (x = -1 など) を表せるようにするためです。
頭の次の位置が盤面の外かどうかを調べる処理 ([src/env/field.rs:77-79](src/env/field.rs#L77-L79)) には、負の値が必要です。

一方で、配列の添字は `usize` でなければなりません。そのため、座標から添字を作るところでは `as usize` で変換しています。

出典: [src/env/observation.rs:56-64](src/env/observation.rs#L56-L64)
```rust
    let (w, h) = (rules.width as usize, rules.height as usize);
    let plane = w * h;
    ...
    let cell = |channel: usize, x: i32, y: i32| channel * plane + y as usize * w + x as usize;
```

`as` を書き忘れて `usize` と `i32` を掛けようとすると、エラーになります。

```
error[E0277]: cannot multiply `usize` by `i32`
```

面倒に見えますが、TS や Python で起きる「負の数や小数が、いつの間にか添字に紛れ込む」誤りを防ぎ、変換する場所をはっきりさせてくれます。

`as` には注意点があります。範囲外の値でもエラーにならず、次のように変換されます (実際に試した結果)。

```rust
300i32 as u8                      // 44         (整数 → 整数: 下位 8 bit だけが残る)
-1i32 as u32                      // 4294967295
(1.7f32 * 255.0).round() as u8    // 255        (小数 → 整数: 範囲の端に丸める)
(-0.3f32 * 255.0).round() as u8   // 0
```

リプレイバッファの量子化 ([src/replay.rs:16](src/replay.rs#L16)) は、この「小数から整数への変換は範囲の端に丸める」性質を使っています (12.3)。
範囲外になりうる整数の変換で失敗を検出したいときは、`u8::try_from(x)` を使います (範囲外なら `Err` を返す)。

### 4.4 式と文: 最後の式が値になる

Rust では、ほとんどのものが「式」で、値を持ちます。
ブロック `{ ... }` の最後に `;` を付けずに書いた式が、そのブロックの値になります。
関数も同じで、`return` を書かなくても、最後の式が戻り値になります。

出典: [src/env/snake.rs:20-22](src/env/snake.rs#L20-L22)
```rust
    pub fn manhattan(self, other: Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
```

行末に `;` が無いことに注目してください。`;` を付けると「値を捨てる文」になってしまいます (実際に試した結果)。

```
error[E0308]: mismatched types
  --> src/env/snake.rs:20:48
   |
20 |     pub fn manhattan(self, other: Position) -> i32 {
   |            ---------                           ^^^ expected `i32`, found `()`
   |            |
   |            implicitly returns `()` as its body has no tail or `return` expression
21 |         (self.x - other.x).abs() + (self.y - other.y).abs();
   |                                                            - help: remove this semicolon to return this value
```

`()` は「値が無い」ことを表す型 (unit 型) です。エラーの最後の行に「このセミコロンを消せ」と直し方まで書いてあります。

`if` も式なので、三項演算子の代わりに使えます。

出典: [src/env/snake.rs:152-156](src/env/snake.rs#L152-L156)
```rust
        self.move_cooldown = if self.boost_remaining > 0 {
            rules.boost_interval_ticks
        } else {
            rules.normal_interval_ticks
        };
```

TS 版では、同じ処理を三項演算子で書いています ([web/src/game/snake.ts:111](../web/src/game/snake.ts#L111))。Rust には三項演算子が無く、`if` 式を使います。
`match` も式です (6 章)。`return` は、途中で関数を抜けたいときにだけ使います (たとえば [src/env/snake.rs:148-151](src/env/snake.rs#L148-L151) の `return false;`)。

### 4.5 タプルと分解

出典: [src/env/snake.rs:12-18](src/env/snake.rs#L12-L18)
```rust
    pub fn step(self, dir: Direction) -> Position {
        let (dx, dy) = dir.delta();
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
```

`delta()` は `(i32, i32)` というタプル (いくつかの値の組) を返し、`let (dx, dy) = ...` で2つの変数に分けて受け取っています。
複数の値を返したいときに、専用の構造体を作らずに済みます。

分解は、いろいろな場所で使えます。

```rust
let [c, h, w] = self.grid_shape;                   // 配列を分解する            src/replay.rs:124
(rules.width, rules.height) = (16, 16);            // 既存の変数にまとめて代入   src/env/tests.rs:14
for (k, &(s, p)) in requests.iter().enumerate() {  // for の中で分解する         src/trainer.rs:354
```

### 4.6 定数 `const`

出典: [src/env/game.rs:7](src/env/game.rs#L7)
```rust
pub const NUM_PLAYERS: usize = 2;
```

`const` はコンパイル時に決まる値で、型を必ず書きます。名前は大文字のスネークケース (SCREAMING_SNAKE_CASE) にするのが決まりです。
`const` は配列の長さにも使えます: `[Field; NUM_PLAYERS]` ([src/env/game.rs:64](src/env/game.rs#L64))。

型に属する定数 (関連定数) も作れます。

出典: [src/env/game.rs:20-29](src/env/game.rs#L20-L29)
```rust
impl Action {
    pub const ALL: [Action; 6] = [
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::Boost,
        Action::UseItem,
    ];
    pub const COUNT: usize = Self::ALL.len();
```

`Action::COUNT` は `ALL` の長さから計算しているので、行動を増やしても数え間違えません。
これを配列の長さに使うと、たとえば行動のラベル `[&str; Action::COUNT]` ([src/monitor.rs:9](src/monitor.rs#L9)) では、行動を増やしたのにラベルを足し忘れたとき、要素の数が合わずにコンパイルエラーになります。

### 4.7 配列・`Vec`・スライス

| 書き方 | 名前 | 長さ | 置き場所 | このコードでの例 |
|---|---|---|---|---|
| `[T; N]` | 配列 | コンパイル時に決まる | その場 (スタックや構造体の中) | `[Field; NUM_PLAYERS]` |
| `Vec<T>` | ベクタ | 実行中に伸び縮みする | ヒープ | `items: Vec<Item>` |
| `&[T]` / `&mut [T]` | スライス | 実行時に決まる | 他の配列や Vec の一部を借りている | `grid: &mut [f32]` |

- 2人対戦のプレイヤー数のように決まっているものは配列、アイテムのように増減するものは `Vec` にします。
- スライスは「配列や `Vec` の一部 (または全部) への参照」です。関数の引数によく使います。`&Vec<T>` より `&[T]` で受け取るほうが、配列でも `Vec` でも渡せるので慣例です ([src/force.rs:91-97](src/force.rs#L91-L97) の `items: &[Item]`)。
- 範囲 `a..b` (a 以上 b 未満) で切り出します: `&self.grid_buf[k * g..(k + 1) * g]` ([src/trainer.rs:368](src/trainer.rs#L368))。範囲外を指定すると panic (その場でプログラムが止まる) します。
- 作り方: `vec![0.0; n]` (0.0 が n 個)、`Vec::new()` (空)、`Vec::with_capacity(n)` (空だが n 個分の領域を確保済み)、`[0; Action::COUNT]` (配列)。

ヒープやスタックについては 12.1 で説明します。

### 4.8 文字列: `String` と `&str`

| 型 | 意味 | 例 |
|---|---|---|
| `String` | 自分で持っている (所有している) 文字列。伸ばせる | `format!(...)` の結果 |
| `&str` | 他の文字列を借りて見ているだけ | 関数の引数 |
| `&'static str` | プログラムが終わるまで有効な文字列 | 文字列リテラル `"up"` |

出典: [src/env/game.rs:35-44](src/env/game.rs#L35-L44)
```rust
    pub fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Down => "down",
            Action::Left => "left",
            Action::Right => "right",
            Action::Boost => "boost",
            Action::UseItem => "use_item",
        }
    }
```

文字列リテラル `"up"` はプログラムの中に埋め込まれていて消えないので、`&'static str` として返せます。呼ぶたびにメモリを確保することもありません。

`String` が必要なときは、`.to_string()` か `format!` で作ります。

出典: [src/export.rs:162](src/export.rs#L162)
```rust
            actions: Action::ALL.iter().map(|a| a.name().to_string()).collect(),
```

`ModelFile` の `actions` は、JSON に書き出したり JSON から読み戻したりする値なので、借り物の `&str` ではなく、自分で持つ `Vec<String>` にしています。

関数の引数は、読むだけなら `&str` にするのが慣例です。`String` を持っていても、`&message` と書けば `&str` として渡せます (自動で変換されます。これを deref coercion と呼びます)。

出典: [src/monitor.rs:69-73](src/monitor.rs#L69-L73)
```rust
    /// 時刻付きで1行書く
    pub fn event(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.text, "{} {message}", now())?;
        self.text.flush()
    }
```

呼ぶ側の `log.event(&message)?;` ([src/trainer.rs:249](src/trainer.rs#L249)) では、`message` は `String` ですが、`&` を付けて渡しています。

### 4.9 `format!` と書式

`format!` は文字列を作り、`println!` は画面に出し、`write!` / `writeln!` はファイルなどに書きます。書式の指定は共通です。

| 書き方 | 意味 | 使用例 |
|---|---|---|
| `{}` | ふつうの表示 (Display) | |
| `{name}` | 変数を直接埋め込む | `format!("  盤面       : {size}")` ([src/main.rs:120](src/main.rs#L120)) |
| `{:?}` / `{name:?}` | デバッグ表示 (Debug) | `{device:?}` ([src/main.rs:122](src/main.rs#L122)) |
| `{:.2}` | 小数点以下 2 桁 | ログの統計 ([src/monitor.rs:82](src/monitor.rs#L82)) |
| `{:+.3}` | 符号を必ず付けて小数点以下 3 桁 | 引力・斥力の報酬 ([src/monitor.rs:82](src/monitor.rs#L82)) |
| `{:02}` | 2 桁になるよう 0 で埋める | 経過時間 ([src/monitor.rs:210](src/monitor.rs#L210)) |

---

## 5. 構造体と impl

### 5.1 構造体を定義する

出典: [src/env/snake.rs:4-9](src/env/snake.rs#L4-L9)
```rust
/// 盤面上の座標。左上が (0, 0) で、x は右、y は下に向かって増える
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

`#[derive(...)]` は「この型に、決まりきった機能を自動で付ける」という指示です (10 章)。
ここでは `{:?}` で表示できる (`Debug`)、値として気軽にコピーできる (`Clone`, `Copy`)、`==` で比べられる (`PartialEq`, `Eq`) ようにしています。

### 5.2 `impl` でメソッドを付ける

Rust では、データ (`struct`) と振る舞い (`impl`) を別々に書きます。

出典: [src/env/snake.rs:83-113](src/env/snake.rs#L83-L113)
```rust
impl Snake {
    /// 頭を `head` に置き、胴体を下向きに伸ばした状態で上向きに生成する
    pub fn new(head: Position, rules: &Rules) -> Self {
        ...
    }

    pub fn head(&self) -> Position {
        self.body[0]
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }
```

最初の引数で、関数の種類が決まります。

| 最初の引数 | 呼び方 | 意味 | 例 |
|---|---|---|---|
| なし | `Snake::new(pos, &rules)` | 関連関数 (他の言語の静的メソッド)。多くはコンストラクタ | `new` |
| `&self` | `snake.head()` | 読むだけ | `head`, `len`, `boost_ready` |
| `&mut self` | `snake.turn(dir)` | 書き換える | `turn`, `advance`, `apply_growth` |
| `self` | `pos.step(dir)` | 値そのものを受け取る (消費する) | `Position::step` |

`self` の種類は、そのまま「このメソッドは状態を変えるか」の宣言になっています。
`head(&self)` のシグネチャを見れば、呼んでもヘビが変わらないことが分かります。
`&self` や `&mut self` の `&` については 8 章で詳しく説明します。

`Self` (大文字で始まる) は「この impl の型」の別名で、`impl Snake` の中では `Snake` と同じ意味です。

Rust にはコンストラクタ専用の構文が無く、`new` という名前の関連関数を作るのが慣例です。
「何から作るか」を名前にすることもあります: `Rules::from_config` ([src/env/rules.rs:34](src/env/rules.rs#L34))、`Config::load` ([src/config.rs:157](src/config.rs#L157))、`TrainLog::create` ([src/monitor.rs:42](src/monitor.rs#L42))。

### 5.3 構造体の値を作る書き方

フィールド名と同じ名前の変数があれば、`フィールド: 変数` を省略できます。

出典: [src/env/field.rs:104](src/env/field.rs#L104)
```rust
                self.items.push(Item { pos, kind });
```

`Item { pos: pos, kind: kind }` と同じ意味です。

`..` を使うと、「残りのフィールドは別の値から持ってくる」と書けます。

出典: [src/model.rs:106-110](src/model.rs#L106-L110)
```rust
                let cfg = nn::ConvConfig {
                    stride: c.stride,
                    padding: PADDING,
                    ..Default::default()
                };
```

`..Default::default()` は「残りのフィールドは既定値で埋める」という意味で、フィールドの多い設定用の構造体でよく使います。
テストでは、「既存の設定から一部だけ変える」使い方もしています ([src/force.rs:235-241](src/force.rs#L235-L241) の `..config()`)。

### 5.4 設計: 読み込む形と使う形を分ける

出典: [src/env/rules.rs:33-44](src/env/rules.rs#L33-L44)
```rust
impl Rules {
    pub fn from_config(cfg: &GameConfig) -> Self {
        let tick = cfg.grid.tick_seconds;
        // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
        let ticks = |sec: f32| (sec / tick).round().max(0.0) as u32;
        let items = &cfg.items;

        Self {
            width: cfg.grid.width,
            height: cfg.grid.height,
            time_limit_ticks: ticks(cfg.grid.time_limit_seconds).max(1),
            initial_length: cfg.snake.initial_length,
```

config.yaml をそのまま読み込んだ `GameConfig` (時間は秒単位) と、ゲーム中に使う `Rules` (時間はティック単位) を、別の型にしています。
読み込むときの形と使うときの形を分け、変換を1か所 (`from_config`) に閉じ込めると、「秒とティックを取り違える」誤りが起きにくくなります。
ゲームの中のコードは `Rules` しか受け取らないので、秒の値を使いようがないからです。

---

## 6. enum と match

### 6.1 enum: いくつかの候補のどれか1つ

出典: [src/env/snake.rs:25-31](src/env/snake.rs#L25-L31)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
```

TS 版では `type Direction = "up" | "down" | "left" | "right";` と、文字列の合併型で書いています ([web/src/game/snake.ts:9](../web/src/game/snake.ts#L9))。
Rust の enum は文字列ではなく小さな整数として扱われる (この型は 1 バイト) ので、比較もコピーも速く、打ち間違いもコンパイル時に見つかります。

### 6.2 `match`: すべての場合を書かせる

出典: [src/env/snake.rs:33-41](src/env/snake.rs#L33-L41)
```rust
impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
```

`match` は、すべての場合を書かないとコンパイルが通りません (網羅性の検査)。これが Rust の大きな武器です。

たとえば、アイテムの種類 `ItemType` ([src/env/field.rs:6-13](src/env/field.rs#L6-L13)) に `Bomb` を足すと、`cargo check` は次の 5 か所をエラーで教えてくれます (実際に試した結果。右の説明は筆者が付けたもの)。

```
error[E0004]: non-exhaustive patterns: `field::ItemType::Bomb` not covered
   --> src/config.rs:99:15            アイテムを取ったときの報酬
   --> src/env/field.rs:186:27        アイテムを取ったときの効果
   --> src/env/observation.rs:44:11   観測のどのチャネルに描くか
   --> src/env/rules.rs:71:15         再出現までの時間
   --> src/force.rs:82:15             引力・斥力
```

「新しいアイテムを足したら、どこを直せばよいか」を、コンパイラが全部挙げてくれるわけです。
このため Rust では、`_ => ...` (その他すべて) をむやみに使わず、1つずつ書くのがよいとされています。
`_` を使うと、新しい場合を足したときに黙ってそこに入ってしまい、直し漏れに気づけません。

### 6.3 パターンのいろいろ

**`|` で複数をまとめる**

出典: [src/env/rules.rs:70-77](src/env/rules.rs#L70-L77)
```rust
    pub fn respawn_ticks(&self, item: ItemType) -> u32 {
        match item {
            ItemType::NormalApple | ItemType::PoisonApple => self.apple_respawn_ticks,
            ItemType::GoldApple | ItemType::BlockClear | ItemType::BlockJam => {
                self.special_respawn_ticks
            }
        }
    }
```

**タプルと配列を同時に調べる: 勝敗の判定**

出典: [src/env/game.rs:144-161](src/env/game.rs#L144-L161)
```rust
    fn judge(&self) -> Option<GameResult> {
        let filled = self.fields.each_ref().map(|f| f.is_filled(&self.rules));
        let alive = self.fields.each_ref().map(|f| f.snake.alive);
        let (winner, reason) = match (filled, alive) {
            // 盤面を埋めたら、相手の生死やスコアに関わらず勝ち
            ([true, false], _) => (Some(0), EndReason::Filled),
            ([false, true], _) => (Some(1), EndReason::Filled),
            ([true, true], _) => (self.leader_by_score(), EndReason::Filled),
            (_, [true, false]) => (Some(0), EndReason::Death),
            (_, [false, true]) => (Some(1), EndReason::Death),
            (_, [false, false]) => (self.leader_by_score(), EndReason::Death),
            _ if self.tick >= self.rules.time_limit_ticks => {
                (self.leader_by_score(), EndReason::TimeUp)
            }
            _ => return None,
        };
        Some(GameResult { winner, reason })
    }
```

読み方:

- `filled` と `alive` は、どちらも `[bool; 2]` (2人分の真偽値) です。`each_ref().map(...)` は「配列の各要素を借りて変換し、同じ長さの配列を作る」メソッドです。
- `(filled, alive)` というタプルを作り、その形に合うかを上から順に調べます。
- `_` は「何でもよい」。`([true, false], _)` は「0番のプレイヤーだけが盤面を埋めた (生死は問わない)」という意味です。
- `_ if 条件` は「ガード」で、形に加えて条件も満たすときだけ選ばれます。
- 最後の `_ => return None` は「まだ決着していない」で、`match` の途中から関数ごと抜けています。
- 上から順に調べるので、「盤面を埋めた」が「死んだ」より優先されます。

TS 版は同じ処理を `if` の列で書いています ([web/src/game/game.ts:97-109](../web/src/game/game.ts#L97-L109))。
Rust の `match` で書くと、条件と結果が表のように並ぶので、漏れや重なりを見つけやすくなります。

**ガードで値を比べる**

出典: [src/trainer.rs:414-418](src/trainer.rs#L414-L418)
```rust
                stream.reward += match result.winner {
                    Some(w) if w == p => reward_cfg.win,
                    Some(_) => reward_cfg.lose,
                    None => reward_cfg.draw,
                };
```

`Some(w)` は「中身があれば、それを `w` という名前で取り出す」パターンです。勝者 `w` が自分 `p` なら勝ちの報酬、ほかの誰かなら負けの報酬、勝者がいなければ引き分けの報酬です。

### 6.4 `Option`: 「無いかもしれない」を型で表す

Rust には null がありません。代わりに `Option<T>` を使います。

```rust
enum Option<T> {
    Some(T),   // 値がある
    None,      // 値が無い
}
```

これも標準ライブラリにある、ただの enum です (`<T>` は「中に入る型は何でもよい」という意味で、10.5 のジェネリクスです)。

出典: [src/env/snake.rs:74](src/env/snake.rs#L74)
```rust
    pub held_item: Option<HeldItem>,
```

`held_item: Option<HeldItem>` は、「アイテムを持っていない」か「ブロック消去かお邪魔のどちらかを持っている」かを表します。
これを `has_item: bool` と `item_kind: HeldItem` の2つのフィールドに分けると、「持っていないのに種類がある」という、ありえない状態が作れてしまいます。
`Option` にすると、ありえない状態を型の上で作れなくなります。これは Rust でよく言われる「不正な状態を表現できないようにする」という考え方です。

勝者 `winner: Option<usize>` ([src/env/game.rs:58](src/env/game.rs#L58)) も同じで、`None` が引き分けを表します。

`Option` の中身を使うときは、必ず `Some` か `None` かを確かめる必要があります。確かめずに中身を触る方法はありません (`unwrap()` は「無ければプログラムを止める」という確かめ方です。9.5)。
これで「null を触って落ちる」誤りがなくなります。

### 6.5 `if let` と `let else`

1つの場合だけ扱いたいときは `if let` を使います。

出典: [src/env/field.rs:176-180](src/env/field.rs#L176-L180)
```rust
        if let Some(idx) = self.items.iter().position(|i| i.pos == next) {
            let item = self.items.remove(idx);
            self.consume(item.kind, rules);
            self.eaten = Some(item.kind);
        }
```

`position` は、条件に合う最初の要素の添字を `Option<usize>` で返します。見つかったときだけ `{ }` の中に入ります。

逆に「形が合わなければ抜ける」ときは `let ... else` を使います。

出典: [src/trainer.rs:410-412](src/trainer.rs#L410-L412)
```rust
            let Some(result) = slot.env.result else {
                continue;
            };
```

試合が終わっていなければ (`None`) 次の対戦へ `continue` し、終わっていれば `result` を取り出して先へ進みます。
`if let` で書くと本体が1段深くなるところを、平らに書けます。`else` の中は、必ず `return` / `continue` / `break` などで抜けなければなりません。

ほかの例: [src/main.rs:222-224](src/main.rs#L222-L224) (フォルダを読めなければ空の一覧を返す)、[src/git.rs:9-11](src/git.rs#L9-L11) (2つの `Option` を同時に取り出す)。

### 6.6 enum を整数として使う

出典: [src/env/snake.rs:52-55](src/env/snake.rs#L52-L55)
```rust
    /// 観測ベクトルの one-hot で使う並び (up, down, left, right)
    pub fn index(self) -> usize {
        self as usize
    }
```

データを持たない enum は、`as usize` で宣言した順の番号 (Up = 0, Down = 1, ...) に変換できます。観測ベクトルの位置として使っています ([src/env/observation.rs:85](src/env/observation.rs#L85))。
逆向き (番号から enum) は `as` ではできないので、`Action::from_index` のように配列から引きます ([src/env/game.rs:31-33](src/env/game.rs#L31-L33))。

### 6.7 比べた結果も enum

出典: [src/env/game.rs:163-170](src/env/game.rs#L163-L170)
```rust
    fn leader_by_score(&self) -> Option<usize> {
        let (a, b) = (self.fields[0].snake.score, self.fields[1].snake.score);
        match a.cmp(&b) {
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => Some(1),
            std::cmp::Ordering::Equal => None,
        }
    }
```

`cmp` は `Ordering` (`Greater` / `Less` / `Equal`) という enum を返します。
`if a > b {...} else if a < b {...} else {...}` と書くより、3通りがそろっていることが一目で分かります。

---

## 7. 所有権: Rust の核心

ここからが Rust 特有の部分です。
Rust にはガベージコレクタ (GC: 使われなくなったメモリを自動で片付ける仕組み) がなく、C 言語のように手で `free` することもしません。
代わりに「所有権」というルールをコンパイラが検査し、メモリを解放する場所をコンパイル時に決めます。

### 7.1 3つのルール

1. どの値にも、所有者 (owner) となる変数がちょうど1つある。
2. 所有者がスコープ (その変数が有効な範囲。ふつうは `{ }` の中) を抜けると、値は破棄 (drop) され、メモリが解放される。
3. 代入したり関数に渡したりすると、所有権は移動 (move) する。移動したあとの元の変数は使えない。

### 7.2 移動 (move) を体験する

main.rs では、設定をトレーナーに渡すときに `clone()` しています。

出典: [src/main.rs:107-108](src/main.rs#L107-L108)
```rust
    let mut trainer = Trainer::new(config.clone(), device, seed, resume)?;
    let grid_len = GRID_CHANNELS * (config.game.grid.width * config.game.grid.height) as usize;
```

`Trainer::new` は `config: Config` を値で受け取ります ([src/trainer.rs:140-145](src/trainer.rs#L140-L145))。つまり、所有権ごと持っていきます。
試しに `.clone()` を消すと、こうなります (実際に試した結果)。

```
error[E0382]: use of moved value: `config.train`
   --> src/main.rs:142:23
    |
 68 |     let mut config = Config::load(&config_path)?;
    |         ---------- move occurs because `config` has type `config::Config`, which does not implement the `Copy` trait
...
107 |     let mut trainer = Trainer::new(config, device, seed, resume)?;
    |                                    ------ value moved here
...
142 |             thousands(config.train.commit_interval_games)
    |                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value used here after move
```

「107 行目で `config` はトレーナーに移動したのに、142 行目でまだ使おうとしている」という意味です。直し方は3通りあります。

- `clone()` で複製を渡す (このコードの選択。設定は小さく、1回しか起きないので、複製のコストは気にならない)
- 関数を `&Config` (借用。8 章) を受け取る形にする
- 使い終わってから渡すように順番を変える

なぜ `Trainer::new` は所有権ごと受け取るのでしょう。トレーナーは設定を自分のフィールド (`config: Config`) として、学習が終わるまで持ち続けるからです。
「長く持ち続けるものは所有権ごと受け取り、その場で読むだけのものは借りる」が目安です。

### 7.3 `Copy` な型: 移動せずにコピーされる

`Position` や `Rules` は、渡したあとも元の変数が使えます。`#[derive(Clone, Copy)]` が付いているからです。

出典: [src/env/rules.rs:11-13](src/env/rules.rs#L11-L13)
```rust
/// ゲーム中に参照するルール値。秒数はすべてティック数に変換済み
#[derive(Debug, Clone, Copy)]
pub struct Rules {
```

`Copy` な型は、代入や引数に渡すときにビット単位で複製され、元の値も残ります。
`Rules` は数値ばかりで 96 バイト (実測) なので、コピーしても安く済みます。
トレーナーは試合ごとに `Slot::new(self.rules, ...)` ([src/trainer.rs:428](src/trainer.rs#L428)) と `Rules` を値で渡していますが、`self.rules` はそのまま残ります。

`Copy` にできるのは、中身が全部 `Copy` な型だけです。`Vec` や `String` を含む型 (`Snake`、`Field`、`Config`) は `Copy` にできません。
これらはヒープのメモリを持っているので、ビット単位で複製すると「同じヒープの領域を2つの所有者が持つ」ことになり、両方が drop したときに二重に解放してしまうからです。

| 型 | Copy か | 理由 |
|---|---|---|
| `i32`、`f32`、`bool`、`usize` | ○ | |
| `Position`、`Direction`、`Action`、`Rules` | ○ | 中身が全部 Copy で、derive している |
| `Option<HeldItem>` | ○ | 中身が Copy なら Option も Copy |
| `&T` (共有参照) | ○ | 読むだけの参照は何個あってもよい |
| `Vec<T>`、`String` | × | ヒープのメモリを持つ |
| `Snake` (`VecDeque` を持つ)、`Field`、`Config` | × | 同上 |
| `&mut T` (可変参照) | × | 書き換えられる参照は1つだけ (8 章) |

### 7.4 `Clone`: はっきり書く複製

`Copy` でない型を複製するには `.clone()` を呼びます。
`clone()` はヒープの中身まで丸ごと複製する (深いコピー) ので、コストがかかることがあります。
Rust では、このコストが `.clone()` という文字としてコードに現れるのが大事な点です。どこで複製しているかを、コードを読むだけで見つけられます。

出典: [src/trainer.rs:389-395](src/trainer.rs#L389-L395)
```rust
            // 引力・斥力の仕事は、動く前の頭・アイテム・お邪魔ブロックの位置から求める
            let before = self.force.is_active().then(|| {
                slot.env
                    .fields
                    .each_ref()
                    .map(|f| (f.snake.head(), f.items.clone(), f.obstacles.clone()))
            });
```

ゲームを1ティック進める前に、アイテムとブロックの位置を `clone()` で取っておいています。
`step` でアイテムが食べられて消えても、動く前の位置で報酬を計算できるようにするためです。
`bool::then(|| ...)` は「true のときだけクロージャを実行して `Some(...)` にする」メソッドなので、引力・斥力を使わない設定なら複製自体をしません。

### 7.5 `Option::take`: 借りている場所から中身を持ち出す

`take()` は「`Option` の中身を取り出し、元の場所には `None` を残す」メソッドです。
`&mut` で借りているだけの場所から、中身を所有権ごと持ち出せます。

出典: [src/env/game.rs:131-140](src/env/game.rs#L131-L140)
```rust
            Action::UseItem => match snake.held_item.take() {
                Some(HeldItem::BlockClear) => self.fields[player].obstacles.clear(),
                Some(HeldItem::BlockJam) => {
                    let opponent = 1 - player;
                    self.fields[opponent]
                        .incoming_jams
                        .push(self.rules.jam_delay_ticks);
                }
                None => {}
            },
```

「持っているアイテムを取り出して使う (手持ちは空になる)」が1行で書けています。
TS 版では「読み出してから null を代入する」の2行です ([web/src/game/game.ts:85-86](../web/src/game/game.ts#L85-L86))。

学習ループでも同じ形が出てきます。

出典: [src/trainer.rs:366-383](src/trainer.rs#L366-L383)
```rust
        for (k, &(s, p)) in requests.iter().enumerate() {
            let obs = Obs::quantize(
                &self.grid_buf[k * g..(k + 1) * g],
                &self.vector_buf[k * v..(k + 1) * v],
            );
            let stream = &mut self.slots[s].streams[p];
            if let Some((prev_obs, prev_action)) = stream.last.take() {
                self.since_update += stream.nstep.push(
                    prev_obs,
                    prev_action,
                    stream.reward,
                    Some(&obs),
                    &mut self.replay,
                );
            }
            stream.reward = 0.0;
            stream.last = Some((obs, actions[k]));
            env_actions[s][p] = Some(Action::from_index(actions[k] as usize));
```

所有権の流れを追ってみましょう。

1. `Obs::quantize(...)` で今回の観測 `obs` を作る。`obs` が所有者になる (`Obs` は中に `Vec` を2つ持つ)。
2. `stream.last.take()` で、前回の観測 `prev_obs` を取り出す。`stream.last` は `None` になる。
3. `prev_obs` を `nstep.push` に値で渡す。所有権は NStepBuilder に移る (中身の複製は起きない)。
4. 今回の観測 `obs` は `Some(&obs)` で借用して渡す (前回の遷移の「次の観測」として中身を読むだけなので、借りれば足りる)。
5. 最後に `obs` そのものを `stream.last` に移動する。次のティックで、手順 2 の `prev_obs` として取り出される。

観測の `Vec` は、作られてからリプレイバッファにコピーされて捨てられるまで、一度も `clone()` されず、所有者だけが移っていきます。

### 7.6 drop: スコープを抜けると解放される

所有者がいなくなると、値は自動で破棄されます。

出典: [src/trainer.rs:426-428](src/trainer.rs#L426-L428)
```rust
            stats.record_game(slot, result);
            self.info.games += 1;
            *slot = Slot::new(self.rules, self.rng.gen(), n_step, gamma);
```

試合が終わった対戦 (`slot`) を、新しい対戦で上書きしています。
上書きされた古い `Slot` (盤面の `Vec` やヘビの `VecDeque` を持つ) は、この代入の瞬間に drop され、メモリが解放されます。
`free` や `delete` を書く必要も、GC を待つ必要もありません。解放し忘れも、二重に解放することも起きません。

drop はメモリ以外にも働きます。

- `File` は drop されるとファイルを閉じる。
- `BufWriter` は drop されると、溜めていた内容を書き出す (monitor.rs では、すぐ見えるように書くたびに `flush()` している)。
- `tch::Tensor` は drop されると、libtorch 側のテンソルへの参照を1つ手放す。最後の参照がなくなるとメモリが解放される (12.6)。

このように「資源の寿命を変数の寿命に結びつける」考え方を RAII と呼びます。

---

## 8. 借用: 所有権を渡さずに使う

### 8.1 `&` と `&mut`

毎回所有権を渡していては不便なので、Rust には「借用 (borrow)」があります。値への参照を作って、所有権を渡さずに使わせる仕組みです。

| 書き方 | 名前 | できること | 同時にいくつ作れるか |
|---|---|---|---|
| `&T` | 共有参照 | 読む | いくつでも |
| `&mut T` | 可変参照 | 読む・書く | 1つだけ (その間は `&T` も作れない) |

この「読むだけなら何人でも、書き換えるなら1人だけ」というルールが、「ある場所が読んでいる最中に、別の場所が書き換える」という誤りを、コンパイル時に防ぎます。

### 8.2 関数の引数で借りる

出典: [src/env/observation.rs:53-54](src/env/observation.rs#L53-L54)
```rust
/// `player` から見た観測を書き込む。`grid` は GRID_CHANNELS * 高さ * 幅、`vector` は VECTOR_FEATURES の長さ
pub fn encode(env: &GameEnv, player: usize, grid: &mut [f32], vector: &mut [f32]) {
```

シグネチャを読むだけで、この関数が何をするかが分かります。

- `env: &GameEnv` … ゲームは読むだけで、変えない。
- `player: usize` … 小さな値なので、コピーで受け取る。
- `grid: &mut [f32]`、`vector: &mut [f32]` … 呼び出し側が用意した領域に書き込む。

呼ぶ側でも、借用であることを `&` / `&mut` で書きます。

出典: [src/trainer.rs:354-361](src/trainer.rs#L354-L361)
```rust
        for (k, &(s, p)) in requests.iter().enumerate() {
            observation::encode(
                &self.slots[s].env,
                p,
                &mut self.grid_buf[k * g..(k + 1) * g],
                &mut self.vector_buf[k * v..(k + 1) * v],
            );
        }
```

呼ぶ側を読んでも、どの引数が書き換えられうるか (`&mut`) が見えます。

### 8.3 メソッドの `self` も借用

`snake.turn(dir)` は、実は `Snake::turn(&mut snake, dir)` の省略です。
メソッドを呼ぶときは、`&` や `&mut` をコンパイラが自動で付けてくれます。

### 8.4 フィールドごとに借りられる (大事)

借用でいちばんつまずくのがここです。構造体の別々のフィールドは、同時に別々に借りられます。

出典: [src/env/game.rs:102-110](src/env/game.rs#L102-L110)
```rust
    pub fn step(&mut self, actions: [Option<Action>; NUM_PLAYERS]) {
        if self.is_over() {
            return;
        }
        self.tick += 1;

        for field in &mut self.fields {
            field.process_pending(&self.rules, &mut self.rng);
        }
```

ループの中で `self.fields` を `&mut` で借りたまま、`self.rules` を `&` で、`self.rng` を `&mut` で借りています。
3つは別々のフィールドなので、コンパイラは「重なっていない」と判断して許します。

ところが、同じことをメソッド経由でやると通りません。トレーナーの `tick` で試してみます。`grid_len()` は `self.rules` しか読まないメソッドです。

出典: [src/trainer.rs:201-203](src/trainer.rs#L201-L203)
```rust
    fn grid_len(&self) -> usize {
        GRID_CHANNELS * self.rules.cell_count()
    }
```

これを、`stream` (`self.slots` の中への `&mut`) を持っている間に呼んでみます。

```rust
            let stream = &mut self.slots[s].streams[p];
            let g = self.grid_len();          // ← この行を足してみる
            if let Some((prev_obs, prev_action)) = stream.last.take() {
```

実際に試した結果:

```
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/trainer.rs:372:21
    |
371 |             let stream = &mut self.slots[s].streams[p];
    |                               ---------- mutable borrow occurs here
372 |             let g = self.grid_len();
    |                     ^^^^ immutable borrow occurs here
373 |             if let Some((prev_obs, prev_action)) = stream.last.take() {
    |                                                    ----------- mutable borrow later used here
```

`self.grid_len()` は `&self`、つまり `self` 全体を借ります。
コンパイラはメソッドの中身を見ずにシグネチャだけで判断するので、「`self.slots` を書き換えている最中に、`self` 全体を読もうとしている」とみなすのです。

実際のコードは、これを避けるために、ループに入る前に値を取っておいています。

出典: [src/trainer.rs:338-339](src/trainer.rs#L338-L339)
```rust
    fn tick(&mut self, stats: &mut WindowStats) {
        let (g, v) = (self.grid_len(), VECTOR_FEATURES);
```

借用のエラーが出たときの定番の直し方は、次のどれかです。

1. 必要な値を先に取り出して、ローカル変数に入れておく (上の例)
2. メソッドではなくフィールドを直接使う (`self.rules.cell_count()` と書けば `self.rules` だけを借りる)
3. 借用の範囲を短くする (使い終わってから次を借りる)
4. 構造体全体ではなく、必要なフィールドだけを引数に取る関数にする。`Field::process_pending(&mut self, rules: &Rules, rng: &mut StdRng)` ([src/env/field.rs:129](src/env/field.rs#L129)) がこの形で、`GameEnv` 全体ではなく `rules` と `rng` だけを受け取っています。だから上の `step` のループで呼べるのです。

### 8.5 添字は区別してもらえない (その先のフィールドは区別される)

フィールドは区別されますが、配列の添字は区別されません。

```rust
        let snake = &mut self.fields[player].snake;
        let other = &mut self.fields[1 - player].snake;   // ← この2行を足してみる
        other.alive = snake.alive;
```

実際に試した結果:

```
error[E0499]: cannot borrow `self.fields[_].snake` as mutable more than once at a time
   --> src/env/game.rs:125:21
    |
124 |         let snake = &mut self.fields[player].snake;
    |                     ------------------------------ first mutable borrow occurs here
125 |         let other = &mut self.fields[1 - player].snake;
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ second mutable borrow occurs here
126 |         other.alive = snake.alive;
    |                       ----------- first borrow later used here
```

人間には `player` と `1 - player` が違うと分かりますが、コンパイラは添字の値までは追いません。
配列の2つの要素を同時に書き換えたいときは、`let [a, b] = &mut self.fields;` と分解するか、`split_at_mut` や `get_disjoint_mut` を使います。

一方で、添字の先のフィールドが違えば、別の場所として扱われます。

出典: [src/env/game.rs:123-132](src/env/game.rs#L123-L132)
```rust
    fn apply_action(&mut self, player: usize, action: Action) {
        let snake = &mut self.fields[player].snake;
        match action {
            Action::Up => snake.turn(Direction::Up),
            Action::Down => snake.turn(Direction::Down),
            Action::Left => snake.turn(Direction::Left),
            Action::Right => snake.turn(Direction::Right),
            Action::Boost => snake.try_boost(&self.rules),
            Action::UseItem => match snake.held_item.take() {
                Some(HeldItem::BlockClear) => self.fields[player].obstacles.clear(),
```

`snake` は `self.fields[player].snake` を `&mut` で借りています。それなのに、最後の行で `self.fields[player].obstacles` を借りられるのは、フィールドが違う (`snake` と `obstacles`) からです。
添字は区別されなくても、その先のフィールドが違えば、コンパイラは重ならないと判断します。
お邪魔を送る腕の `self.fields[opponent].incoming_jams` ([src/env/game.rs:133-138](src/env/game.rs#L133-L138)) も、同じ理由で通ります。
(`match` の後でもう一度 `snake` を使うように書き換えてもコンパイルが通ることを、実際に確かめています。)

### 8.6 借用は「最後に使ったところ」で終わる

出典: [src/env/tests.rs:113-121](src/env/tests.rs#L113-L121)
```rust
    move_once(&mut env, None);
    let snake = &env.fields[0].snake;
    assert_eq!((snake.score, snake.len(), snake.pending_growth), (1, 3, 1));
    assert!(env.fields[0].items.is_empty());
    assert_eq!(env.fields[0].eaten, Some(ItemType::NormalApple));

    // 取ったことが残るのは、取ったティックだけ
    env.step([None, None]);
    assert_eq!(env.fields[0].eaten, None);
```

`snake` は `env` の中を `&` で借りています。`env.step(...)` は `&mut self` のメソッドなので、`snake` が借りている間は呼べないはずです。
それでも通るのは、借用が変数のスコープの終わりではなく、その参照を最後に使ったところで終わるからです (Non-Lexical Lifetimes、NLL と呼ばれる仕組み)。
`snake` を最後に使ったのは 115 行目なので、120 行目では借用はもう終わっていて、`env` を `&mut` で借りられます。

試しに `env.step` のあとで `snake` を使うと、借用が 121 行目まで延びるので、エラーになります (実際に試した結果)。

```
error[E0502]: cannot borrow `env` as mutable because it is also borrowed as immutable
   --> src/env/tests.rs:120:5
    |
114 |     let snake = &env.fields[0].snake;
    |                 -------------------- immutable borrow occurs here
...
120 |     env.step([None, None]);
    |     ^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
121 |     assert_eq!(snake.score, 1);
    |     -------------------------- immutable borrow later used here
```

### 8.7 ループと借用: `iter` / `iter_mut` / `into_iter`

| 書き方 | 要素の型 | 元のコレクション |
|---|---|---|
| `for x in &v` / `v.iter()` | `&T` | 残る (読むだけ) |
| `for x in &mut v` / `v.iter_mut()` | `&mut T` | 残る (中身を書き換えられる) |
| `for x in v` / `v.into_iter()` | `T` | 消費される (要素の所有権が外に出ていく) |

`game.rs` の `for field in &mut self.fields` から `&mut` を消すと、こうなります (実際に試した結果)。

```
error[E0507]: cannot move out of `self.fields` which is behind a mutable reference
   --> src/env/game.rs:108:22
    |
108 |         for field in self.fields {
    |                      ^^^^^^^^^^^
    |                      |
    |                      `self.fields` moved due to this implicit call to `.into_iter()`
    |                      move occurs because `self.fields` has type `[field::Field; 2]`, which does not implement the `Copy` trait
```

`for x in v` は `v` を消費する (`into_iter`) ので、借りているだけの `self.fields` からは取り出せません。エラーの続きには「`&self.fields` にせよ」という help も出ます。

一方で、消費してよいもの (その場限りの値) は、`into_iter` で回します。

出典: [src/env/game.rs:111-115](src/env/game.rs#L111-L115)
```rust
        for (player, action) in actions.into_iter().enumerate() {
            if let Some(action) = action {
                self.apply_action(player, action);
            }
        }
```

`actions` は引数として受け取った配列で、このあとは使わないので、消費して構いません。

### 8.8 `*` で参照の先を触る

参照の先にある値を書き換えるときは、`*` を付けます。

出典: [src/env/field.rs:145-152](src/env/field.rs#L145-L152)
```rust
        let mut due_jams = 0;
        self.incoming_jams.retain_mut(|t| {
            *t = t.saturating_sub(1);
            if *t == 0 {
                due_jams += 1;
            }
            *t > 0
        });
```

`t` は `&mut u32` (配列の要素への参照) です。`*t = ...` で要素そのものを書き換えます。
メソッドを呼ぶとき (`t.saturating_sub(1)`) は、参照を自動でたどってくれるので `*` は要りません。

`*slot = Slot::new(...)` ([src/trainer.rs:428](src/trainer.rs#L428))、`*action = ...` ([src/trainer.rs:440](src/trainer.rs#L440)) も同じ形です。

### 8.9 パターンで `&` を外す

クロージャの引数やパターンに `&` を書くと、参照を外した値として受け取れます。

出典: [src/env/snake.rs:165-173](src/env/snake.rs#L165-L173)
```rust
    pub fn collides_with_self(&self, pos: Position) -> bool {
        let tail_moves = self.pending_growth == 0;
        let checked = if tail_moves {
            self.body.len() - 1
        } else {
            self.body.len()
        };
        self.body.iter().take(checked).any(|&p| p == pos)
    }
```

`iter()` が出す要素は `&Position` ですが、`|&p|` と書くと `p` は `Position` になります (`Position` が Copy なのでできる)。`|p| *p == pos` と書いても同じ意味です。

`Some(&pos) =>` ([src/env/field.rs:103](src/env/field.rs#L103))、`for &units in ...` ([src/model.rs:57](src/model.rs#L57)) も同じ仕組みです。

### 8.10 参照を返す関数とライフタイム

参照を返す関数では、「その参照はどこから借りたものか」が問題になります。

出典: [src/export.rs:29-35](src/export.rs#L29-L35)
```rust
/// `model_file_name` の形のファイル名なら、その盤面サイズの部分を返す
pub fn size_from_file_name(name: &str) -> Option<&str> {
    let size = name.strip_prefix("snake-model-")?.strip_suffix(".json")?;
    let (w, h) = size.split_once('x')?;
    let is_number = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (is_number(w) && is_number(h)).then_some(size)
}
```

戻り値の `&str` は、引数 `name` の一部を指しています (新しい文字列は作っていない)。
省略せずに書くと、こうなります。

```rust
pub fn size_from_file_name<'a>(name: &'a str) -> Option<&'a str>
```

`'a` は「ライフタイム」(参照が有効な期間) の名前で、「戻り値は `name` と同じ期間だけ有効」という意味です。
引数の参照が1つだけなら、コンパイラが自動で補ってくれる (ライフタイムの省略規則) ので、ふだんは書きません。
`&self` を取るメソッドが参照を返す場合も、`self` から借りたものとみなされます ([src/env/game.rs:84-86](src/env/game.rs#L84-L86) の `rules()`)。

参照の引数が2つ以上あると、どちらから借りたか補えず、エラーになります (実際に試した結果)。

```rust
fn pick(name: &str, fallback: &str) -> &str {
    if name.is_empty() { fallback } else { name }
}
```

```
error[E0106]: missing lifetime specifier
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `name` or `fallback`
```

このときは `fn pick<'a>(name: &'a str, fallback: &'a str) -> &'a str` と書きます。
このプロジェクトでは、`'a` のようなライフタイムを自分で書いた場所は1つもありません (`&'static str` を除く)。
初心者のうちは、ライフタイムが必要になったら「借用ではなく、所有する値 (`String` や `Vec`) を返せばよいのでは」と考えるのも手です。

関数の中で作った値への参照は、返せません。

```rust
fn make() -> &String {
    let s = String::from("hi");
    &s     // s は関数を抜けると drop されるので、その参照は返せない
}
```

これもコンパイルエラー (E0106) になり、help に「所有する値を返すほうがよいでしょう」と出ます。
C や C++ でよくある「解放済みのメモリを指すポインタ (ダングリングポインタ)」は、Rust では作れません。

---

## 9. エラー処理

### 9.1 `Result`

失敗するかもしれない処理は、`Result<T, E>` を返します。これも enum です。

```rust
enum Result<T, E> {
    Ok(T),    // 成功。値 T を持つ
    Err(E),   // 失敗。エラー E を持つ
}
```

Rust には例外 (`throw` / `try-catch`) がありません。失敗は戻り値として返し、呼ぶ側は必ずそれを扱います。

### 9.2 `?` 演算子

出典: [src/config.rs:156-164](src/config.rs#L156-L164)
```rust
impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("設定ファイル {} を読めません: {e}", path.display()))?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| format!("設定ファイル {} の形式が不正です: {e}", path.display()))?;
        config.validate()?;
        Ok(config)
    }
```

- 行末の `?` は、「`Ok(v)` なら `v` を取り出して先へ進む。`Err(e)` なら、その場で `Err(e)` を返して関数を抜ける」という意味です。
- `map_err` はエラーの中身を変換します。ここでは「どのファイルで何が起きたか」を足した文章にしています。元のエラーだけでは「No such file or directory」としか出ず、どのファイルか分からないからです。
- 最後の `Ok(config)` で成功を返します。

`?` を使わずに書くと、こうなります。

```rust
let content = match fs::read_to_string(path) {
    Ok(c) => c,
    Err(e) => return Err(format!("設定ファイル {} を読めません: {e}", path.display()).into()),
};
```

`?` は、この決まりきった書き方を1文字にしたものです。

### 9.3 `Box<dyn Error>`: どんなエラーでも入る箱

`Box<dyn std::error::Error>` は、「`Error` トレイトを実装している、何らかのエラー」を入れる箱です (`dyn` は 10.5、`Box` は 10.5 と 12.1)。
ファイルのエラー、YAML のエラー、自分で作った文字列のエラーなど、種類の違うエラーを1つの型で返せます。

`?` は、エラーの型を戻り値の型へ自動で変換します (`From` トレイト)。
`String` から `Box<dyn Error>` への変換も用意されているので、`map_err` で `String` にしたエラーも、`Result<(), String>` を返す `validate()` のエラーも、そのまま `?` で返せます。

自分でエラーを作って返すときは、`.into()` で変換します。

出典: [src/main.rs:206-212](src/main.rs#L206-L212)
```rust
    if model_size != size {
        return Err(format!(
            "再開元モデル {} の盤面 {model_size} が config.yaml の盤面 {size} と異なります",
            path.display()
        )
        .into());
    }
```

`main` 自身も `Result` を返せます ([src/main.rs:61](src/main.rs#L61))。`main` から `Err` が返ると、そのエラーを表示して、終了コード 1 で終わります。

このコードでのエラー型の選び方:

| 戻り値の型 | 使いどころ | 例 |
|---|---|---|
| `Result<T, Box<dyn Error>>` | いろいろな種類のエラーが混ざる。上に返すだけ | `Config::load`、`ModelFile::read`、`main` |
| `Result<T, String>` | 自分で作るエラー文だけ | `Config::validate`、`git::commit_and_push`、`ModelFile::load_into` |
| `io::Result<T>` | 入出力だけ (`Result<T, io::Error>` の別名) | `TrainLog` のメソッド |

もっと大きなプログラムでは、`anyhow` (アプリケーション向け) や `thiserror` (ライブラリ向け。エラーを enum で定義する) というクレートがよく使われます。このプロジェクトの規模なら、標準ライブラリだけで十分です。

### 9.4 検査を並べる

出典: [src/config.rs:166-175](src/config.rs#L166-L175)
```rust
    fn validate(&self) -> Result<(), String> {
        let grid = &self.game.grid;
        let snake = &self.game.snake;
        let train = &self.train;
        let check = |ok: bool, msg: &str| if ok { Ok(()) } else { Err(msg.to_string()) };

        check(
            grid.width >= 5 && grid.height >= 5,
            "grid の幅と高さは 5 以上にしてください",
        )?;
```

`check` は「条件が偽ならエラーにする」クロージャ (11.1) です。`check(...)?;` を並べると、最初に引っかかった検査のエラーで抜けます。
`Result<(), String>` の `()` は「成功しても返す値は無い」という意味です。

設計の話: 設定は読み込んだ直後に1か所でまとめて検査し、それ以降のコードは「値は正しい」と信じて書きます。
たとえば `n_step >= 1` を検査済みなので、NStepBuilder の中では 0 の場合を考えなくて済みます。「外から入ってくるデータは入口で検査する」が原則です。

### 9.5 `panic!`・`unwrap`・`expect`

`panic!` は、プログラムをその場で止めます (回復はしない)。
`unwrap()` は「`Ok` / `Some` なら中身を返し、`Err` / `None` なら panic する」メソッド、`expect("理由")` は panic したときのメッセージを指定できる `unwrap` です。

使ってよい場面:

- 論理的に起こりえない場合 (起きたらバグ)。

出典: [src/replay.rs:178-185](src/replay.rs#L178-L185)
```rust
        self.steps.push_back((obs, action, reward));
        match next {
            Some(next) => {
                if self.steps.len() < self.n {
                    return 0;
                }
                let ret = self.discounted_return();
                let (obs, action, _) = self.steps.pop_front().unwrap();
```

直前で `push_back` しているので、`pop_front()` が `None` になることはありません。

- 回復のしようがない場合: `expect("ターゲットネットワークの同期に失敗")` ([src/trainer.rs:526-528](src/trainer.rs#L526-L528))。
- テスト: 失敗したらテストが落ちればよいので、`unwrap()` を気軽に使います。

使ってはいけない場面: ファイルが無い、ユーザーの入力が正しくない、など普通に起こりうる失敗。これらは `Result` で返します。

### 9.6 あえて止めない

すべてのエラーを上に返せばよいわけではありません。

出典: [src/trainer.rs:303-312](src/trainer.rs#L303-L312)
```rust
    /// 最新モデルを git にコミットしてプッシュし、結果を知らせる文を返す。失敗しても学習は止めない
    fn commit_recent_model(&self, paths: &OutputPaths) -> String {
        match git::commit_and_push(&paths.recent_model, MODEL_COMMIT_MESSAGE) {
            Ok(()) => format!(
                "最新モデルをコミットしてプッシュしました ({} 試合)",
                thousands(self.info.games)
            ),
            Err(e) => format!("警告: 最新モデルをコミット・プッシュできませんでした: {e}"),
        }
    }
```

何時間も続く学習を、ネットワークの一時的な不調で止めたくないので、エラーを警告の文章に変えてログに残すだけにしています。

画面表示の失敗は、わざと無視しています。

出典: [src/monitor.rs:158-163](src/monitor.rs#L158-L163)
```rust
        let _ = if self.interactive {
            write!(out, "\r\x1b[2K{}", self.text)
        } else {
            writeln!(out, "{}", self.text)
        };
        let _ = out.flush();
```

`Result` を使わずに捨てるとコンパイラが警告を出すので、`let _ =` と書いて「分かっていて捨てている」ことを示します。

### 9.7 `Option` と `Result` の便利なメソッド

| メソッド | 意味 | 使用例 |
|---|---|---|
| `?` (Option に) | `None` ならその場で `None` を返す (戻り値が Option の関数で) | [src/export.rs:31-32](src/export.rs#L31-L32) |
| `unwrap_or_else(f)` | 無ければ `f()` の値を使う | [src/main.rs:65](src/main.rs#L65) |
| `map_or(既定値, f)` | あれば `f(中身)`、無ければ既定値 | [src/main.rs:76](src/main.rs#L76) (Result に対して) |
| `is_some()` / `is_none()` | あるか / 無いか | [src/env/game.rs:89](src/env/game.rs#L89) |
| `is_none_or(f)` | 無いか、あって `f(中身)` が真か | [src/trainer.rs:229](src/trainer.rs#L229) |
| `take()` | 中身を取り出して `None` を残す | 7.5 |
| `as_ref()` / `as_mut()` | `&Option<T>` を `Option<&T>` にする | [src/export.rs:132](src/export.rs#L132) |
| `ok()` | `Result` を `Option` にする (エラーを捨てる) | [src/main.rs:227](src/main.rs#L227) |
| `bool::then(f)` / `then_some(v)` | 真なら `Some(...)`、偽なら `None` | [src/trainer.rs:390](src/trainer.rs#L390)、[src/export.rs:34](src/export.rs#L34) |

たとえば `--games` の終了判定はこうです。

出典: [src/trainer.rs:228-230](src/trainer.rs#L228-L230)
```rust
        while running.load(Ordering::SeqCst)
            && max_games.is_none_or(|m| self.info.games - session_start < m)
        {
```

「試合数の上限が無い (`None`)、または上限に達していない」あいだ、学習を続けます。

---

## 10. トレイトと derive

### 10.1 トレイトとは

トレイトは「この型は、こういう操作ができる」という約束で、TypeScript の interface に近いものです。
標準ライブラリの主なトレイト:

| トレイト | できること | derive できるか |
|---|---|---|
| `Debug` | `{:?}` で表示 | ○ |
| `Clone` | `.clone()` で複製 | ○ |
| `Copy` | 代入のときに暗黙にコピー (Clone も必要) | ○ |
| `PartialEq` / `Eq` | `==`、`!=` で比べる | ○ |
| `PartialOrd` / `Ord` | `<`、`cmp` で大小を比べる | ○ |
| `Default` | `T::default()` で既定値を作る | ○ |
| `Display` | `{}` で表示 | × (自分で書く) |
| `From` / `Into` | 型の変換 | × |
| `Iterator` | `for` で回せる。`map` などが使える | × |

### 10.2 derive で自動実装する

出典: [src/export.rs:50-55](src/export.rs#L50-L55)
```rust
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TrainingInfo {
    pub games: u64,
    pub decisions: u64,
    pub updates: u64,
}
```

- `Default` があるので、`TrainingInfo::default()` で全部 0 の値が作れます ([src/trainer.rs:157](src/trainer.rs#L157))。
- `Copy` があるので、`pub fn info(&self) -> TrainingInfo { self.info }` ([src/trainer.rs:197-199](src/trainer.rs#L197-L199)) のように、参照ではなく値を気軽に返せます。
- `Serialize` と `Deserialize` は serde のトレイトで、JSON との変換ができるようになります (13.1)。

`PartialEq` を derive しているおかげで、構造体をまるごと比べることもできます。

出典: [src/main.rs:214-216](src/main.rs#L214-L216)
```rust
    if model.game != config.game {
        notices.push("注意: 再開元モデルと config.yaml のゲームルールが異なります。現在の config.yaml のルールで学習を続けます".to_string());
    }
```

`GameConfig` の中身 (盤面・ヘビ・アイテムの全フィールド) を1つずつ比べるコードを、自分で書かずに済んでいます。

何を derive するかの目安:

- とりあえず `Debug` は付ける (テストが失敗したときや `{:?}` で中身を見られる)。
- 小さくて、数値や enum だけでできている型には `Clone, Copy` を付ける。
- 比べる必要があれば `PartialEq`。浮動小数点数を含まなければ `Eq` も付ける。`f32` には NaN (どの値とも等しくない特別な値) があるので `Eq` にできません。`GameConfig` が `PartialEq` だけなのはこのためです。

### 10.3 トレイトのメソッドは `use` しないと使えない

出典: [src/replay.rs:1-2](src/replay.rs#L1-L2)
```rust
use rand::rngs::StdRng;
use rand::Rng;
```

`rng.gen_range(...)` の `gen_range` は、`StdRng` 自身のメソッドではなく、`Rng` トレイトのメソッドです。
トレイトを `use` していないと、メソッドが見つかりません。`use rand::Rng;` を消すと、こうなります (実際に試した結果)。

```
error[E0599]: no method named `gen_range` found for mutable reference `&mut StdRng` in the current scope
   --> src/replay.rs:112:25
    |
112 |             let i = rng.gen_range(0..self.len);
    |                         ^^^^^^^^^
    |
    = help: items from traits can only be used if the trait is in scope
...
help: trait `Rng` which provides `gen_range` is implemented but not in scope; perhaps you want to import it
    |
  1 + use rand::Rng;
```

「メソッドが無い」と言われたら、まずトレイトの `use` 漏れを疑ってください。このプロジェクトで出てくる例:

| use | 使えるようになるもの | 場所 |
|---|---|---|
| `rand::Rng` | `gen`、`gen_range` | replay.rs、trainer.rs |
| `rand::SeedableRng` | `StdRng::seed_from_u64` | game.rs、trainer.rs |
| `rand::seq::SliceRandom` | スライスの `choose` | field.rs |
| `std::io::Write` | `write!`、`writeln!`、`flush` | monitor.rs |
| `std::io::IsTerminal` | `is_terminal` | monitor.rs |
| `tch::nn::OptimizerConfig` | `nn::Adam::default().build(...)` | trainer.rs |
| `serde::ser::Error as _` | `S::Error::custom` | export.rs |
| `clap::Parser` | `Cli::parse()` | main.rs |

`use serde::ser::Error as _;` の `as _` は、「名前は要らない (ほかの `Error` とぶつからないように)。メソッドだけ使えればよい」という書き方です。

### 10.4 自分でトレイトを実装する

このプロジェクトには手で書いたトレイト実装はありませんが、書き方を知っておきましょう。たとえば `Position` を `{}` で `(3, 4)` と表示したければ、`Display` を実装します。

```rust
use std::fmt;

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
// これで format!("{pos}") や pos.to_string() が使える
```

`impl トレイト for 型 { ... }` の形で、トレイトが要求するメソッドを書きます。

### 10.5 ジェネリクスとトレイト境界、そして `dyn`

出典: [src/export.rs:99-108](src/export.rs#L99-L108)
```rust
/// 重み配列は整形出力でも1行にまとめる (1要素1行だと数百万行になるため)
fn inline_array<S: Serializer>(values: &[f32], serializer: S) -> Result<S::Ok, S::Error> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(S::Error::custom("重みに NaN または無限大が含まれています"));
    }
    let json = serde_json::to_string(values).map_err(S::Error::custom)?;
    serde_json::value::RawValue::from_string(json)
        .map_err(S::Error::custom)?
        .serialize(serializer)
}
```

`<S: Serializer>` は、「`S` はどんな型でもよいが、`Serializer` トレイトを実装していること」という意味です (トレイト境界)。
`S::Ok` や `S::Error` は、そのトレイトが決めている型 (関連型) です。

ジェネリクスは、使われる型ごとに専用の機械語が作られる (単相化と呼ぶ) ので、実行時のコストはありません。

`dyn トレイト` (トレイトオブジェクト) はその逆で、「中身の型は実行時に決まる」ものです。
`Box<dyn Error>` は、中身がどのエラー型かを実行時まで決めません。そのぶん大きさがコンパイル時に分からないので、`Box` (ヒープに置いた値を指す、所有権付きのポインタ) に入れて扱います。

| | ジェネリクス `<T: Trait>` | トレイトオブジェクト `dyn Trait` |
|---|---|---|
| 型が決まるとき | コンパイル時 | 実行時 |
| 速さ | 速い (直接呼び出す) | 少し遅い (間接的に呼び出す) |
| 種類の違う値を混ぜて入れる | できない | できる (`Vec<Box<dyn Error>>` など) |
| このコードでの例 | `inline_array<S: Serializer>` | `Box<dyn std::error::Error>` |

### 10.6 `TryFrom`: 失敗するかもしれない変換

出典: [src/export.rs:110-112](src/export.rs#L110-L112)
```rust
fn to_vec(t: &Tensor) -> Vec<f32> {
    Vec::<f32>::try_from(t.flatten(0, -1)).expect("f32 テンソルへの変換に失敗")
}
```

この変換は、1 次元でないテンソルを渡すと失敗する (そのため `flatten(0, -1)` で1次元にしてから渡している) など、失敗することがあるので、`try_from` は `Result` を返します。
要素の型は、tch が自動で `f32` に変換します。
`Vec::<f32>::` のような `::<>` は「ターボフィッシュ」と呼ばれ、型引数をはっきり書く書き方です。

---

## 11. クロージャとイテレータ

### 11.1 クロージャ

クロージャは、その場で作る名前の無い関数で、周りの変数を使えます。TypeScript のアロー関数 `(x) => x + 1` に当たり、Rust では `|x| x + 1` と書きます。

出典: [src/env/rules.rs:35-37](src/env/rules.rs#L35-L37)
```rust
        let tick = cfg.grid.tick_seconds;
        // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
        let ticks = |sec: f32| (sec / tick).round().max(0.0) as u32;
```

`ticks` は、周りの変数 `tick` を取り込んで (キャプチャして) います。このあと `ticks(cfg.grid.time_limit_seconds)` のように、同じ変換を 8 回使っています。
関数の中だけで使う小さな処理をまとめるのに便利です。

同じ使い方: [src/env/observation.rs:64](src/env/observation.rs#L64) の `cell` (座標から添字を計算)、[src/config.rs:170](src/config.rs#L170) の `check`、[src/trainer.rs:239](src/trainer.rs#L239) の `crossed` (試合数が区切りを越えたか)。

引数の無いクロージャは、同じものをいくつも作る「工場」としても使えます。

出典: [src/trainer.rs:55-66](src/trainer.rs#L55-L66)
```rust
    fn new(rules: Rules, seed: u64, n_step: usize, gamma: f32) -> Self {
        let stream = || Stream {
            last: None,
            reward: 0.0,
            force_reward: 0.0,
            nstep: NStepBuilder::new(n_step, gamma),
        };
        Self {
            env: GameEnv::new(rules, seed),
            streams: [stream(), stream()],
        }
    }
```

### 11.2 `move` クロージャ

クロージャは、ふつう周りの変数を借用します。`move` を付けると、所有権ごと取り込みます。
クロージャが今の関数より長生きするとき (別のスレッドで使うときなど) に必要です。

出典: [src/main.rs:171-179](src/main.rs#L171-L179)
```rust
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        // 1回目は学習ループに止まってもらい、保存してから終了する。2回目は保存せずに終了する
        if !flag.swap(false, Ordering::SeqCst) {
            println!();
            std::process::exit(130);
        }
    })?;
```

Ctrl+C のハンドラは別のスレッドで動き、プログラムが終わるまで生き続けます。
`main` の変数を借りたままだと、借りた変数のほうが先に消えてしまうかもしれないので、`move` で `flag` の所有権を渡しています (`Arc` と `AtomicBool` は 12.7)。

### 11.3 イテレータの基本

イテレータは「要素を1つずつ取り出せるもの」で、`map` や `filter` をつなげて処理を書けます。
TS の配列メソッドに似ていますが、違いが2つあります。

1. 遅延評価: `map` などを並べただけでは何も起きず、最後に `collect` / `sum` / `for` などで取り出すときに初めて動く。途中の配列を作らない。
2. ゼロコスト: 最適化で、手で書いた `for` ループと同じ速さになる。

よく使うもの:

| メソッド | 意味 | 使用例 |
|---|---|---|
| `map(f)` | 1つずつ変換する | 多数 |
| `filter(f)` | 条件に合うものだけ残す | [src/env/field.rs:95](src/env/field.rs#L95) |
| `filter_map(f)` | 変換して、`Some` になったものだけ残す | [src/main.rs:226](src/main.rs#L226) |
| `flat_map(f)` | 1つから複数を作り、平らにつなぐ | [src/trainer.rs:344](src/trainer.rs#L344) |
| `enumerate()` | (番号, 要素) の組にする | [src/trainer.rs:354](src/trainer.rs#L354) |
| `zip(other)` | 2つを並べて (a, b) の組にする | [src/trainer.rs:388](src/trainer.rs#L388) |
| `chain(other)` | 後ろにつなぐ | [src/env/field.rs:89-90](src/env/field.rs#L89-L90) |
| `take(n)` / `skip(n)` | 先頭の n 個だけ / 先頭の n 個を飛ばす | [src/env/snake.rs:172](src/env/snake.rs#L172)、[src/env/observation.rs:73](src/env/observation.rs#L73) |
| `rev()` | 逆順にする | [src/replay.rs:164](src/replay.rs#L164) |
| `windows(n)` | 長さ n の窓を1つずつずらしながら | [src/force.rs:172](src/force.rs#L172) |
| `copied()` | `&T` を `T` にする (Copy な型) | [src/env/tests.rs:45](src/env/tests.rs#L45) |
| `collect()` | `Vec` などにまとめる | 多数 |
| `sum()` / `product()` | 合計 / 積 | [src/model.rs:78](src/model.rs#L78)、[src/replay.rs:51](src/replay.rs#L51) |
| `any(f)` / `all(f)` | どれかが / 全部が条件を満たすか | [src/env/snake.rs:172](src/env/snake.rs#L172)、[src/config.rs:196](src/config.rs#L196) |
| `position(f)` | 最初に条件を満たす要素の添字 | [src/env/field.rs:176](src/env/field.rs#L176) |
| `count()` | 個数 | [src/env/tests.rs:71](src/env/tests.rs#L71) |
| `fold(初期値, f)` | 前から順に畳み込む | [src/replay.rs:165](src/replay.rs#L165) |

### 11.4 例1: 空いているマスを列挙する

出典: [src/env/field.rs:81-98](src/env/field.rs#L81-L98)
```rust
    /// ヘビ・アイテム・ブロックのどれも無いマスを列挙する
    fn empty_cells(&self, rules: &Rules) -> Vec<Position> {
        let w = rules.width;
        let mut occupied = vec![false; rules.cell_count()];
        let occupied_positions = self
            .snake
            .body
            .iter()
            .chain(self.items.iter().map(|i| &i.pos))
            .chain(self.obstacles.iter());
        for p in occupied_positions {
            occupied[(p.y * w + p.x) as usize] = true;
        }
        (0..rules.cell_count() as i32)
            .filter(|&i| !occupied[i as usize])
            .map(|i| Position { x: i % w, y: i / w })
            .collect()
    }
```

1. `occupied_positions` は、ヘビの体・アイテムの位置・ブロックの位置を `chain` で1列につないだイテレータです。3つの `Vec` を結合した新しい `Vec` は作っていません。どれも `&Position` を出すように、アイテムだけ `map(|i| &i.pos)` で形をそろえています。
2. `for` で回して、埋まっているマスに印を付けます。
3. `0..n` (範囲もイテレータ) から、印の無いマスだけを `filter` で残し、添字を座標に `map` で変換して、`collect` で `Vec<Position>` にします。戻り値の型が `Vec<Position>` なので、`collect` は何にまとめればよいかを推論できます。

### 11.5 例2: `collect` は型で行き先が変わる

出典: [src/env/snake.rs:85-91](src/env/snake.rs#L85-L91)
```rust
    pub fn new(head: Position, rules: &Rules) -> Self {
        let body = (0..rules.initial_length as i32)
            .map(|i| Position {
                x: head.x,
                y: head.y + i,
            })
            .collect();
```

ここでは `body` の型をどこにも書いていません。このあと `Self { body, ... }` に入れていて、そのフィールドの型が `VecDeque<Position>` なので、`collect` は `VecDeque` を作ります。
`collect` の行き先は `Vec`・`VecDeque`・`String`・`HashMap` など何でもよく、型で決まります。
推論できないときは `collect::<Vec<_>>()` のように書きます (`_` は「中身の型は推論して」という意味。[src/env/tests.rs:45](src/env/tests.rs#L45))。

### 11.6 例3: 行動を決めるプレイヤーを集める

出典: [src/trainer.rs:340-349](src/trainer.rs#L340-L349)
```rust
        let requests: Vec<(usize, usize)> = self
            .slots
            .iter()
            .enumerate()
            .flat_map(|(s, slot)| {
                (0..NUM_PLAYERS)
                    .filter(|&p| slot.env.needs_decision(p))
                    .map(move |p| (s, p))
            })
            .collect();
```

64 個の対戦 × 2 人のうち、このティックで行動を決めるプレイヤーの (対戦の番号, プレイヤーの番号) を集めています。

- `enumerate()` で、対戦に番号 `s` を付ける。
- 対戦ごとに、`0..2` から行動が必要なプレイヤーだけを `filter` で残し、`(s, p)` に `map` する。
- `flat_map` で、対戦ごとの結果 (0〜2 個) を1列につなぐ。

内側の `move |p| (s, p)` の `move` は、`s` を借用ではなくコピーで取り込むためのものです。
内側のイテレータは外側のクロージャの呼び出しが終わったあとも使われるので、外側のクロージャの引数 `s` を借りたままにはできないからです。

### 11.7 例4: 割引収益を後ろから畳み込む

出典: [src/replay.rs:161-166](src/replay.rs#L161-L166)
```rust
    fn discounted_return(&self) -> f32 {
        self.steps
            .iter()
            .rev()
            .fold(0.0, |acc, (_, _, r)| r + self.gamma * acc)
    }
```

報酬 r0, r1, r2 から r0 + γ·r1 + γ²·r2 を計算します。後ろから `acc = r + γ * acc` を繰り返すと、

- r2
- r1 + γ·r2
- r0 + γ·(r1 + γ·r2) = r0 + γ·r1 + γ²·r2

となります。`(_, _, r)` はタプル `(Obs, i64, f32)` の分解で、使わない要素は `_` で捨てています。

### 11.8 例5: 状態を持つクロージャで層の形を計算する

出典: [src/model.rs:33-53](src/model.rs#L33-L53)
```rust
    pub fn new(model: &ModelConfig, rules: &Rules) -> Self {
        let (mut c, mut h, mut w) = (
            GRID_CHANNELS as i64,
            rules.height as i64,
            rules.width as i64,
        );
        let conv = model
            .conv_layers
            .iter()
            .map(|layer| {
                let spec = ConvSpec {
                    in_channels: c,
                    out_channels: layer.channels,
                    stride: layer.stride,
                };
                c = layer.channels;
                h = conv_out(h, layer.stride);
                w = conv_out(w, layer.stride);
                spec
            })
            .collect();
```

`map` のクロージャが、外の `c`・`h`・`w` を書き換えています。前の層の出力チャネル数が次の層の入力チャネル数になるので、層を順に処理しながら形を更新しているのです。

このような「外の変数を書き換える `map`」は、要素を先頭から順に1回ずつ処理することが前提です。
イテレータは遅延評価なので、`collect` するまでは1回も実行されない点にも気を付けてください。
慣れないうちは `for` ループで書いても構いません。すぐ後の全結合層 ([src/model.rs:55-63](src/model.rs#L55-L63)) は、同じ種類の処理を `for` で書いています。どちらにするかは読みやすさで選びます。

### 11.9 例6: 2つの列が同じか比べる

出典: [src/export.rs:196-201](src/export.rs#L196-L201)
```rust
        let same_names = |names: &[String], expected: &[&str]| {
            names
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())
        };
```

`&[String]` と `&[&str]` は型が違うので、そのままでは `==` で比べられません。
`map(String::as_str)` で `&String` を `&str` に変換し (クロージャの代わりに関数名をそのまま渡せる)、`copied()` で `&&str` を `&str` にそろえ、`eq` で1つずつ比べています。

### 11.10 `retain_mut`: その場で間引く

出典: [src/env/field.rs:128-143](src/env/field.rs#L128-L143)
```rust
    /// 出現待ちのタイマーを進め、時間になったものを出現させる
    pub fn process_pending(&mut self, rules: &Rules, rng: &mut StdRng) {
        let mut due_items = Vec::new();
        self.pending_spawns.retain_mut(|p| {
            p.ticks = p.ticks.saturating_sub(1);
            if p.ticks == 0 {
                due_items.push(p.kind);
            }
            p.ticks > 0
        });
        for kind in due_items {
            if !self.spawn_item(kind, rules, rng) {
                // 盤面が埋まっていたら次のティックで再挑戦
                self.pending_spawns.push(PendingSpawn { kind, ticks: 1 });
            }
        }
```

`retain_mut` は、クロージャが `true` を返した要素だけを残し、ほかを取り除きます。新しい `Vec` は作らず、その場で詰めます。
ここでは「タイマーを1減らし、0 になったものは取り除いて `due_items` に移す」を、1回の走査でやっています。

なぜ `due_items` にいったん集めるのでしょう。
`retain_mut` の実行中は `self.pending_spawns` を借りているので、クロージャの中で `self.spawn_item(...)` (`&mut self`、つまり `self` 全体を借りる) は呼べないからです。
いったん集めておき、`retain_mut` が終わってから出現させています。8.4 の借用のルールが、コードの形を決めている例です。

---

## 12. メモリ管理: このプロジェクトの核

強化学習では、大量の経験を貯めながら、同じ処理 (ゲームを進める・推論する・学習する) を何百万回も繰り返します。
メモリの確保と解放が多ければ遅くなり、無駄が多ければメモリが足りなくなります。
この章では、このコードがメモリをどう扱っているかを見ていきます。7 章 (所有権) と 8 章 (借用) の知識を使います。

### 12.1 スタックとヒープ

| | スタック | ヒープ |
|---|---|---|
| 置かれるもの | 関数のローカル変数や引数 (大きさがコンパイル時に決まっている必要がある) | `Vec`・`String`・`Box` などが実行中に確保した領域 |
| 確保と解放 | 関数に入る・出るだけ。とても速い | アロケータ (メモリの管理役) に頼む。比較的遅い |
| 寿命 | 関数を抜けるまで | 所有者が drop するまで |

`Vec<T>` は、スタックに置かれる「ポインタ・長さ・容量」の 24 バイトと、ヒープに置かれる要素の列でできています。

```
スタック (Vec<Position> の本体: 24 バイト)       ヒープ
┌──────────────┐
│ ptr ──────────────────────────▶ [Position][Position][Position][ 空き ][ 空き ]
│ len = 3      │                   ←───── len (使用中) ─────→
│ capacity = 5 │                   ←──────────── capacity (確保済み) ─────────────→
└──────────────┘
```

大きさの決まった値でも、`Vec` の要素になればヒープに置かれます (上の図の `Position` や、トレーナーの `Vec<Slot>` の中の `GameEnv`)。どこに置かれるかは、その値の持ち主がどこにいるかで決まります。

`Box<T>` は「値を1つだけヒープに置き、そのポインタを所有する」型です。大きさがコンパイル時に決まらない `dyn Error` (10.5) を入れるのに使っています。

このプロジェクトの型の大きさ (`std::mem::size_of` で実測。64 bit の Linux):

| 型 | バイト数 | メモ |
|---|---|---|
| `Position` | 8 | `i32` が2つ |
| `Direction`、`Action`、`Option<Action>` | 1 | enum は小さな整数。`Option` も 1 バイトに収まる |
| `Option<usize>` | 16 | `usize` にはすべての値が意味を持つので、「無い」の印の分だけ大きくなる |
| `Rules` | 96 | 数値だけ。Copy にしても安い |
| `Vec<T>`、`String` | 24 | ポインタ + 長さ + 容量 (中身はヒープ) |
| `VecDeque<Position>` | 32 | |
| `&str`、`&[f32]` | 16 | ポインタ + 長さ (ファットポインタと呼ぶ) |
| `Snake` | 56 | 体の中身はヒープ |
| `Field` | 160 | |
| `GameEnv` | 768 | うち 320 バイトは乱数生成器 `StdRng` の内部状態 |
| `tch::Tensor` | 8 | libtorch 側のテンソルを指すポインタだけ |

### 12.2 `Vec` の容量と再確保

`Vec` に `push` して容量が足りなくなると、より大きな領域を確保し直し、中身を全部移します (再確保)。
これが何度も起きると遅くなるので、個数が分かっているなら最初に確保しておきます。

出典: [src/replay.rs:102-110](src/replay.rs#L102-L110)
```rust
    pub fn sample(&self, batch_size: usize, rng: &mut StdRng, device: Device) -> Batch {
        let (g, v) = (self.grid_len, self.vector_len);
        let mut grids = Vec::with_capacity(batch_size * g);
        let mut next_grids = Vec::with_capacity(batch_size * g);
        let mut vectors = Vec::with_capacity(batch_size * v);
        let mut next_vectors = Vec::with_capacity(batch_size * v);
        let mut actions = Vec::with_capacity(batch_size);
        let mut returns = Vec::with_capacity(batch_size);
        let mut dones = Vec::with_capacity(batch_size);
```

`with_capacity` は「中身は空だが、この数までは再確保なしで入る」`Vec` を作ります。
バッチの大きさ (128) × 観測の長さ分を最初に確保するので、このあとの `extend_from_slice` で再確保は起きません。

### 12.3 リプレイバッファ: 大量のデータを詰めて持つ

リプレイバッファは、過去の経験 (観測 → 行動 → 報酬 → 次の観測) を最大 20 万件貯めておく入れ物です。学習で使うメモリの大半はここです。

素直に書くと、1件を構造体にして `Vec` に並べたくなるでしょう。

```rust
// 素直な書き方 (このコードでは使っていない)
struct Transition {
    obs: Obs,      // 中に Vec<u8> と Vec<f32>
    action: i64,
    ret: f32,
    next: Obs,     // 中に Vec<u8> と Vec<f32>
    done: bool,
}
let buffer: Vec<Transition> = ...;
```

これだと1件ごとにヒープの確保が4回あり、20 万件で 80 万個の小さな領域がばらばらに置かれます。
確保と解放に時間がかかり、メモリも散らばるので、読むときに CPU のキャッシュが効きにくくなります。

実際のコードは、データの種類ごとに1本の大きな配列にまとめています。この形を Structure of Arrays (SoA) と呼びます。

出典: [src/replay.rs:32-66](src/replay.rs#L32-L66)
```rust
/// n ステップ遷移を貯めるリングバッファ
pub struct ReplayBuffer {
    capacity: usize,
    grid_len: usize,
    vector_len: usize,
    grid_shape: [i64; 3],
    grids: Vec<u8>,
    next_grids: Vec<u8>,
    vectors: Vec<f32>,
    next_vectors: Vec<f32>,
    actions: Vec<i64>,
    returns: Vec<f32>,
    dones: Vec<f32>,
    len: usize,
    pos: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize, grid_shape: [i64; 3], vector_len: usize) -> Self {
        let grid_len = grid_shape.iter().product::<i64>() as usize;
        Self {
            capacity,
            grid_len,
            vector_len,
            grid_shape,
            grids: vec![0; capacity * grid_len],
            next_grids: vec![0; capacity * grid_len],
            vectors: vec![0.0; capacity * vector_len],
            next_vectors: vec![0.0; capacity * vector_len],
            actions: vec![0; capacity],
            returns: vec![0.0; capacity],
            dones: vec![0.0; capacity],
            len: 0,
            pos: 0,
        }
```

メモリの並びはこうなります。

```
grids:   [ 遷移0 の盤面 (16x16 なら 2304 バイト) | 遷移1 の盤面 | ... | 遷移199999 の盤面 ]
vectors: [ 遷移0 (13 個) | 遷移1 (13 個) | ... ]
actions: [ a0 | a1 | a2 | ... ]
```

- 起動時に `vec![0; capacity * grid_len]` で全部を確保し、以後は確保も解放もしません。
- i 番目の遷移の盤面は、`grids[i * g..(i + 1) * g]` という計算で取り出せます。

書き込みは `copy_from_slice` (中身はメモリのコピー命令) です。

出典: [src/replay.rs:77-100](src/replay.rs#L77-L100)
```rust
    /// `next` が None なら終端遷移
    pub fn push(&mut self, obs: &Obs, action: i64, ret: f32, next: Option<&Obs>) {
        let i = self.pos;
        let (g, v) = (self.grid_len, self.vector_len);
        self.grids[i * g..(i + 1) * g].copy_from_slice(&obs.grid);
        self.vectors[i * v..(i + 1) * v].copy_from_slice(&obs.vector);
        match next {
            Some(next) => {
                self.next_grids[i * g..(i + 1) * g].copy_from_slice(&next.grid);
                self.next_vectors[i * v..(i + 1) * v].copy_from_slice(&next.vector);
                self.dones[i] = 0.0;
            }
            None => {
                // 終端では次状態を使わないが、古い値が残らないよう消しておく
                self.next_grids[i * g..(i + 1) * g].fill(0);
                self.next_vectors[i * v..(i + 1) * v].fill(0.0);
                self.dones[i] = 1.0;
            }
        }
        self.actions[i] = action;
        self.returns[i] = ret;
        self.pos = (self.pos + 1) % self.capacity;
        self.len = (self.len + 1).min(self.capacity);
    }
```

リングバッファの仕組み: `pos` は次に書く位置で、末尾まで行くと `% self.capacity` で先頭に戻ります。
いっぱいになったら、いちばん古い遷移から上書きされます。`len` は入っている件数で、`capacity` で頭打ちになります。

```
capacity = 5 のとき
1〜5 回目の push:  [0][1][2][3][4]   pos: 0→1→2→3→4→0、len: 1→5
6 回目の push:     [5][1][2][3][4]   いちばん古い 0 を上書き。pos = 1、len = 5 のまま
```

`copy_from_slice` は、コピー元と先の長さが違うと panic します。長さが食い違っても黙って壊れるのではなく、その場で止まって教えてくれます。

**量子化: 盤面を 1 バイトに詰める**

出典: [src/replay.rs:6-20](src/replay.rs#L6-L20)
```rust
/// リプレイに保存する観測。盤面はメモリ節約のため 0..=255 に量子化する
#[derive(Debug, Clone)]
pub struct Obs {
    pub grid: Vec<u8>,
    pub vector: Vec<f32>,
}

impl Obs {
    pub fn quantize(grid: &[f32], vector: &[f32]) -> Self {
        Self {
            grid: grid.iter().map(|&v| (v * 255.0).round() as u8).collect(),
            vector: vector.to_vec(),
        }
    }
}
```

盤面の値は 0.0〜1.0 なので、255 倍して `u8` (0〜255 の 1 バイト整数) にすれば、`f32` (4 バイト) の 4 分の 1 の大きさで持てます。
学習で取り出すときに、`/ 255.0` で元に戻します ([src/replay.rs:125-131](src/replay.rs#L125-L131))。
胴体の値 (首が 1.0 で、尻尾に向かって小さくなる) には誤差が出ますが、刻みは 1/255 ≒ 0.004 なので、ネットワークの入力としては十分です。

状態ベクトル (13 個) は `f32` のまま持っています。スコア差のように負の値 (-1〜1) があり、数も 13 個だけなので、詰めても節約はわずかだからです (練習問題 7)。

1 遷移あたりのバイト数は、コードで計算しています。

出典: [src/replay.rs:69-71](src/replay.rs#L69-L71)
```rust
    pub fn bytes_per_transition(grid_len: usize, vector_len: usize) -> usize {
        2 * (grid_len + vector_len * 4) + 8 + 4 + 4
    }
```

(観測 + 次の観測) × (盤面 `u8` + ベクトル `f32` × 4 バイト) + 行動 `i64` + 収益 `f32` + 終端の印 `f32`、という計算です。

| 盤面 | 盤面の要素数 (9 チャネル) | 1 遷移 (`u8` に量子化) | `f32` のままなら | 20 万件 (`u8`) | 20 万件 (`f32`) |
|---|---|---|---|---|---|
| 8×8 | 576 | 1,272 バイト | 4,728 バイト | 約 254 MB | 約 946 MB |
| 16×16 | 2,304 | 4,728 バイト | 18,552 バイト | 約 946 MB | 約 3.7 GB |

16×16 の盤面を `f32` のまま 20 万件持つと 3.7 GB 必要なところを、量子化で約 0.95 GB に抑えています。
学習を始めるときに、この見積もりを画面に表示しています ([src/main.rs:109-111](src/main.rs#L109-L111))。

### 12.4 バッファを使い回す: 呼び出し側がメモリを用意する

観測を作る関数 `encode` は、新しい `Vec` を返すのではなく、呼び出し側が渡した領域に書き込みます (8.2 のシグネチャ)。

もし `encode` が `Vec<f32>` を返す形だったら、行動を決めるたびに (64 対戦 × 2 人が、ほぼ 2 ティックに1回ずつ)、観測 (16×16 なら `f32` が 2,304 個で約 9 KB) を確保しては捨てることになります。

トレーナーは、観測を書き込む領域をフィールド (`grid_buf`、`vector_buf`) として持ち続け、使い回しています。

出典: [src/trainer.rs:351-353](src/trainer.rs#L351-L353)
```rust
        let n = requests.len();
        self.grid_buf.resize(n * g, 0.0);
        self.vector_buf.resize(n * v, 0.0);
```

`resize` は長さを変えますが、容量が足りていれば再確保しません。最初の数ティックで最大の大きさまで育ったあとは、確保が起きなくなります。
この「`&mut [T]` を受け取って書き込む関数」と「呼び出し側が持ち続けるバッファ」の組み合わせは、速さが大事な Rust のコードでよく使われる形です。

使い回すバッファには前回の値が残っているので、`encode` は書き込む前に `grid.fill(0.0)` で全体を消しています ([src/env/observation.rs:66](src/env/observation.rs#L66))。
リプレイバッファの `push` が、終端のときに次の観測の場所を消している ([src/replay.rs:89-94](src/replay.rs#L89-L94)) のも同じ理由です。

### 12.5 `VecDeque`: 両端で出し入れする

ヘビの体は `VecDeque<Position>` です ([src/env/snake.rs:67](src/env/snake.rs#L67))。

出典: [src/env/snake.rs:175-183](src/env/snake.rs#L175-L183)
```rust
    pub fn advance(&mut self, next: Position) {
        self.body.push_front(next);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.body.pop_back();
        }
        self.last_moved_dir = self.dir;
    }
```

進むときは、先頭に新しい頭を足し、末尾の尻尾を取ります。
`Vec` の先頭に挿入すると全要素をずらすので時間が長さに比例します (O(n))。`VecDeque` (リングバッファでできた両端キュー) なら、両端の出し入れはどちらも一定時間 (O(1)) です。
TS 版は配列の `unshift` を使っていて ([web/src/game/snake.ts:129](../web/src/game/snake.ts#L129))、こちらは長さに比例した時間がかかります (ヘビは短いので、実際には問題になりません)。

n ステップの遷移を組み立てる `NStepBuilder` も、「後ろに足して前から取る」ので `VecDeque` を使っています ([src/replay.rs:149](src/replay.rs#L149))。

| 入れ物 | 得意なこと | このコードでの例 |
|---|---|---|
| `Vec<T>` | 末尾への追加、添字でのアクセス | アイテム、リプレイバッファ |
| `VecDeque<T>` | 両端での追加・削除 | ヘビの体、n ステップのキュー |
| `[T; N]` | 大きさが固定。ヒープを使わない | 2 人分の盤面、行動ごとの集計 |
| `HashMap<K, V>` | キーで値を引く | (このコードでは使っていない) |

### 12.6 テンソルのメモリ (tch)

`tch::Tensor` は 8 バイトのハンドル (取っ手) で、本体のデータは libtorch (C++) 側のメモリにあります。
Rust の所有権は、このハンドルにもそのまま働きます。
`Tensor` が drop されると、libtorch 側のテンソルへの参照が1つ減り、最後の参照がなくなったときにメモリが解放されます。
参照には、`shallow_clone` で作ったハンドルや、逆伝播のために計算グラフが持っているものも含まれます。

気を付ける点が4つあります。

1. **`Tensor::from_slice` はコピーする**: Rust の `Vec` の中身を libtorch のメモリに複製します ([src/replay.rs:126](src/replay.rs#L126) など)。GPU を使うときは、`.to(device)` でさらに GPU へコピーします。

2. **`shallow_clone` はコピーしない**:

出典: [src/model.rs:136-141](src/model.rs#L136-L141)
```rust
    pub fn forward(&self, grid: &Tensor, vector: &Tensor) -> Tensor {
        let mut x = grid.shallow_clone();
        for conv in &self.conv {
            x = x.apply(conv).relu();
        }
        let mut x = Tensor::cat(&[x.flatten(1, -1), vector.shallow_clone()], 1);
```

`forward` は入力を `&Tensor` (借用) で受け取りますが、`x = x.apply(...)` と次々に置き換えていくには、自分で所有する `Tensor` が必要です。
`shallow_clone` は、同じデータを指す新しいハンドルを作ります (データそのものはコピーしない)。
tch の `Tensor` は `Clone` トレイトを実装しておらず、データを複製したいときは `copy()`、ハンドルだけ増やしたいときは `shallow_clone()` と、はっきり書き分けるようになっています。

3. **`no_grad` は計算の記録を止める**:

出典: [src/trainer.rs:462](src/trainer.rs#L462)
```rust
        let best = tch::no_grad(|| self.online.forward(&grid, &vector).argmax(1, false));
```

PyTorch は、学習 (逆伝播) のために計算の途中結果を記録します。
行動を選ぶだけのときや、目標値を作るときは記録が要らないので、`no_grad` の中で計算して、メモリと時間を節約しています (Python の `with torch.no_grad():` と同じ)。

4. **途中のテンソルのハンドルはすぐ drop される**: `x = x.apply(conv).relu();` では、`apply` の結果の `Tensor` は `relu` に使われたあと不要になり、その文の終わりで drop されます。
   学習中 (`no_grad` の外) は、逆伝播のために計算グラフが中身を持ち続けることがありますが、それも逆伝播が終わってグラフが要らなくなれば解放されます。GC を待たず、参照がなくなったところで解放されます。

### 12.7 スレッド間で共有する: `Arc` と `AtomicBool`

Ctrl+C のハンドラ (別のスレッド) と学習ループ (メインのスレッド) は、「学習を続けるか」のフラグを共有しています ([src/main.rs:171-179](src/main.rs#L171-L179)。コードは 11.2 に引用)。

- `Arc<T>` (Atomically Reference Counted): 1つの値を複数の所有者で共有するための箱。`clone()` しても中身は複製されず、「所有者の数」が1つ増えるだけ。最後の所有者が drop したときに、中身が解放される。
- `AtomicBool`: 複数のスレッドから同時に読み書きしても壊れない `bool`。

ふつうの `bool` や参照ではだめなのでしょうか。試すと、どれもコンパイルエラーか、正しく動かないコードになります。

- `&mut bool` は1つしか作れない (8.1) ので、ハンドラとループの両方から書き換えられません。
- ローカル変数への参照 `&running` をハンドラに渡すと、「`running` が十分長く生きていない」というエラー (E0597) になります (実際に試した結果)。ハンドラはプログラムの終わりまで生きる (`'static`) ことを要求されるのに、`running` は `main` の終わりで消えるからです。
- スレッドをまたげない `Rc` (Arc のスレッド非対応版) を使うと、こうなります (実際に試した結果)。

```
error[E0277]: `Rc<Cell<bool>>` cannot be sent between threads safely
    = help: within `{closure@src/main.rs:173:24: 173:31}`, the trait `Send` is not implemented for `Rc<Cell<bool>>`
```

`Send` は「別のスレッドに渡してよい」ことを表すトレイトです。
このように Rust では、スレッド間で安全でない共有をしようとすると、コンパイルの時点で止められます。
`Arc<AtomicBool>` は、`&` (共有参照) のままで値を書き換えられる (`swap` や `store`) ように作られていて、スレッド間で安全に共有できます。

学習ループの側は、`&AtomicBool` で受け取って読むだけです。

```rust
trainer.run(&running, cli.games, &paths, commit, &mut log)?;   // src/main.rs:189
while running.load(Ordering::SeqCst)                            // src/trainer.rs:228
```

`swap(false, ...)` は「false を書き込み、書き込む前の値を返す」メソッドです。
1回目の Ctrl+C では前の値が true なので、何もせずに戻ります (学習ループがフラグを見て止まり、モデルを保存する)。
2回目では前の値がすでに false なので、保存を待たずに終了します。

`Ordering::SeqCst` は、複数のスレッドの間でメモリの読み書きの順序をどこまで保証するかの指定です。SeqCst がいちばん強く、迷ったらこれを使えば安全です。

### 12.8 整数のオーバーフローと範囲外アクセス

Rust は、何が起きるか分からない状態 (C でいう未定義動作) を、ふつうのコードでは起こさせません。

符号なし整数から引いて 0 を下回ると、ビルドの種類によって結果が変わります (実際に試した結果)。

```rust
let mut move_cooldown: u32 = 0;
move_cooldown -= 1;
// debug ビルド:   panic ("attempt to subtract with overflow")
// release ビルド: 4294967295 に折り返す (panic しない)
```

このコードでは、0 を下回りうる場所では `saturating_sub` (0 で止まる引き算) を使っています。

出典: [src/env/game.rs:92-94](src/env/game.rs#L92-L94)
```rust
    pub fn remaining_ticks(&self) -> u32 {
        self.rules.time_limit_ticks.saturating_sub(self.tick)
    }
```

一方で、`self.move_cooldown -= 1;` ([src/env/snake.rs:148](src/env/snake.rs#L148)) はふつうの引き算です。
`move_cooldown` は常に 1 以上に保たれる、という前提があるからです (0 になったらすぐ移動の間隔に戻し、その間隔は `max(1)` で 1 以上にしている。[src/env/rules.rs:45-46](src/env/rules.rs#L45-L46))。
前提が崩れれば、debug ビルドで動かしたときに panic して気づけます。
このように「起こりうるなら `saturating_sub`、起こらないはずならふつうの演算 (debug で検査される)」と使い分けます。
ほかに `checked_sub` (失敗すると `None`) や `wrapping_sub` (わざと折り返す) もあります。

配列の範囲外アクセスは、release でも必ず panic します (境界チェック)。

```
index out of bounds: the len is 4 but the index is 5
```

C のように、隣のメモリを黙って読み書きすることはありません。境界チェックのコストは小さく、イテレータを使うと最適化で消えることも多いです。

`debug_assert_eq!` は、debug ビルドでだけ働く検査で、release では消えます。

出典: [src/env/observation.rs:58-59](src/env/observation.rs#L58-L59)
```rust
    debug_assert_eq!(grid.len(), GRID_CHANNELS * plane);
    debug_assert_eq!(vector.len(), VECTOR_FEATURES);
```

何百万回も呼ばれる関数なので、学習 (release) の速さは落とさずに、debug ビルドで動かしたときだけ呼び出し側の誤りを見つけるための検査です。

注意: このプロジェクトの `just test` は、tch を使うテストを速くするために `cargo test --release` で実行します。
そのため `just test` では、整数のオーバーフローの検査も `debug_assert!` も働きません。
これらの検査を効かせたいときは、`cargo test` (`--release` なし) で実行してください (tch を使うテストは遅くなります)。
release でも検査したい場合は、Cargo.toml の `[profile.release]` に `overflow-checks = true` や `debug-assertions = true` を書く方法もあります。

### 12.9 ファイルを壊さずに書く

出典: [src/export.rs:183-192](src/export.rs#L183-L192)
```rust
    /// 一時ファイルに書いてから置き換えるので、書き込み中に中断しても壊れたファイルが残らない
    pub fn write(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
```

モデルの JSON は 16×16 の盤面で約 1.8 MB あり、書いている途中で Ctrl+C やエラーが起きると、中途半端なファイルが残ってしまいます。
一時ファイルに最後まで書いてから `rename` で置き換えると (同じファイルシステムの中の rename は一瞬で入れ替わる)、「古くて完全なファイル」か「新しくて完全なファイル」のどちらかしか存在しません。
メモリ管理と同じく、「中途半端な状態を外から見せない」考え方です。

ログは `BufWriter` でまとめて書きつつ、1 行ごとに `flush()` しています ([src/monitor.rs:69-73](src/monitor.rs#L69-L73))。
`BufWriter` は小さな書き込みをメモリに溜めてまとめて OS に渡すことで、システムコール (OS への依頼) の回数を減らします。
ただし溜めている間は `tail -f` で見えないので、見せたい区切りで `flush` します。

### 12.10 まとめ: このコードで使った工夫

| 工夫 | 場所 | 効果 |
|---|---|---|
| 起動時にまとめて確保し、使い回す | `ReplayBuffer::new` | 学習中に確保・解放が起きない |
| データの種類ごとに1本の配列 (SoA) | `ReplayBuffer` | 小さな確保が無く、メモリが連続する |
| `u8` への量子化 | `Obs::quantize` | 盤面のメモリが 4 分の 1 |
| `&mut [T]` に書き込ませる | `observation::encode` | 呼ぶたびの確保が無い |
| バッファを `resize` で使い回す | `Trainer::tick` | 容量が足りれば再確保しない |
| `with_capacity` | `ReplayBuffer::sample` | 再確保が無い |
| 所有権の移動 (`clone` しない) | `Stream::last` から `NStepBuilder` へ | 観測を複製しない |
| `VecDeque` | `Snake::body` | 先頭への追加が一定時間 |
| `retain_mut` | `Field::process_pending` | 新しい `Vec` を作らずに間引く |
| `no_grad` | `select_actions`、`train_step` | 要らない計算の記録をしない |
| `shallow_clone` | `QNetwork::forward` | テンソルを複製しない |
| drop による自動解放 | `*slot = Slot::new(...)` など | 解放し忘れ・二重解放が無い |

---

## 13. 外部クレートの使い方

### 13.1 serde: 構造体と YAML・JSON の変換

出典: [src/config.rs:6-15](src/config.rs#L6-L15)
```rust
/// config.yaml 全体
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub game: GameConfig,
    pub model: ModelConfig,
    pub reward: RewardConfig,
    pub train: TrainConfig,
    pub export: ExportConfig,
}
```

- `#[derive(Deserialize)]` を付けるだけで、`serde_yaml::from_str(...)` で YAML から構造体を作れます。YAML のキー名とフィールド名が対応します。
- `#[serde(deny_unknown_fields)]` は、構造体に無いキーが YAML にあればエラーにします。キー名の打ち間違い (`learning_rat:` など) が黙って無視されるのを防ぎます。
- 型も検査されます。`width: 8.5` と書けば、`i32` にならないのでエラーになります。
- 足りないキーもエラーになります (`Option<T>` のフィールドや、`#[serde(default)]` を付けたフィールドは省略できる)。

書き出し (`Serialize`) も同じです。`ModelFile` ([src/export.rs:37-48](src/export.rs#L37-L48)) は、`serde_json::to_string_pretty(self)` の1行で JSON になります。

フィールドごとの細かい調整もできます。

出典: [src/export.rs:82-85](src/export.rs#L82-L85)
```rust
    #[serde(serialize_with = "inline_array")]
    pub weight: Vec<f32>,
    #[serde(serialize_with = "inline_array")]
    pub bias: Vec<f32>,
```

重みは全部で約 16 万個 (16×16 の設定の場合) あり、整形して出力すると1要素1行になるので、ファイルが十数万行になってしまいます。
`serialize_with` で、この2つだけ自作の関数 `inline_array` (10.5 に引用) で書き出し、1 行にまとめています。
`inline_array` は、NaN や無限大 (JSON では表せない) が混じっていたらエラーにする検査も兼ねています。

### 13.2 clap: コマンドライン引数

出典: [src/main.rs:26-38](src/main.rs#L26-L38)
```rust
/// DuelSnake-AI の自己対戦強化学習。Ctrl+C で中断すると、その時点のモデルを JSON で保存して終了する
#[derive(Parser)]
struct Cli {
    /// 設定ファイル [既定: learn/config.yaml]
    #[arg(long)]
    config: Option<PathBuf>,
    /// モデルの出力先 [既定: model/]
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// 保存済みモデル JSON から学習を再開する。値を省略すると config.yaml の盤面サイズのモデル
    /// (<out_dir>/recent-model/snake-model-<幅>x<高さ>.json) から再開し、無ければ警告を出して新しく学習する
    #[arg(long, value_name = "MODEL_JSON")]
    resume: Option<Option<PathBuf>>,
```

- 構造体を書いて `#[derive(Parser)]` を付けると、`Cli::parse()` で引数を解析してくれます。
- フィールド名 `out_dir` から、`--out-dir` という引数が作られます。
- `///` のコメントが `--help` の説明文になります (`cargo run -- --help` で確かめられる)。
- フィールドの型で、引数の性質が決まります。

| フィールドの型 | 意味 | 例 |
|---|---|---|
| `bool` | 付ければ true になるフラグ | `--auto-commit` |
| `Option<u64>` | 省略できる値 | `--games 50000` |
| `Option<Option<PathBuf>>` | 省略できる。付けたときも値を省略できる | `--resume` / `--resume path.json` |

`--resume` の3通りは、そのまま `match` で分けています。

出典: [src/main.rs:84-105](src/main.rs#L84-L105)
```rust
    let resume = match cli.resume {
        Some(Some(path)) => Some(read_resume_model(&path, &config, &mut notices)?),
        Some(None) if recent_model.exists() => {
            Some(read_resume_model(&recent_model, &config, &mut notices)?)
        }
        Some(None) => {
            ...
        }
        None => None,
    };
```

`Some(Some(path))` が `--resume path.json`、`Some(None)` が `--resume` だけ、`None` が付けていない場合です。

### 13.3 rand: 再現できる乱数

出典: [src/env/game.rs:72-74](src/env/game.rs#L72-L74)
```rust
    pub fn new(rules: Rules, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let fields = [Field::new(&rules, &mut rng), Field::new(&rules, &mut rng)];
```

乱数生成器 `StdRng` を、試合ごとにシード (乱数の種) から作り、`&mut rng` で渡して使います。
どこからでも呼べる乱数 (`rand::random()`) ではなく、生成器を引数で回しているのは、同じシードなら同じ試合を再現できるようにするためです。
テストでシードを固定している (`GameEnv::new(rules, 0)`、[src/env/tests.rs:20](src/env/tests.rs#L20)) のもこのためです。

乱数は取り出すたびに内部の状態が進むので、使うには `&mut` が必要です。
乱数を使う関数はシグネチャに `rng: &mut StdRng` が現れるので、どこで乱数を使っているかも一目で分かります ([src/env/field.rs:101](src/env/field.rs#L101) の `spawn_item`)。

### 13.4 tch: PyTorch を Rust から使う

tch は、PyTorch の C++ 版 (libtorch) を Rust から使うためのクレートです。PyTorch を知っていれば、名前はほぼ同じです。

出典: [src/model.rs:99-119](src/model.rs#L99-L119)
```rust
impl QNetwork {
    pub fn new(vs: &nn::Path, spec: &NetworkSpec) -> Self {
        let conv = spec
            .conv
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let cfg = nn::ConvConfig {
                    stride: c.stride,
                    padding: PADDING,
                    ..Default::default()
                };
                nn::conv2d(
                    vs / format!("conv{i}"),
                    c.in_channels,
                    c.out_channels,
                    KERNEL_SIZE,
                    cfg,
                )
            })
            .collect();
```

- `nn::VarStore` はパラメータ (重み) の入れ物です。`online_vs.root()` で得られる `nn::Path` (この関数の引数 `vs`) に `/` を使うと、`vs / "conv0"` のように名前の階層を作れます (Rust の演算子オーバーロード。`/` の働きは `Div` トレイトで決まり、tch は `nn::Path` に実装している。`VarStore` そのものには `/` は使えない)。
- `nn::conv2d` や `nn::linear` で層を作ると、そのパラメータが VarStore に登録されます。
- オプティマイザは VarStore 全体を対象に作ります: `nn::Adam::default().build(&online_vs, train.learning_rate)` ([src/trainer.rs:163](src/trainer.rs#L163))。

PyTorch との対応:

| PyTorch (Python) | tch (Rust) | 使用箇所 |
|---|---|---|
| `nn.Conv2d(cin, cout, 3, stride=s, padding=1)` | `nn::conv2d(vs / "conv0", cin, cout, 3, cfg)` | [src/model.rs:111](src/model.rs#L111) |
| `layer(x)` | `x.apply(&layer)` | [src/model.rs:139](src/model.rs#L139) |
| `torch.relu(x)` | `x.relu()` | [src/model.rs:139](src/model.rs#L139) |
| `torch.cat([a, b], 1)` | `Tensor::cat(&[a, b], 1)` | [src/model.rs:141](src/model.rs#L141) |
| `x.flatten(1)` | `x.flatten(1, -1)` | [src/model.rs:141](src/model.rs#L141) |
| `x.argmax(1)` | `x.argmax(1, false)` | [src/trainer.rs:462](src/trainer.rs#L462) |
| `q.gather(1, a).squeeze(1)` | `q.gather(1, &a, false).squeeze_dim(1)` | [src/trainer.rs:502](src/trainer.rs#L502) |
| `F.smooth_l1_loss(q, t)` | `q.smooth_l1_loss(&t, Reduction::Mean, 1.0)` | [src/trainer.rs:516](src/trainer.rs#L516) |
| `with torch.no_grad():` | `tch::no_grad(\|\| ...)` | [src/trainer.rs:462](src/trainer.rs#L462) |
| `torch.tensor(a)` (コピーする) | `Tensor::from_slice(&v)` | [src/replay.rs:126](src/replay.rs#L126) |
| `x.view(b, c, h, w)` | `x.view([b, c, h, w])` | [src/replay.rs:127](src/replay.rs#L127) |
| `x.item()` | `x.double_value(&[])` | [src/trainer.rs:500](src/trainer.rs#L500) |
| `target.load_state_dict(online.state_dict())` | `target_vs.copy(&online_vs)` | [src/trainer.rs:162](src/trainer.rs#L162) |
| `zero_grad()` → `backward()` → `clip_grad_norm_()` → `step()` | `optimizer.backward_step_clip_norm(&loss, max_norm)` | [src/trainer.rs:517-518](src/trainer.rs#L517-L518) |

Python との違い: Rust には省略できる引数やキーワード引数が無いので、`argmax(1, false)` のように全部の引数を書きます (この `false` は keepdim)。

### 13.5 標準ライブラリのそのほかの道具

- `std::process::Command`: 外部コマンドを実行する ([src/git.rs:19-25](src/git.rs#L19-L25))。
- `std::time::Instant` と `Duration`: 経過時間を測る ([src/trainer.rs:220](src/trainer.rs#L220)、[src/trainer.rs:224](src/trainer.rs#L224))。
- `std::io::IsTerminal`: 出力先が端末かどうかを調べる ([src/monitor.rs:139](src/monitor.rs#L139))。端末ならその場で書き換える表示に、ファイルへのリダイレクトなら1行ずつの表示に切り替えています。
- `std::path::Path` と `PathBuf`: ファイルのパス。`&str` と `String` と同じ関係で、`Path` が借用、`PathBuf` が所有です。引数は `&Path` で受けるのが慣例です ([src/config.rs:157](src/config.rs#L157))。

---

## 14. 強化学習のコードを読む

ここまでの文法の知識で、学習の中心部を読んでみましょう。

### 14.1 全体像

- **環境**: `GameEnv`。1ティックずつ進みます。AI が行動を決めるのは、ヘビが動く直前のティックだけです (`needs_decision`)。
- **エージェント**: `QNetwork`。観測 (盤面 9 チャネル + 状態ベクトル 13 個) から、6 つの行動それぞれの Q 値 (その行動をとったとき、将来もらえる報酬の見込み) を出します。
- **自己対戦**: 両方のプレイヤーを同じネットワークで動かし、両方の経験で学習します。
- **並列化**: 64 対戦を同時に進め、行動を決めるプレイヤーの観測をまとめて1回で推論します (バッチ推論)。

### 14.2 ε-greedy: `select_actions`

出典: [src/trainer.rs:432-447](src/trainer.rs#L432-L447)
```rust
    /// ε-greedy。ランダムに決まらなかった分だけまとめて推論する
    fn select_actions(&mut self, n: usize, stats: &mut WindowStats) -> Vec<i64> {
        let (g, v) = (self.grid_len(), VECTOR_FEATURES);
        let epsilon = self.epsilon();
        let mut actions = vec![0i64; n];
        let mut greedy = Vec::new();
        for (k, action) in actions.iter_mut().enumerate() {
            if self.rng.gen::<f64>() < epsilon {
                *action = self.rng.gen_range(0..Action::COUNT as i64);
            } else {
                greedy.push(k);
            }
        }
        if greedy.is_empty() {
            return actions;
        }
```

確率 ε でランダムに行動し、それ以外は Q 値が最大の行動を選びます。
ランダムでないもの (`greedy`) だけを集めて、1回の `forward` でまとめて推論しています。
ε は学習の進み具合に応じて、1.0 から 0.05 まで直線的に下げていきます ([src/trainer.rs:205-210](src/trainer.rs#L205-L210))。

### 14.3 n ステップの遷移: `NStepBuilder`

1 手ごとの報酬だけでなく、n 手先 (n = 3) までの報酬をまとめた遷移を作ります。

(観測 s_t, 行動 a_t, 収益 r_t + γ·r_{t+1} + γ²·r_{t+2}, 3 手後の観測 s_{t+3})

- `steps` (`VecDeque`) に 1 手ずつ足し、n 個溜まったら先頭から1つ取り出して遷移を確定させます ([src/replay.rs:180-188](src/replay.rs#L180-L188))。
- 試合が終わったら、残りをすべて終端遷移 (次の観測なし) として出します ([src/replay.rs:189-198](src/replay.rs#L189-L198))。

### 14.4 Double DQN の更新: `train_step`

出典: [src/trainer.rs:486-518](src/trainer.rs#L486-L518)
```rust
    /// 1回更新し、(損失, 学習バッチでの max Q の平均) を返す
    fn train_step(&mut self) -> (f64, f64) {
        let train = &self.config.train;
        let batch = self
            .replay
            .sample(train.batch_size, &mut self.rng, self.device);
        let bootstrap_discount = train.gamma.powi(train.n_step as i32);

        let q_all = self.online.forward(&batch.grid, &batch.vector);
        ...
        let q = q_all.gather(1, &batch.actions, false).squeeze_dim(1);
        let target = tch::no_grad(|| {
            // Double DQN: 次の行動はオンライン側で選び、その価値はターゲット側で見積もる
            let next_action = self
                .online
                .forward(&batch.next_grid, &batch.next_vector)
                .argmax(1, true);
            let next_q = self
                .target
                .forward(&batch.next_grid, &batch.next_vector)
                .gather(1, &next_action, false)
                .squeeze_dim(1);
            &batch.returns + next_q * (1.0 - &batch.dones) * bootstrap_discount
        });
        let loss = q.smooth_l1_loss(&target, Reduction::Mean, 1.0);
        self.optimizer
            .backward_step_clip_norm(&loss, train.grad_clip_norm);
```

目標値の式は、次のとおりです。

y = R + γⁿ · (1 − done) · Q_target(s', argmax_a Q_online(s', a))

| 式 | コード |
|---|---|
| Q_online(s, a) | `q_all.gather(1, &batch.actions, false)`: 実際にとった行動の Q 値を取り出す |
| argmax_a Q_online(s', a) | `self.online.forward(次の観測).argmax(1, true)` |
| Q_target(s', その行動) | `self.target.forward(次の観測).gather(1, &next_action, false)` |
| R (n ステップの収益) | `batch.returns` |
| (1 − done) | `(1.0 - &batch.dones)`: 試合が終わっていたら、次の価値を足さない |
| γⁿ | `bootstrap_discount` |
| 損失 | Huber 損失 (`smooth_l1_loss`) |

Rust らしい点:

- `&batch.returns + next_q * ...` の `&`: tch の演算子は `Tensor` (所有) でも `&Tensor` (借用) でも受け付けます。所有で渡すとその `Tensor` は消費されるので、構造体の一部 (`batch.returns`) のように消費したくないものは、`&` を付けて借用で計算します。演算子も所有権のルールに従うのです。
- 目標値は `no_grad` の中で計算して、目標側に勾配が流れないようにしています (PyTorch の `.detach()` に当たる)。
- ターゲットネットワークは、`target_update_interval` (1000) 回の更新ごとにオンライン側からコピーします ([src/trainer.rs:520-529](src/trainer.rs#L520-L529))。`is_multiple_of` は「割り切れるか」を調べるメソッドです。

### 14.5 報酬の補助: force.rs

リンゴに近づくと少しだけ報酬が増える「引力」などを、位置エネルギーの差で計算しています (potential-based reward shaping と呼ばれる方法)。
近づいてから離れると差し引きゼロになるので、「リンゴの周りを行ったり来たりして報酬を稼ぐ」ことができません。
テスト `round_trip_earns_nothing` ([src/force.rs:209-224](src/force.rs#L209-L224)) が、これを確かめています。詳しくは [RULES.md](RULES.md) の 12 章を見てください。

### 14.6 Web との約束: 観測とモデルの JSON

学習したモデルは Web (TypeScript) で動かすので、次の2つを両側で完全に一致させる必要があります。

- 観測の作り方: `learn/src/env/observation.rs` と `web/src/ai/observation.ts`
- ネットワークの計算: tch と `web/src/ai/model.ts`

export.rs のテスト `reference_forward_matches_tch` ([src/export.rs:373-395](src/export.rs#L373-L395)) は、Web 側と同じ手順の素朴な順伝播を Rust で書き、tch の結果と一致するかを確かめています。
「別の言語で書いた2つの実装が同じ答えを出す」ことを、テストで保証しているわけです。
観測の仕様を変えたら、[CLAUDE.md](../CLAUDE.md) にあるとおり両方をそろえ、モデルを学習し直します。

---

## 15. テストの書き方

### 15.1 テストの置き場所

置き方は2通りあります。

**1. 同じファイルの末尾に置く (いちばん一般的)**

出典: [src/monitor.rs:213-222](src/monitor.rs#L213-L222)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_counts_for_display() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(1_234_567), "1,234,567");
```

- `#[cfg(test)]` で、テストのときだけコンパイルします。
- `use super::*;` で、親モジュール (monitor) の中身を全部使えるようにします。子モジュールなので、`pub` でない関数も呼べます (3.3)。
- `#[test]` を付けた関数が1つのテストになります。
- `1_000` の `_` は、数字を読みやすくするための区切りで、値には影響しません。

**2. 別のファイルに分ける**

env のテストは量が多いので [src/env/tests.rs](src/env/tests.rs) に分け、[src/env/mod.rs](src/env/mod.rs) で `#[cfg(test)] mod tests;` と宣言しています。

### 15.2 assert の種類

```rust
assert!(条件);                            // 真であること
assert!(条件, "失敗したときの文 {x}");      // メッセージ付き
assert_eq!(実際の値, 期待する値);           // 等しいこと。失敗すると両方の値を表示する
assert_ne!(a, b);                         // 等しくないこと
```

`assert_eq!` は、失敗したときに両方の値を `{:?}` で表示するので、比べる型に `Debug` が必要です (とりあえず `Debug` を derive しておく理由の1つ)。

浮動小数点数には誤差があるので、`==` ではなく差の大きさで比べます。

出典: [src/force.rs:141-143](src/force.rs#L141-L143)
```rust
    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }
```

### 15.3 状況を作る補助関数

同じ準備を何度も書かないように、補助関数を作ります。

出典: [src/env/tests.rs:35-48](src/env/tests.rs#L35-L48)
```rust
/// プレイヤー0が次に移動するティックまで進め、そのティックでは `action` を入力する
fn move_once(env: &mut GameEnv, action: Option<Action>) {
    while !env.needs_decision(0) {
        env.step([None, None]);
    }
    env.step([action, None]);
}

fn set_body(env: &mut GameEnv, player: usize, body: &[Position], dir: Direction) {
    let snake = &mut env.fields[player].snake;
    snake.body = body.iter().copied().collect::<VecDeque<_>>();
    snake.dir = dir;
    snake.last_moved_dir = dir;
}
```

これを使うと、1つのテストが「状況を作る → 動かす → 確かめる」の3段で短く書けます。

出典: [src/env/tests.rs:196-210](src/env/tests.rs#L196-L210)
```rust
#[test]
fn hitting_wall_is_immediate_loss() {
    let mut env = empty_env();
    set_body(
        &mut env,
        0,
        &[pos(8, 0), pos(8, 1), pos(8, 2)],
        Direction::Up,
    );
    env.fields[0].snake.score = 10;
    move_once(&mut env, None);
    let result = env.result.expect("試合が終わっていない");
    assert_eq!(result.winner, Some(1));
    assert_eq!(result.reason, EndReason::Death);
}
```

テストの名前は「何を確かめるか」を文にした snake_case にしています (`hitting_wall_is_immediate_loss` = 壁にぶつかったら即負け)。
失敗したとき、名前だけで何が壊れたかが分かります。

### 15.4 表で回すテスト

出典: [src/env/tests.rs:221-234](src/env/tests.rs#L221-L234)
```rust
#[test]
fn simultaneous_deaths_are_decided_by_score() {
    for (scores, winner) in [([2, 5], Some(1)), ([4, 4], None)] {
        let mut env = empty_env();
        for p in 0..2 {
            set_body(&mut env, p, &[pos(3, 0), pos(3, 1)], Direction::Up);
            env.fields[p].snake.score = scores[p];
        }
        move_once(&mut env, None);
        let result = env.result.expect("試合が終わっていない");
        assert_eq!(result.winner, winner);
        assert_eq!(result.reason, EndReason::Death);
    }
}
```

(入力, 期待する結果) の組の配列を `for` で回すと、似たケースをまとめて書けます。

### 15.5 実行する

```sh
cargo test --release                  # 全部 (just test と同じ)
cargo test --release force            # 名前に force を含むテストだけ
cargo test --release -- --nocapture   # テストの中の println! も表示する
```

`--release` を付けているのは、tch を使うテストが debug ビルドだと遅いからです。
ただし release では、整数のオーバーフローの検査と `debug_assert!` が働かなくなります (12.8)。ルールのテストだけなら `cargo test env::` のように debug で実行するのも手です。

---

## 16. Rust らしい書き方 (慣例)

### 16.1 名前の付け方

| 種類 | 書き方 | 例 |
|---|---|---|
| 変数・関数・モジュール | snake_case | `pending_growth`、`spawn_item`、`observation` |
| 型・トレイト・enum の値 | UpperCamelCase | `GameEnv`、`ItemType::NormalApple` |
| 定数 | SCREAMING_SNAKE_CASE | `NUM_PLAYERS`、`GRID_CHANNELS` |

メソッドの名前の慣例:

| 形 | 意味 | 例 |
|---|---|---|
| `new` | コンストラクタ | `Snake::new` |
| `from_xxx` | xxx から作る | `Rules::from_config` |
| `to_xxx` | 新しい値を作って変換する (コストがあるかもしれない) | `to_string`、`to_vec` |
| `as_xxx` | 安い変換 (参照のまま見方を変える) | `as_str`、`as_ref` |
| `into_xxx` | 自分を消費して変換する | `into_iter` |
| `is_xxx` | `bool` を返す | `is_over`、`is_filled`、`is_active` |
| `try_xxx` | 失敗するかもしれない (できなければ何もしない、または `Result` を返す) | `try_boost`、`try_from` |
| `xxx_mut` | `&mut` を扱う版 | `iter_mut`、`retain_mut` |

値を読むだけのメソッド (ゲッター) には `get_` を付けないのが慣例です (`rules()` であって `get_rules()` ではない)。

### 16.2 引数は借用で、戻り値は所有で

- 読むだけの引数は、所有する型より借用で受け取る: `String` より `&str`、`PathBuf` より `&Path`、`Vec<T>` より `&[T]`。呼ぶ側が何を持っていても渡せます。
- 長く持ち続けるもの (構造体に保存するもの) は、所有権ごと受け取る: `Trainer::new(config: Config, ...)`。
- 戻り値は、引数から借りたものでなければ、所有する型で返す。

### 16.3 型で間違いを防ぐ

- ありえない状態を作れない型にする: `Option<HeldItem>` (6.4)、勝敗の理由の enum `EndReason`。
- 単位の違うものは型を分ける: `GameConfig` (秒) と `Rules` (ティック) (5.4)。
- 長さの決まったものは配列にする: `[Field; NUM_PLAYERS]`、`[f64; Action::COUNT]`。
- `match` では `_` に頼らず、全部の場合を書く (6.2)。

### 16.4 入口で検査して、中では信じる

`Config::validate` (9.4)、`ModelFile::read` の形式の確認 ([src/export.rs:173-179](src/export.rs#L173-L179))、`load_into` の層の形の確認 ([src/export.rs:207-230](src/export.rs#L207-L230)) がその例です。
外から来るデータは入口で全部調べ、中のコードでは同じ検査を繰り返しません。

### 16.5 コメントには「なぜ」を書く

出典: [src/env/rules.rs:36-37](src/env/rules.rs#L36-L37)
```rust
        // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
        let ticks = |sec: f32| (sec / tick).round().max(0.0) as u32;
```

コードを読めば分かる「何をしているか」ではなく、「なぜ `round` (四捨五入) なのか、切り捨てではだめな理由」を書いています。
テスト `seconds_are_rounded_to_ticks` ([src/env/tests.rs:50-62](src/env/tests.rs#L50-L62)) が、これを確かめています。

### 16.6 ツールに任せる

- `cargo fmt`: 書式はすべて rustfmt に任せ、手で揃えません。`just check` は `cargo fmt --check` も実行します。
- `cargo clippy`: 「もっと良い書き方」を教えてくれます。たとえば `(digits.len() - i) % 3 == 0` と書くと、clippy は `is_multiple_of(3)` を使うよう勧めます (実際に試した結果。monitor.rs の `thousands` はその形になっている)。2026-09-28 時点で、本体のコードに clippy の警告はありません (テストに2件。練習問題 9)。
- 警告 (warning) は放っておかない。使っていない変数や要らない `mut` などは、すぐに直します。

---

## 17. よくあるコンパイルエラーと直し方

この表のエラーは、どれもこの教材を書くときに実際に出して確かめたものです。

| エラー | 意味 | よくある原因 | 直し方 | 本文 |
|---|---|---|---|---|
| E0382 use of moved value | 移動したあとの値を使った | 値を渡したあと、もう一度使う | `&` で借用して渡す / `.clone()` / 順番を変える | 7.2 |
| E0499 cannot borrow as mutable more than once | `&mut` を2つ同時に作った | 同じ配列の2つの要素の同じフィールドを、同時に `&mut` で借りる | 分解する (`let [a, b] = &mut arr;`) / `split_at_mut` / 借用を短くする | 8.5 |
| E0502 cannot borrow as immutable because it is also borrowed as mutable | `&mut` を持ったまま `&` を作った | フィールドを借りたまま `&self` のメソッドを呼ぶ | 先に値を取り出す / フィールドを直接使う | 8.4 |
| E0507 cannot move out of ... behind a reference | 借りているものから所有権を持ち出そうとした | `for x in self.v` | `&self.v` / `.take()` / `.clone()` | 8.7 |
| E0594 cannot assign to ..., as ... is not declared as mutable / E0384 cannot assign twice to immutable variable | `mut` の無い変数 (やそのフィールド) に代入した | `let` に `mut` を付け忘れた | `let mut` にする | 4.1 |
| E0596 cannot borrow ... as mutable | `mut` の無い変数を `&mut` で借りた | `let v = Vec::new(); v.push(1);` のように、`&mut self` のメソッドを呼んだ | `let mut` にする | 4.1 |
| E0597 does not live long enough | 参照の先が先に消えてしまう | ローカル変数の参照を、長生きするもの (別スレッドなど) に渡した | 所有権ごと渡す (`move`、`Arc`) | 12.7 |
| E0308 mismatched types | 型が違う | 行末の `;` で値を捨てた / 型の違う値を渡した | `;` を外す / `as` などで変換する | 4.4 |
| E0277 the trait bound ... is not satisfied | 必要なトレイトが無い | `usize * i32`、`Rc` を別スレッドへ、`[値; N]` の値が Copy でない | `as` で型をそろえる / derive を足す / 型を変える | 4.3、12.7 |
| E0599 no method named ... found | メソッドが見つからない | トレイトを `use` していない | `use rand::Rng;` などを足す | 10.3 |
| E0004 non-exhaustive patterns | `match` に漏れがある | enum に値を足した | 漏れた場合の腕を足す | 6.2 |
| E0106 missing lifetime specifier | 参照の出どころが分からない | 参照の引数が複数ある関数で参照を返す / 関数の中の値の参照を返す | `'a` を書く / 所有する型を返す | 8.10 |

エラーを読むコツ:

1. いちばん上のエラーから直す。後ろのエラーは、最初のエラーの巻き添えのことが多い。
2. `-->` の場所、`^^^^` の下線、`help:` を読む。Rust のエラーには直し方まで書いてあることが多い。
3. `rustc --explain E0502` で、詳しい説明と例を読める。

---

## 18. ゼロから同じ構成のプロジェクトを作る

次に似たもの (ゲームと学習、シミュレーションと解析など) を作るときの手順です。

```sh
cargo new my-project           # Cargo.toml と src/main.rs ができる
cd my-project
cargo add serde --features derive
cargo add serde_json serde_yaml
cargo add clap --features derive
cargo add rand@0.8 chrono ctrlc
cargo add tch@0.14 --features download-libtorch
```

`rand` は 0.9 で `gen` が `random` に、`gen_range` が `random_range` に名前が変わりました。この教材のコードと同じ書き方で進めるなら、上のように 0.8 を指定してください。

進め方:

1. **ドメイン (ゲームのルール) を先に書く**。外部クレートにほとんど頼らない、独立したモジュールにします (このプロジェクトの `env/`)。乱数は引数で受け取ります (13.3)。
2. **ルールのテストを一緒に書く**。`cargo test` で確かめながら進めます (15 章)。
3. **設定は serde の構造体で受け、読み込んだ直後に検査する** (9.4、13.1)。
4. **重い処理 (学習) は、ドメインの上に別のモジュールとして載せる**。ドメインから学習のコードは参照しません (`env/` は `trainer` を知らない)。
5. **main.rs は薄く保つ**。引数を読み、部品を組み立て、エラーは `?` で返すだけにします。
6. **`cargo fmt`・`cargo clippy`・`cargo test` を習慣にする**。

main.rs の雛形:

```rust
mod config;
mod env;

use clap::Parser;
use std::path::PathBuf;

/// プログラムの説明 (--help に出る)
#[derive(Parser)]
struct Cli {
    /// 設定ファイル
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = config::Config::load(&cli.config)?;
    // ここで部品を組み立てて動かす
    Ok(())
}
```

---

## 19. 練習問題

実際に `learn/` のコードに書き足して試してください。解答は、この教材を書くときにすべてコンパイルし、テストが通ることを確かめています。
テストからしか使わない関数を足すと、`cargo build` で「使われていない (dead_code)」という警告が出ますが、練習なので気にしなくて構いません。
終わったら `git restore src` で元に戻せます。

### 問題1 (構造体と配列)

`Position` に、上下左右の4マスを返すメソッド `neighbors(self) -> [Position; 4]` を追加し、テストを書いてください。

<details>
<summary>ヒント</summary>

配列にも `map` があります (`[a, b, c].map(f)` は `[f(a), f(b), f(c)]`)。`Position::step` が使えます。

</details>

<details>
<summary>解答</summary>

src/env/snake.rs の `impl Position` の中に:

```rust
    /// 上下左右の 4 マス (盤面の外も含む)
    pub fn neighbors(self) -> [Position; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .map(|d| self.step(d))
    }
```

src/env/tests.rs に:

```rust
#[test]
fn neighbors_are_four_adjacent_cells() {
    let n = pos(3, 3).neighbors();
    assert_eq!(n, [pos(3, 2), pos(3, 4), pos(2, 3), pos(4, 3)]);
    assert!(n.iter().all(|&p| p.manhattan(pos(3, 3)) == 1));
}
```

`self` は `Position` (Copy) なので、クロージャの中で何度使っても構いません。

</details>

### 問題2 (イテレータ)

`Field` に、指定した種類のアイテムがいくつあるかを返す `count_items(&self, kind: ItemType) -> usize` を追加してください。
テスト `initial_state_matches_rules` ([src/env/tests.rs:71](src/env/tests.rs#L71)) の `count` クロージャを、これで置き換えてみましょう。

<details>
<summary>解答</summary>

```rust
    /// `kind` のアイテムが盤面にいくつあるか
    pub fn count_items(&self, kind: ItemType) -> usize {
        self.items.iter().filter(|i| i.kind == kind).count()
    }
```

`ItemType` は `PartialEq` を derive しているので `==` で比べられます。

</details>

### 問題3 (enum と match)

`EndReason` に、日本語の表示名を返す `label(self) -> &'static str` を追加してください ("盤面を埋めた" / "衝突" / "時間切れ")。
書けたら、わざと1つの腕を消して、どんなエラーが出るか見てみましょう。

<details>
<summary>解答</summary>

src/env/game.rs に:

```rust
impl EndReason {
    pub fn label(self) -> &'static str {
        match self {
            EndReason::Filled => "盤面を埋めた",
            EndReason::Death => "衝突",
            EndReason::TimeUp => "時間切れ",
        }
    }
}
```

腕を消すと E0004 (non-exhaustive patterns) になり、どの値が漏れたかが表示されます。

</details>

### 問題4 (所有権と借用のある構造体をテストする)

`Snake::apply_growth` ([src/env/snake.rs:185-201](src/env/snake.rs#L185-L201)) を直接テストしてください。
`Snake::new(pos(8, 8), &rules())` で長さ 3 のヘビを作り、`pending_growth = 2` にしてから `apply_growth(-3)` を呼ぶと、長さと `pending_growth` はどうなるでしょう。
予想してから確かめてください。さらに `apply_growth(-10)` を呼んだらどうなるでしょう。

<details>
<summary>解答</summary>

-3 のうち 2 は伸びる予定の打ち消しに使われ、残り 1 マスだけ尻尾が縮みます。長さ 2、`pending_growth` 0 です。
さらに -10 しても、最短の 1 マスで止まります。

tests.rs の先頭の `use super::snake::{...}` に `Snake` を足してから:

```rust
#[test]
fn shrink_cancels_pending_growth_first() {
    let mut snake = Snake::new(pos(8, 8), &rules());
    snake.pending_growth = 2;
    snake.apply_growth(-3);
    assert_eq!((snake.len(), snake.pending_growth), (2, 0));
    snake.apply_growth(-10);
    assert_eq!(snake.len(), 1);
}
```

</details>

### 問題5 (エラー処理)

`"16x16"` のような文字列を盤面の大きさ `(i32, i32)` に変換する関数 `parse_size(s: &str) -> Result<(i32, i32), String>` を export.rs に書いてください。
`"16"`、`"16x"`、`"ax3"` のような文字列には、分かりやすいエラー文を返します。

<details>
<summary>ヒント</summary>

`str::split_once('x')` は `Option<(&str, &str)>` を返します。`Option::ok_or_else` で `Result` に変えれば `?` が使えます。
数への変換は `v.parse::<i32>()` で、失敗すると `Err` を返します。`map_err` でエラー文を作りましょう。

</details>

<details>
<summary>解答</summary>

```rust
/// "16x16" のような文字列を (幅, 高さ) にする
pub fn parse_size(s: &str) -> Result<(i32, i32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| format!("{s} は <幅>x<高さ> の形ではありません"))?;
    let parse = |v: &str| {
        v.parse::<i32>()
            .map_err(|e| format!("{s} の {v:?} を数として読めません: {e}"))
    };
    Ok((parse(w)?, parse(h)?))
}
```

テスト (export.rs の `mod tests` の中):

```rust
    #[test]
    fn parses_board_size() {
        assert_eq!(parse_size("16x16"), Ok((16, 16)));
        for bad in ["16", "16x", "ax3"] {
            assert!(parse_size(bad).is_err(), "{bad}");
        }
    }
```

実際のエラー文は、たとえば `16x の "" を数として読めません: cannot parse integer from empty string` になります。

</details>

### 問題6 (借用のエラーを直す)

次のメソッドを `impl Trainer` に足すと、E0502 になります (練習用で、処理に意味はありません)。
エラーを読み、2通りの方法で直してください。

```rust
    fn fill_rewards(&mut self) {
        for slot in &mut self.slots {
            for stream in &mut slot.streams {
                stream.reward = self.grid_len() as f32;
            }
        }
    }
```

<details>
<summary>解答</summary>

`self.slots` を `&mut` で借りている間に、`self.grid_len()` が `self` 全体を `&` で借りようとするのが原因です (8.4)。

方法1: ループの前に値を取り出す。

```rust
    fn fill_rewards(&mut self) {
        let g = self.grid_len() as f32;
        for slot in &mut self.slots {
            for stream in &mut slot.streams {
                stream.reward = g;
            }
        }
    }
```

方法2: メソッドではなく、必要なフィールドだけを使う。

```rust
                stream.reward = (GRID_CHANNELS * self.rules.cell_count()) as f32;
```

`self.rules` と `self.slots` は別のフィールドなので、同時に借りられます。

</details>

### 問題7 (メモリの見積もり)

状態ベクトル (13 個) も `u8` に量子化すると、16×16 の盤面で 1 遷移は何バイトになり、20 万件で何 MB 減るでしょう。
それでも `f32` のままにしている理由も考えてください (12.3)。

<details>
<summary>解答</summary>

2 × (2304 + 13) + 8 + 4 + 4 = 4,650 バイト。今の 4,728 バイトとの差は 78 バイトで、20 万件なら 15.6 MB (全体の約 1.6%) です。

一方で、スコア差は負の値 (-1〜1) をとるので、`u8` にするには符号の扱いを考える必要があり、ほかの特徴量の精度も落ちます。
節約が小さいわりに間違いの元が増えるので、`f32` のままが妥当です。
「測ってから最適化する」「効果の大きいところ (盤面) だけ最適化する」という判断の例です。

</details>

### 問題8 (学習のコードを変える)

ε の減らし方を、直線から指数関数的な減衰 ε = end + (start − end) · exp(−判断数 / decay) に変えてください ([src/trainer.rs:205-210](src/trainer.rs#L205-L210))。
変えたら `cargo test --release` と `just train-smoke` で動くことを確かめ、元に戻してください。

<details>
<summary>解答</summary>

```rust
    fn epsilon(&self) -> f64 {
        let t = &self.config.train;
        let progress = self.info.decisions as f64 / t.epsilon_decay_decisions.max(1) as f64;
        t.epsilon_end + (t.epsilon_start - t.epsilon_end) * (-progress).exp()
    }
```

`f64` には `exp()` などの数学の関数がメソッドとして付いています。

</details>

### 問題9 (clippy の指摘を読む)

`learn/` で `cargo clippy --tests` を実行すると、tests.rs に2件の警告 (`the loop variable p is used to index scores`) が出ます。
警告の説明を読み、1件を clippy の勧める形に直してみてください。直すべきかどうかも考えてみましょう。

<details>
<summary>解答</summary>

`for p in 0..2 { ... scores[p] ... }` のように、ループの番号で配列を引いていることへの指摘です。`enumerate` を使うと、こう書けます。

```rust
        for (p, &score) in scores.iter().enumerate() {
            set_body(&mut env, p, &[pos(3, 0), pos(3, 1)], Direction::Up);
            env.fields[p].snake.score = score;
        }
```

ただ、このテストでは `p` を `env.fields[p]` と `set_body` にも使っていて、「プレイヤー 0 と 1 について」という意図は元の書き方のほうが伝わりやすい、とも言えます。
clippy の指摘は絶対ではなく、読みやすさと比べて決めてよいものです (指摘を抑えるには `#[allow(clippy::needless_range_loop)]` を付けます)。

</details>

---

## 20. 次に読むもの

- The Rust Programming Language 日本語版 (通称 the book): https://doc.rust-jp.rs/book-ja/
  - この教材との対応: 4 章 (所有権) ↔ 7・8 章、6 章 (enum) ↔ 6 章、9 章 (エラー処理) ↔ 9 章、10 章 (ジェネリクス・トレイト・ライフタイム) ↔ 8.10・10 章、13 章 (クロージャとイテレータ) ↔ 11 章、16 章 (並行性) ↔ 12.7
- Rust By Example 日本語版: https://doc.rust-jp.rs/rust-by-example-ja/
- Rustlings (小さな問題を解きながら覚える): https://github.com/rust-lang/rustlings
- 標準ライブラリのドキュメント: https://doc.rust-lang.org/std/ (`Vec`、`Option`、`Result`、`Iterator` のページは一度眺めておくと役に立ちます)
- tch 0.14 のドキュメント: https://docs.rs/tch/0.14.0/tch/
- Rust API Guidelines (名前の付け方などの慣例): https://rust-lang.github.io/api-guidelines/

---

## 付録: 用語集

| 用語 | 意味 | 本文 |
|---|---|---|
| クレート (crate) | Rust のパッケージの単位。このプロジェクトは `duelsnake-learn` というクレート | 2.1 |
| モジュール | クレートの中を分ける単位。ふつうは1ファイルが1モジュール | 3 |
| マクロ | `!` の付く、コンパイル時に展開される道具 (`println!`、`vec!` など) | 2.3 |
| 所有権 (ownership) | 値を持ち、drop する責任。値ごとに所有者は1人 | 7 |
| 移動 (move) | 所有権を別の変数に渡すこと。元の変数は使えなくなる | 7.2 |
| Copy | 代入のときに暗黙に複製される性質 | 7.3 |
| Clone | `.clone()` ではっきり書いて複製する性質 | 7.4 |
| drop | 所有者がいなくなった値を破棄し、資源を解放すること | 7.6 |
| RAII | 資源の寿命を変数の寿命に結びつける考え方 | 7.6 |
| 借用 (borrow) | 所有権を渡さずに、参照を作って使わせること | 8 |
| 共有参照 `&T` | 読むだけの参照。いくつでも作れる | 8.1 |
| 可変参照 `&mut T` | 書き換えられる参照。同時に1つだけ | 8.1 |
| ライフタイム | 参照が有効な期間。`'a` のように書く | 8.10 |
| スライス | 配列や `Vec` の一部への参照 (`&[T]`) | 4.7 |
| パターン | 値の形に合わせて分解・照合する書き方 (`Some(x)`、`(a, b)`、`[true, false]`) | 6.3 |
| トレイト | 型ができる操作の約束 (TS の interface に近い) | 10 |
| derive | よくあるトレイトを自動で実装する属性 | 10.2 |
| ジェネリクス | 型を引数にとる書き方 (`<T>`) | 10.5 |
| トレイトオブジェクト | `dyn Trait`。中身の型が実行時に決まる値 | 10.5 |
| クロージャ | その場で作る名前の無い関数。周りの変数を取り込める | 11.1 |
| イテレータ | 要素を1つずつ取り出せるもの。遅延評価 | 11.3 |
| panic | 回復しない異常終了 | 9.5 |
| スタック / ヒープ | 関数のローカル変数の置き場 / `Vec` や `Box` などが実行中に確保する領域 | 12.1 |
| SoA (Structure of Arrays) | データの種類ごとに1本の配列にまとめる並べ方 | 12.3 |
| 量子化 | 値を粗い刻みの小さな型 (ここでは `u8`) に詰めること | 12.3 |
| リングバッファ | 末尾まで行ったら先頭に戻って上書きする入れ物 | 12.3 |
| Arc | スレッド間で共有できる参照カウント付きの箱 | 12.7 |
| アトミック型 | 複数のスレッドから同時に触っても壊れない型 (`AtomicBool` など) | 12.7 |
