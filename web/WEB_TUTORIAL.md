# Web アプリ開発入門: duelsnake-ai の web/ を教材にして

この文書は、`web/` にある Web アプリ (人と AI がリアルタイムに対戦するスネークゲーム) を教材にして、Web アプリの仕組みと作り方を学ぶためのものです。
Web アプリをほとんど作ったことのない人が、読み終えたときに次の 2 つをできるようになることを目標にしています。

- このリポジトリの `web/` のコードを、どのファイルのどの行が何をしているか説明できる
- 同じ規模のアプリ (ブラウザで動くゲームや、学習済みモデルを使うツール) をゼロから作れる

前半 (1〜5 章) で「コードがどこで動き、何がどう通信しているか」と「それをどういう観点で選ぶか」を押さえます。
中盤 (6〜15 章) で TypeScript・React・Canvas・入力・AI 推論の実装を、実際のコードを引用しながら読みます。
後半 (16〜21 章) でテスト・公開・変更の手順と、ゼロから作るときの手順をまとめます。

## 目次

- [0. この教材の使い方](#0-この教材の使い方)
- [1. 全体像: どこで何が動いているか](#1-全体像-どこで何が動いているか)
- [2. 通信: 何がいつ送られるか](#2-通信-何がいつ送られるか)
- [3. 設計の選び方: どこで動かし、どう通信するか](#3-設計の選び方-どこで動かしどう通信するか)
- [4. ツールチェーン: Node.js・npm・Vite・TypeScript・Vitest](#4-ツールチェーン-nodejsnpmvitetypescriptvitest)
- [5. ディレクトリ構成と依存の向き](#5-ディレクトリ構成と依存の向き)
- [6. ブラウザがページを表示するまで](#6-ブラウザがページを表示するまで)
- [7. TypeScript の基礎をこのリポジトリで学ぶ](#7-typescript-の基礎をこのリポジトリで学ぶ)
- [8. React の基礎: コンポーネント・props・state](#8-react-の基礎-コンポーネントpropsstate)
- [9. React のフック: useEffect・useRef・useMemo・useCallback](#9-react-のフック-useeffectuserefusememousecallback)
- [10. ゲームエンジン: src/game/](#10-ゲームエンジン-srcgame)
- [11. ゲームループ: requestAnimationFrame と固定ティック](#11-ゲームループ-requestanimationframe-と固定ティック)
- [12. 描画: Canvas と DOM の使い分け](#12-描画-canvas-と-dom-の使い分け)
- [13. 入力: キーボードとタッチ](#13-入力-キーボードとタッチ)
- [14. AI: モデルの読み込みと推論](#14-ai-モデルの読み込みと推論)
- [15. ブラウザの機能: localStorage と History API](#15-ブラウザの機能-localstorage-と-history-api)
- [16. スタイル: CSS とレイアウト](#16-スタイル-css-とレイアウト)
- [17. テスト: Vitest](#17-テスト-vitest)
- [18. ビルドと公開: GitHub Actions と GitHub Pages](#18-ビルドと公開-github-actions-と-github-pages)
- [19. 変更するときのレシピ](#19-変更するときのレシピ)
- [20. ゼロから同じ構成のアプリを作る](#20-ゼロから同じ構成のアプリを作る)
- [21. よくあるハマりどころ](#21-よくあるハマりどころ)
- [22. 練習問題](#22-練習問題)
- [23. 次に読むもの](#23-次に読むもの)
- [付録: 用語集](#付録-用語集)

---

## 0. この教材の使い方

### 0.1 読み方

- コードを引用するときは、直前に「出典:」としてファイルと行番号のリンクを付けています。VS Code や GitHub で開くと、その行に飛べます。
  リンクはこのファイル (`web/WEB_TUTORIAL.md`) からの相対パスです。`src/...` は `web/src/...` を指します。
- 引用は説明に必要な部分だけを抜き出しています。前後を読みたいときはリンク先を開いてください。
- 行番号は書いた時点のものです。コードを直すとずれるので、ずれていたら関数名で検索してください。
- 「ルールそのもの」の説明は [../learn/RULES.md](../learn/RULES.md) が正です。この文書は「Web でどう実装しているか」に絞ります。
- Rust 側 (学習) の説明は [../learn/RUST_TUTORIAL.md](../learn/RUST_TUTORIAL.md) にあります。この文書と対になっています。

### 0.2 手を動かしながら読む

読むだけより、動かしながら読むほうが早く身につきます。まず次を動かせるようにしてください (Node.js v18 以降が必要です)。

```sh
just web-install  # 初回のみ。web/node_modules/ に依存パッケージを入れる
just web-dev      # http://localhost:3000 で開発サーバーを起動
just web-test     # テストを実行
```

`just` が無ければ、`web/` に移動して `npm install`、`npm run dev`、`npm test` と打っても同じです ([../justfile](../justfile) の末尾を見ると、中身がこの npm コマンドだと分かります)。

開発サーバーを動かしたまま `src/` のファイルを保存すると、ブラウザが自動で更新されます。
「この値を変えたらどうなるか」を試しながら読むのがおすすめです。たとえば次を試してみてください。

- [src/components/draw.ts:20-23](src/components/draw.ts#L20-L23) の色を変える → ヘビの色が変わる
- [src/components/GameScreen.tsx:16-29](src/components/GameScreen.tsx#L16-L29) にキーを足す → そのキーで操作できる
- [src/components/gestures.ts:24](src/components/gestures.ts#L24) の `swipeDistance` を変える → スワイプの感度が変わる

ブラウザの開発者ツール (F12 キー、Mac は Cmd+Option+I) もよく使います。

| タブ | 使いみち |
|---|---|
| Console | `console.log` の出力とエラーを見る |
| Network | どのファイルをいつ取りに行ったか (通信) を見る。2 章で使う |
| Elements | 今の HTML と、各要素にかかっている CSS を見る |
| Sources | ブレークポイントを置いてコードを 1 行ずつ動かす |
| Performance | 1 フレームにかかった時間を測る。11 章・14 章の話を確かめられる |
| Application | localStorage の中身を見る・消す。15 章で使う |

---

## 1. 全体像: どこで何が動いているか

### 1.1 一言でいうと

このアプリは **「サーバーを持たない、ブラウザだけで完結するアプリ」** です。

- ゲームの進行 (ヘビの移動・衝突・アイテム) も、AI の推論 (次の一手を決める計算) も、すべて遊ぶ人のブラウザの中で動きます。
- サーバー (GitHub Pages) は、HTML・JavaScript・CSS・画像・モデルの JSON という **ファイルを返すだけ** です。プログラムは動かしません。
- ゲーム中にブラウザとサーバーの間で通信は起きません。一度ページとモデルを読み込めば、ネットワークを切っても遊べます。

このような構成を **静的サイト** (static site) や、1 枚の HTML の中で画面を切り替えることから **SPA** (Single Page Application) と呼びます。

### 1.2 コードが動く 4 つの場所

「コードが動いている場所」は 1 つではありません。時期ごとに 4 か所あります。

```
 [1] 学習する PC (Rust)          [2] 開発する PC (Node.js)         [3] GitHub Actions (Node.js)
 learn/ を cargo run             web/ を npm run dev / npm test    main に push すると
   │ 自己対戦で学習                │ Vite が TS を JS に変換して      │ npm ci → npm test → npm run build
   │                               │ localhost:3000 で配信            │ web/dist/ を作る
   ▼                               │                                  ▼
 model/recent-model/               │                                GitHub Pages
 snake-model-16x16.json ──git──────┴──────────────────────────────▶ (ファイルを置くだけの
 (重みとルールを入れた JSON)                                           静的ホスティング)
                                                                        │ HTTP で配る
                                                                        ▼
                                                             [4] 遊ぶ人のブラウザ (JavaScript)
                                                                  React が画面を作る
                                                                  ゲームエンジンが 0.1 秒ごとに進む
                                                                  AI がモデルで推論する
                                                                  Canvas に盤面を描く
```

| 場所 | 動く言語・道具 | 何をしているか | このリポジトリの該当箇所 |
|---|---|---|---|
| [1] 学習する PC | Rust | 自己対戦で強化学習し、モデルを JSON に書き出す | [../learn/src/export.rs](../learn/src/export.rs) |
| [2] 開発する PC | Node.js + Vite | TypeScript を変換して配信、テストを実行 | [vite.config.js](vite.config.js), [package.json](package.json) |
| [3] GitHub Actions | Node.js | テストとビルドをして、できたファイルを Pages に上げる | [../.github/workflows/deploy-web.yml](../.github/workflows/deploy-web.yml) |
| [4] 遊ぶ人のブラウザ | JavaScript (TypeScript を変換したもの) | ゲーム・AI・描画・入力のすべて | [src/](src/) の全部 |

ここで大事なのは、**Node.js は開発とビルドの道具として使うだけで、公開後にはどこでも動いていない** という点です。
「TypeScript で書いたから Node.js のサーバーが要る」わけではありません。ブラウザが実行するのは、ビルドで作った JavaScript だけです。

### 1.3 ブラウザの中で動いているもの

ブラウザの中では、次の部品が協力して動いています。矢印は「呼び出す」向きです。

```
                 App.tsx (設定欄・モデルの読み込み・画面の切り替え)
                    │
        ┌───────────┼──────────────┐
        ▼           ▼              ▼
  GameScreen   SoloScreen     RulesScreen          ← components/ (React の画面)
   │  │  │        │
   │  │  └──── Board.tsx → draw.ts (Canvas に描く)
   │  └─────── PlayerPanel.tsx (スコアや Q 値の表)
   │           gestures.ts (タッチ操作を見分ける)
   │
   ├──▶ ai/agent.ts ──▶ ai/observation.ts (盤面を数値の配列にする)
   │         └───────▶ ai/model.ts (ニューラルネットの計算)
   │
   └──▶ game/game.ts ─▶ game/field.ts ─▶ game/snake.ts   ← game/ (ルール。React を知らない)
                          game/rules.ts (秒をティック数に直す)
```

1 ティック (0.1 秒) ごとに次のことが起きます。

1. `GameScreen` のゲームループが「0.1 秒たった」と判断する ([11 章](#11-ゲームループ-requestanimationframe-と固定ティック))
2. AI の番なら `AiPlayer.decide` が盤面を数値にしてモデルに通し、行動を 1 つ決める ([14 章](#14-ai-モデルの読み込みと推論))
3. 人のキー入力 (たまっていたもの) と AI の行動を `GameEnv.step` に渡し、ゲームを 1 ティック進める ([10 章](#10-ゲームエンジン-srcgame))
4. React に「変わった」と知らせ、`Board` が Canvas に描き直し、`PlayerPanel` が数字を更新する ([12 章](#12-描画-canvas-と-dom-の使い分け))

### 1.4 Rust 側との関係

ゲームのルールは **Rust (learn/) と TypeScript (web/) に 2 回書いてあります**。

- 学習では、何百万試合も高速にシミュレートするので Rust で書いています。
- Web では、ブラウザで動かすので TypeScript で書いています。
- 2 つの実装は、同じルールで動くことをテストで確かめています ([17 章](#17-テスト-vitest))。

2 つをつなぐのは **モデルの JSON ファイル** だけです。JSON には重みのほかに、学習したときのルール (`game`) と観測の仕様 (`observation`) が入っています。
Web 側はこの JSON からルールの数値を読むので、盤面サイズや速度を変えて学習し直しても、Web 側のコードを変えずに済みます。

```jsonc
// ../model/recent-model/snake-model-16x16.json の先頭 (重みは長いので省略)
{
  "format": "duelsnake-dqn",
  "format_version": 1,
  "created_at": "2026-09-26T17:50:44.438081710+09:00",
  "training": { "games": 415733, "decisions": 83049366, "updates": 2594323 },
  "game": {
    "grid": { "width": 16, "height": 16, "tick_seconds": 0.1, "time_limit_seconds": 30.0 },
    "snake": { "initial_length": 3, "normal_speed_interval_sec": 0.2, ... },
    "items": { ... }
  },
  "observation": { "grid_channels": [...], "vector_features": [...], ... },
  "actions": ["up", "down", "left", "right", "boost", "use_item"],
  "difficulty": [{ "name": "easy", "random_action_rate": 0.6 }, ...],
  "network": { "conv": [...], "dense": [...] }
}
```

JSON の各キーの意味は [../learn/RULES.md の 11 章](../learn/RULES.md) にまとまっています。

---

## 2. 通信: 何がいつ送られるか

### 2.1 このアプリの通信はすべて「ファイルの GET」

このアプリの通信は、ブラウザが HTTP でファイルを取りに行く (GET リクエスト) ものだけです。
API サーバーへの問い合わせも、WebSocket によるリアルタイム通信もありません。

公開版 (https://24-fuji.github.io/duelsnake-ai/) を開いて、開発者ツールの Network タブを見ると、次の順に通信が起きます。

```
 ブラウザ                                          GitHub Pages
    │  GET /duelsnake-ai/                             │
    │ ──────────────────────────────────────────────▶ │
    │ ◀────────────── index.html (数百バイト) ─────── │
    │                                                 │
    │  index.html の <script> と <link> を見て          │
    │  GET ./assets/index-XXXX.js                     │
    │  GET ./assets/index-XXXX.css                    │
    │ ──────────────────────────────────────────────▶ │
    │ ◀──── JS (React とアプリのコード、約 180KB) ──── │
    │ ◀──── CSS ────────────────────────────────────── │
    │                                                 │
    │  JS が動き出し、React が画面を作る                 │
    │  Board が画像を読む: GET ./assets/normal_apple-XXXX.png など
    │  App がモデルを読む: GET ./assets/snake-model-16x16-XXXX.json (約 1.8MB)
    │ ──────────────────────────────────────────────▶ │
    │ ◀────────────── 画像・モデル JSON ────────────── │
    │                                                 │
    │  ここから先はゲーム中も通信なし                     │
    │  (盤面を 8x8 に切り替えたときだけ、8x8 のモデルを GET する)
```

ファイル名の `XXXX` はビルドのたびに中身から作られる文字列 (ハッシュ) です。理由は [4.4 節](#44-vite-開発サーバーとビルド) で説明します。

実際にビルドしてできたファイルは `web/dist/` にあります。次は、ある時点のビルド結果です。

```
web/dist/
├── index.html
└── assets/
    ├── index-Bo95ntBr.js               # 約 176KB: React + src/ の全コード
    ├── index-B06MtCSa.css              # 約 3KB: src/styles.css
    └── snake-model-16x16-BIUYl9Zr.json # 約 1.8MB: モデル (中身はそのままコピー)
```

`dist/index.html` はソースの [index.html](index.html) とほぼ同じですが、`<script src="/src/main.tsx">` がビルドした JS への参照に書き換わっています。

```html
<!-- dist/index.html (ビルド後) -->
<script type="module" crossorigin src="./assets/index-Bo95ntBr.js"></script>
<link rel="stylesheet" crossorigin href="./assets/index-B06MtCSa.css">
```

### 2.2 コードの中で通信している場所

アプリのコードの中で明示的に通信しているのは、**モデルを読み込む 1 か所だけ** です。

出典: [src/ai/model.ts:83-89](src/ai/model.ts#L83-L89)

```ts
static async load(url: string): Promise<SnakeModel> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`モデルを読み込めません: ${url} (${response.status})`);
  }
  return new SnakeModel((await response.json()) as ModelFile);
}
```

- `fetch(url)` はブラウザに組み込みの関数で、URL にリクエストを送り、返事 (`Response`) を **Promise** で返します。Promise は「あとで値が届く約束」で、`await` を付けると届くまで待てます ([7.8 節](#78-非同期-promise-と-asyncawait))。
- `response.ok` は、HTTP のステータスコードが 200〜299 (成功) なら `true` です。ファイルが無い (404) ときは `false` になるので、エラーにしています。`fetch` は 404 でも例外を投げない点に注意してください。例外になるのは、ネットワークがつながらないときなどです。
- `response.json()` は本文を JSON として読み、JavaScript のオブジェクトにします。これも Promise です。

画像は `fetch` ではなく `new Image()` に URL を入れることで、ブラウザが取りに行きます。

出典: [src/components/draw.ts:49-59](src/components/draw.ts#L49-L59)

```ts
export function loadBoardImages(): Promise<void> {
  spritesLoading ??= Promise.all(
    (Object.keys(SPRITE_URLS) as Sprite[]).map((sprite) => {
      const img = new Image();
      img.src = SPRITE_URLS[sprite];   // ここで GET が起きる
      sprites.set(sprite, img);
      return img.decode().catch(() => undefined);
    }),
  ).then(() => undefined);
  return spritesLoading;
}
```

### 2.3 URL はどこから来るのか

`fetch` に渡している URL は、ビルドツール (Vite) が作ったものです。

出典: [src/ai/catalog.ts:36-42](src/ai/catalog.ts#L36-L42)

```ts
export const BUNDLED_MODELS = bundledModels(
  import.meta.glob<string>("../../../model/recent-model/snake-model-*.json", {
    query: "?url",
    import: "default",
    eager: true,
  }),
);
```

`import.meta.glob` は Vite の機能で、**ビルドするときに** パターンに合うファイルを探し、次のようなオブジェクトに置き換えます。

```ts
// ビルド後の JS の中身のイメージ
{
  "../../../model/recent-model/snake-model-16x16.json": "./assets/snake-model-16x16-BIUYl9Zr.json",
  "../../../model/recent-model/snake-model-8x8.json":   "./assets/snake-model-8x8-XXXXXXXX.json",
}
```

- `query: "?url"` は「中身ではなく URL をください」という指定です。これが無いと JSON の中身 (1.8MB) が JS に埋め込まれ、使わない盤面のモデルまで最初に全部ダウンロードすることになります。
- `eager: true` は「今すぐ (同期的に) 値をください」という指定です。URL の文字列だけなので軽く、すぐ使えます。
- こうして、`model/recent-model/` に JSON を置くだけで選択肢が増える仕組みになっています。一覧を手で書く必要はありません。

画像も同じ仕組みで、`import` すると URL が手に入ります。

出典: [src/components/draw.ts:5-10](src/components/draw.ts#L5-L10)

```ts
import blockClearUrl from "../assets/items/block_clear.png";
import normalAppleUrl from "../assets/items/normal_apple.png";
// ...
```

画像は `?url` を付けなくても、Vite が URL の文字列として扱ってくれます (4KB 未満の小さい画像は、ファイルにせず JS に直接埋め込まれます)。

### 2.4 読み込みの競合を防ぐ

盤面を 16×16 から 8×8 に切り替えると、8×8 のモデルを取りに行きます。
もし 16×16 の読み込みが終わる前に切り替えたら、「あとから選んだ 8×8」より「先に選んだ 16×16」の結果が遅れて届き、画面が 16×16 に戻ってしまうかもしれません。これを **競合状態** (race condition) と呼びます。

App ではこれを防いでいます。

出典: [src/App.tsx:121-138](src/App.tsx#L121-L138)

```tsx
useEffect(() => {
  const entry = BUNDLED_MODELS.find((m) => m.key === board);
  if (!entry) return;
  // 読み込み中に別の盤面を選んだら、古いほうの結果は捨てる
  let cancelled = false;
  SnakeModel.load(entry.url)
    .then((model) => {
      if (cancelled) return;
      setCurrent(loaded(model));
      setError(null);
    })
    .catch((e: unknown) => {
      if (!cancelled) setError(`${entry.path} を読み込めません: ${String(e)}`);
    });
  return () => {
    cancelled = true;
  };
}, [board]);
```

`useEffect` から返した関数 (クリーンアップ) は、`board` が変わって次の effect が動く直前に呼ばれます。
そこで `cancelled = true` にしておけば、古いほうの読み込みが終わっても結果を捨てられます。`useEffect` の詳しい動きは [9.1 節](#91-useeffect-描画のあとに外の世界とやりとりする) で説明します。

### 2.5 キャッシュ

同じファイルを何度もダウンロードしないよう、ブラウザは受け取ったファイルを **キャッシュ** に保存します。
ビルドしたファイルの名前にハッシュが付いているのは、キャッシュとの相性のためです。

- 中身が変わればハッシュが変わり、ファイル名が変わる → ブラウザは新しいファイルとして取り直す
- 中身が同じならファイル名も同じ → キャッシュを使える

`index.html` だけはハッシュが付かないので、公開し直したあとに古い画面が出るときは、再読み込み (Ctrl+Shift+R など) を試してください。

---

## 3. 設計の選び方: どこで動かし、どう通信するか

この章では、「なぜこのアプリはブラウザだけで動かしたのか」と、「別のアプリならどう選ぶか」を考えます。
次に Web アプリを作るとき、最初に決めるのがこの 2 点です。

### 3.1 どこで計算するかの選択肢

| 構成 | 計算する場所 | 通信 | 向いているもの | 例 |
|---|---|---|---|---|
| **静的サイト + SPA** (このアプリ) | ブラウザ | ファイルの GET だけ | 1 人で完結する道具・ゲーム、公開してよいデータ | 電卓、オフラインのゲーム、このアプリ |
| SPA + API サーバー | ブラウザ + サーバー | HTTP で JSON をやりとり (REST など) | 保存・共有が要るもの、秘密を扱うもの | TODO アプリ、掲示板、ランキング |
| サーバーで HTML を作る (SSR) | 主にサーバー | ページごとに HTML を GET | 検索エンジンに載せたい記事、初回表示を速くしたいもの | ブログ、EC サイト |
| リアルタイムサーバー | ブラウザ + サーバー | WebSocket などで常時接続 | 複数人で同時に動くもの | オンライン対戦、チャット、共同編集 |

### 3.2 このアプリで静的サイトを選んだ理由

このアプリでは、次の条件がそろっていたので、サーバーを持たない構成を選びました。

1. **相手は AI で、同じブラウザの中にいる**
   人どうしのオンライン対戦ではないので、ほかの人の端末と状態を合わせる必要がありません。
2. **0.1 秒ごとに判断が要る**
   AI の推論をサーバーでやると、ティックごとに往復の通信が起きます。回線が遅いと 0.1 秒に間に合わず、ゲームが止まったり遅れたりします。ブラウザで計算すれば通信の遅れはゼロです。
3. **モデルが小さい**
   ネットワークは畳み込み 3 層と全結合 2 層で、JSON でも 1.8MB (8×8 なら 0.7MB) です。16×16 の盤面でも 1 回の推論は約 100 万回の掛け算で、スマホでも数ミリ秒で終わります。
4. **秘密が無い**
   モデルもルールも公開してよいものです。ブラウザに送ったものはすべて利用者に見えるので、秘密の API キーや、見せたくないモデルがあるなら、この構成は選べません。
5. **保存するデータがほとんど無い**
   保存するのは「パソコン版か携帯版か」の 1 つだけで、ブラウザの localStorage で足ります ([15 章](#15-ブラウザの機能-localstorage-と-history-api))。
6. **無料で公開できる**
   GitHub Pages はファイルを置くだけなら無料です。サーバーを持つと、動かし続ける費用と、落ちたときの対応が要ります。

### 3.3 サーバーが必要になるとき

逆に、次のどれかに当てはまるなら、サーバーを用意します。

| やりたいこと | 必要になるもの | 通信の方式 |
|---|---|---|
| 人どうしでオンライン対戦したい | 状態を持つゲームサーバー。不正を防ぐなら、サーバーでゲームを進めて結果だけを配る | WebSocket (または WebRTC) |
| ランキングを全員で共有したい | データベースと、書き込みを受け付ける API | HTTP (fetch で POST) |
| ログインさせたい | 認証の仕組み (自作せず、Firebase Auth などのサービスを使うことが多い) | HTTP |
| モデルを見せたくない、または大きすぎる | 推論サーバー (GPU など) | HTTP、ストリーミングなら SSE |
| 秘密の API キーを使いたい (外部の AI API など) | キーを持って代わりに呼ぶサーバー | HTTP |

通信の方式もまとめておきます。

| 方式 | 向き | 特徴 | 使う場面 |
|---|---|---|---|
| HTTP (fetch) | ブラウザ → サーバー → 返事 | 1 回ごとに問い合わせる。一番簡単 | ファイルの取得、データの保存・取得 |
| SSE (Server-Sent Events) | サーバー → ブラウザ | サーバーから一方的に流し続ける | 通知、AI の文章生成を少しずつ表示 |
| WebSocket | 双方向 | つなぎっぱなしにして、どちらからでもすぐ送れる | オンライン対戦、チャット |
| WebRTC | ブラウザ ↔ ブラウザ | サーバーを通さず端末どうしで直接つなぐ (最初の仲介だけサーバーが要る) | ビデオ通話、低遅延の対戦 |

### 3.4 推論をどう実装するかの選択肢

ブラウザでニューラルネットを動かす方法もいくつかあります。

| 方法 | 良いところ | 気をつけること |
|---|---|---|
| **自分でループを書く** (このアプリ) | 依存ゼロ。何をしているか全部見える。ファイルが小さい | 大きなモデルは遅い。層の種類を増やすたびに自分で書く |
| ONNX Runtime Web | PyTorch などで作ったモデルを変換してそのまま使える。WebAssembly や WebGPU で速い | ライブラリが数 MB ある。変換の手間 |
| TensorFlow.js | 学習もブラウザでできる。WebGL で速い | ライブラリが大きい。TensorFlow 形式に合わせる |
| WebAssembly に Rust をコンパイル | Rust の実装をそのまま使える (ルールの二重実装も消せる) | ビルドの仕組みが複雑になる。JS との受け渡しに慣れが要る |

このアプリのネットワークは小さく、層も畳み込みと全結合の 2 種類だけなので、自分で書くのが一番簡単でした ([14.4 節](#144-順伝播-畳み込みと全結合を手で書く))。
モデルが大きくなったり、層の種類が増えたりしたら、ONNX Runtime Web を検討するとよいでしょう。

### 3.5 判断のチェックリスト

次に Web アプリを作るときは、次の質問に順に答えると構成が決まります。

1. 複数の人の端末で、同じ状態を共有する必要があるか → あるならサーバー (リアルタイムなら WebSocket)
2. 秘密にしたいもの (キー・モデル・答え) があるか → あるならサーバー側に置く
3. 端末を変えても残したいデータがあるか → あるならデータベース。同じブラウザだけでよいなら localStorage
4. 計算は端末で間に合うか (スマホも含めて) → 間に合わないならサーバー、または WebAssembly・WebGPU で速くする
5. 応答の速さがどれくらい要るか → 1 秒に何回も要るなら、通信を挟まずブラウザで計算する
6. すべて「いいえ」なら、静的サイト + SPA が一番簡単で安い

---

## 4. ツールチェーン: Node.js・npm・Vite・TypeScript・Vitest

ブラウザは TypeScript や JSX (`<div>` のような HTML に似た書き方) をそのままでは実行できません。
そこで、開発する PC でいくつかの道具を使って、ブラウザが読める JavaScript に変換します。

| 道具 | 役割 | このリポジトリでの設定 |
|---|---|---|
| Node.js | ブラウザの外で JavaScript を動かす実行環境。以下の道具はすべて Node.js の上で動く | GitHub Actions では v22 ([../.github/workflows/deploy-web.yml:32](../.github/workflows/deploy-web.yml#L32)) |
| npm | パッケージ (ライブラリ) を入れる・スクリプトを実行する | [package.json](package.json), [package-lock.json](package-lock.json) |
| TypeScript (`tsc`) | 型の検査 | [tsconfig.json](tsconfig.json) |
| Vite | 開発サーバーと、公開用ファイルのビルド | [vite.config.js](vite.config.js) |
| Vitest | テストの実行 (Vite と同じ変換を使う) | `*.test.ts` |
| React | 画面を作るライブラリ。これだけは公開後もブラウザで動く | `dependencies` |

### 4.1 package.json

出典: [package.json](package.json)

```json
{
  "name": "duelsnake-web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "typecheck": "tsc",
    "test": "vitest run"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.2.0",
    "vite": "^5.0.0",
    "vitest": "^3.2.0"
  }
}
```

- `"private": true` は、npm に誤って公開しないための印です。
- `"type": "module"` は、`.js` ファイルを ES モジュール (`import` / `export` を使う形式) として扱う指定です。[vite.config.js](vite.config.js) で `import` が使えるのはこのためです。
- `scripts` は `npm run <名前>` で実行するコマンドです。`npm run dev` で `vite` が、`npm run build` で `tsc && vite build` が動きます (`npm test` だけは `run` を省けます)。
  - `build` で先に `tsc` を動かすのは、Vite は型を検査せずに変換だけするからです。型エラーがあればここで止まります。
- `dependencies` はブラウザで動くコードが使うもの、`devDependencies` は開発とビルドにだけ使うものです。このアプリが公開後に使うライブラリは **React だけ** です。
  - `@types/react` は React の型定義です。React 本体は JavaScript で書かれているので、型は別のパッケージで配られています。
- `^18.2.0` は「18.2.0 以上で、19 未満」という意味です。実際に入った版は [package-lock.json](package-lock.json) に固定されます。

`npm install` と `npm ci` の違いも覚えておきましょう。

| コマンド | 動き | 使う場面 |
|---|---|---|
| `npm install` | package.json に合う版を入れ、package-lock.json を更新することがある | 開発中。パッケージを追加するとき (`npm install <名前>`) |
| `npm ci` | package-lock.json のとおりに入れ直す。食い違えばエラー | CI (GitHub Actions)。毎回同じ版で試すため |

package-lock.json は必ず git にコミットします。`node_modules/` と `dist/` は、作り直せるのでコミットしません ([../.gitignore](../.gitignore))。

### 4.2 tsconfig.json

出典: [tsconfig.json](tsconfig.json)

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noEmit": true,
    "isolatedModules": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "types": ["vite/client"]
  },
  "include": ["src"]
}
```

| 設定 | 意味 |
|---|---|
| `target`, `lib` | どの版の JavaScript の機能と、どの API (ここでは `DOM` = ブラウザの API) を使えるとみなすか |
| `module`, `moduleResolution: "bundler"` | `import` の解決を Vite のようなバンドラーに合わせる |
| `jsx: "react-jsx"` | `.tsx` の中の `<div>` を React の関数呼び出しとして扱う |
| `strict` | 厳しい型検査をすべて有効にする。`null` かもしれない値をそのまま使うとエラーになる。**必ず有効にする** |
| `noEmit` | JS を出力しない。変換は Vite がするので、`tsc` は検査だけに使う |
| `isolatedModules` | ファイル 1 つずつ変換しても問題ない書き方を強制する (Vite はファイルごとに変換するため) |
| `resolveJsonModule` | `import x from "./a.json"` を許す。テストでモデルを読むのに使う ([src/game/game.test.ts:5](src/game/game.test.ts#L5)) |
| `noUnusedLocals`, `noUnusedParameters` | 使っていない変数・引数をエラーにする |
| `types: ["vite/client"]` | `import.meta.glob` や画像の `import` の型を使えるようにする |

### 4.3 vite.config.js

出典: [vite.config.js](vite.config.js)

```js
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  // GitHub Pages ではサブパス (/duelsnake-ai/) で配信されるので、アセットを相対パスで参照する
  base: './',
  server: {
    port: 3000,
    // リポジトリ直下の model/ にある学習済みモデルを読み込めるようにする
    fs: { allow: ['..'] },
  },
});
```

- `plugins: [react()]` で JSX の変換と、保存したときに状態を保ったまま画面を差し替える機能 (Fast Refresh) が有効になります。
- `base: './'` は、ビルドした HTML から JS や CSS を **相対パス** (`./assets/...`) で参照する指定です。
  GitHub Pages では `https://24-fuji.github.io/duelsnake-ai/` のように、ドメインの直下ではなく `/duelsnake-ai/` の下に置かれます。既定の `base: '/'` だと `/assets/...` (ドメインの直下) を見に行ってしまい、ファイルが見つかりません。
- `server.fs.allow: ['..']` は、開発サーバーが `web/` の外 (リポジトリ直下の `model/`) のファイルを配ってよいという許可です。Vite は安全のため、既定ではプロジェクトの外のファイルを配りません。

### 4.4 Vite: 開発サーバーとビルド

Vite には 2 つの顔があります。

**開発サーバー (`npm run dev`)**

- ブラウザが `/src/main.tsx` を要求すると、その場で TypeScript と JSX を JavaScript に変換して返します。
- ファイルは束ねず、1 ファイルずつ返します。ブラウザが `import` を見つけるたびに次のファイルを取りに行きます (Network タブを見ると、`App.tsx`、`GameScreen.tsx`… と別々に届いているのが分かります)。
- ファイルを保存すると、そのファイルだけを差し替えます (HMR: Hot Module Replacement)。

**ビルド (`npm run build`)**

- すべてのコードと React を 1 つ (か少数) の JS ファイルに束ねます (バンドル)。ファイル数が少ないほど、公開したときの読み込みが速くなります。
- 空白や長い変数名を縮めて小さくします (minify)。
- 画像やモデル JSON を `dist/assets/` にコピーし、ファイル名にハッシュを付けます ([2.5 節](#25-キャッシュ))。
- 使われていないコードを取り除きます (tree shaking)。

`npm run preview` を使うと、ビルドした `dist/` を手元で配信して確かめられます。公開前に「ビルドすると動かない」問題を見つけるのに使えます。

---

## 5. ディレクトリ構成と依存の向き

### 5.1 ファイル一覧

```
web/
├── index.html              # 最初に読まれる HTML。<div id="root"> と main.tsx の読み込みだけ
├── package.json            # 依存パッケージとスクリプト (4.1 節)
├── package-lock.json       # 入れたパッケージの正確な版
├── tsconfig.json           # TypeScript の設定 (4.2 節)
├── vite.config.js          # Vite の設定 (4.3 節)
└── src/
    ├── main.tsx            # 入口。React を #root に描く
    ├── App.tsx             # 設定欄・モデルの読み込み・画面の切り替え
    ├── styles.css          # 全体のスタイル
    ├── game/               # ゲームのルール。React もブラウザの API も使わない
    │   ├── rules.ts        #   モデル JSON の game → ティック単位のルール
    │   ├── snake.ts        #   ヘビ 1 匹 (向き・移動・伸び縮み・ブースト)
    │   ├── field.ts        #   1 人分の盤面 (ヘビ・アイテム・お邪魔ブロック)
    │   ├── game.ts         #   2 人対戦 1 試合 (入力の反映・勝敗)
    │   ├── solo.ts         #   一人モード (Web 版だけのルール)
    │   ├── game.test.ts    #   対戦のテスト (Rust のテストと同じ内容)
    │   └── solo.test.ts    #   一人モードのテスト
    ├── ai/                 # AI。React を使わない
    │   ├── catalog.ts      #   同梱モデルの一覧 (import.meta.glob)
    │   ├── model.ts        #   モデル JSON の読み込みと順伝播
    │   ├── observation.ts  #   盤面 → 入力の数値配列
    │   ├── agent.ts        #   AiPlayer: 観測 → 推論 → 行動
    │   └── agent.test.ts   #   同梱モデルで最後まで対戦できるかのテスト
    ├── components/         # 画面 (React) と、画面の手伝い
    │   ├── GameScreen.tsx  #   対戦画面。ゲームループ・入力・レイアウト
    │   ├── SoloScreen.tsx  #   一人モードの画面
    │   ├── RulesScreen.tsx #   遊ぶ人向けのルール画面
    │   ├── Board.tsx       #   盤面の <canvas>
    │   ├── draw.ts         #   Canvas への描き方 (React を使わない)
    │   ├── PlayerPanel.tsx #   スコア・所持アイテム・Q 値の表
    │   ├── gestures.ts     #   タッチ操作の見分け方と、React 用のフック
    │   └── gestures.test.ts
    └── assets/             # 画像
        ├── README.md
        └── items/*.png
```

### 5.2 依存の向きを一方向にする

このリポジトリで一番大事な設計は、**依存 (import) の向きが一方向になっている** ことです。

```
  components/, App.tsx   (React・DOM・Canvas を使う)
          │ import
          ▼
        ai/              (計算だけ。fetch 以外のブラウザ API を使わない)
          │ import
          ▼
        game/            (計算だけ。何にも依存しない)
```

- `game/` のファイルは、`react` も `document` も `window` も使っていません。確かめたければ `grep -rn "react\|document\|window" src/game/` を実行してください (何も出ません)。
- `ai/` は `game/` を使いますが、画面のことは知りません。
- 画面 (`components/`) は両方を使います。

こうしておくと、次の良いことがあります。

1. **テストしやすい**: `game/` と `ai/` は、ブラウザを立ち上げずに Node.js だけでテストできます ([17 章](#17-テスト-vitest))。
2. **Rust の実装と対応させやすい**: `game/` のファイルは `learn/src/env/` のファイルと 1 対 1 に対応しています (`snake.ts` ↔ `snake.rs`、`field.ts` ↔ `field.rs`、`game.ts` ↔ `game.rs`、`rules.ts` ↔ `rules.rs`、`observation.ts` ↔ `observation.rs`)。
3. **画面を作り直しやすい**: React をやめて別の仕組みにしても、`game/` と `ai/` はそのまま使えます。
4. **同じエンジンを使い回せる**: パソコン版と携帯版はどちらも同じ `GameEnv` を使い、見た目と入力だけが違います。

`components/` の中でも、同じ考え方をしています。

- [src/components/draw.ts](src/components/draw.ts) は React を使わず、「Canvas にどう描くか」だけを書いています。React の部品 [src/components/Board.tsx](src/components/Board.tsx) がそれを呼びます。
- [src/components/gestures.ts](src/components/gestures.ts) の `GestureRecognizer` クラスは React を使わず、「指の動きからジェスチャーを見分ける」だけです。その下の `useGestures` が React とつなぎます。テストは `GestureRecognizer` だけを相手にします ([src/components/gestures.test.ts](src/components/gestures.test.ts))。

**「計算するコード」と「画面に出すコード」を分ける** のは、規模が大きくなっても困らないための基本です。

---

## 6. ブラウザがページを表示するまで

URL を開いてからゲーム画面が出るまでを、実際のファイルで追いかけます。

### 6.1 index.html

出典: [index.html](index.html)

```html
<!doctype html>
<html lang="ja">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>DuelSnake AI</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- HTML にはほとんど何も書いていません。中身は、あとで JavaScript (React) が `<div id="root">` の中に作ります。これが SPA の特徴です。
- `<meta name="viewport" ...>` は、スマホで画面の幅に合わせて表示するための指定です。これが無いと、スマホはパソコン用の幅で描いてから縮めて見せるので、文字がとても小さくなります。
- `<script type="module">` は、ES モジュールとして読む指定です。モジュールの中では `import` が使えます。

### 6.2 main.tsx

出典: [src/main.tsx](src/main.tsx)

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- `document.getElementById("root")` で、HTML の `<div id="root">` を取り出します。見つからなければ `null` なので、末尾の `!` で「ここでは必ずある」と TypeScript に伝えています ([7.5 節](#75-null-と-undefined-の扱い))。
- `createRoot(...).render(<App />)` で、その `div` の中に `App` を描かせます。ここから先は React が DOM を管理します。
- `import "./styles.css"` は CSS を読み込む書き方です。Vite が `<link>` や `<style>` に変換します。
- `<StrictMode>` は開発中だけ動く検査機能です。わざと部品を 2 回作ったり、`useEffect` を「実行 → 片付け → 実行」と 2 回動かしたりして、片付け忘れのバグを見つけやすくします ([21 章](#21-よくあるハマりどころ))。公開版では何もしません。

### 6.3 App が最初に描くもの

`App` ([src/App.tsx](src/App.tsx)) は、最初はモデルを持っていません。起動の流れは次のとおりです。

1. `BUNDLED_MODELS` (モデルの URL の一覧) は、ファイルを読み込んだ時点でもうできています ([2.3 節](#23-url-はどこから来るのか))。
2. `App` の最初の描画: `current` (読み込んだモデル) が `null` なので「モデルを読み込み中...」と出します ([src/App.tsx:237-238](src/App.tsx#L237-L238))。
3. 描画のあと、`useEffect` がモデルの `fetch` を始めます ([src/App.tsx:121-138](src/App.tsx#L121-L138))。
4. 読み込みが終わると `setCurrent(...)` が呼ばれ、React が `App` を描き直します。今度は `current` があるので `GameScreen` を描きます ([src/App.tsx:248-257](src/App.tsx#L248-L257))。
5. `GameScreen` は「Enter キーで開始」の状態 (`ready`) で待ちます。

```tsx
// src/App.tsx:237-258 (要点)
{!current ? (
  !error && <p>モデルを読み込み中...</p>
) : mode === "solo" ? (
  <SoloScreen key={`${current.id}-solo-${soloTimeLimit}`} ... />
) : (
  <GameScreen key={`${current.id}-${mode}-${difficulty}`} ... />
)}
```

`key` の役割は [8.6 節](#86-key-で部品を作り直す) で説明します。

---

## 7. TypeScript の基礎をこのリポジトリで学ぶ

TypeScript は、JavaScript に **型** を足した言語です。型は「この変数にはどんな値が入るか」の約束で、約束を破るとエディタとビルド (`tsc`) がエラーを出します。
実行するときは型を消して、ただの JavaScript になります。型は実行を速くするものではなく、**間違いを実行前に見つけるためのもの** です。

### 7.1 変数: `const` と `let`

```ts
const tickMs = (rules.tickSeconds * 1000) / speed;  // 再代入しない
let last = performance.now();                       // 再代入する
```

出典: [src/components/GameScreen.tsx:107-110](src/components/GameScreen.tsx#L107-L110)

- 基本は `const` を使い、あとで値を入れ替えるものだけ `let` にします。`var` は使いません。
- `const` でも、オブジェクトや配列の **中身** は変えられます (`const a = []; a.push(1)` は OK)。変えられないのは「別の値を入れ直すこと」です。

### 7.2 基本の型と、型の推論

```ts
let pending = 0;          // number と推論される
const names = ["あなた", "AI"];  // string[] と推論される
```

TypeScript は、多くの場合型を自分で推論します。関数の引数と、公開する関数の戻り値にだけ型を書くのが普通です。

| 型 | 例 |
|---|---|
| `number` | `0.1`, `16` (整数と小数の区別は無い) |
| `string` | `"up"` |
| `boolean` | `true` |
| `T[]` | `Position[]` (配列) |
| `[A, B]` | `[Field, Field]` (長さと各要素の型が決まった配列 = タプル) |
| `null` / `undefined` | 値が無いこと |
| `unknown` | 何か分からない値。使う前に型を確かめる必要がある |

### 7.3 interface と type: データの形を決める

**interface** は、オブジェクトがどんなプロパティを持つかの約束です。

出典: [src/game/snake.ts:4-7](src/game/snake.ts#L4-L7)

```ts
/** 盤面上の座標。左上が (0, 0) で、x は右、y は下に向かって増える */
export interface Position {
  x: number;
  y: number;
}
```

モデル JSON の形も interface で書いています。JSON を読んだ結果に型を付けると、`file.network.conv[0].kernel_size` のような長い参照でも、エディタが候補を出し、打ち間違いを教えてくれます。

出典: [src/ai/model.ts:46-56](src/ai/model.ts#L46-L56)

```ts
export interface ModelFile {
  format: string;
  format_version: number;
  created_at: string;
  training: { games: number; decisions: number; updates: number };
  game: GameRules;
  observation: ObservationSpec;
  actions: Action[];
  difficulty: Difficulty[];
  network: { conv: ConvLayer[]; dense: DenseLayer[] };
}
```

注意: 型は **実行時には確かめられません**。`(await response.json()) as ModelFile` ([src/ai/model.ts:88](src/ai/model.ts#L88)) の `as` は「これは ModelFile だと信じて」という指示で、中身が違っても通ってしまいます。
そのため、`SnakeModel` のコンストラクタでは `format` や重みの要素数を自分で確かめています ([src/ai/model.ts:91-122](src/ai/model.ts#L91-L122))。**外から来たデータは、型を付けるだけでなく中身も確かめる** のが鉄則です。

### 7.4 文字列リテラル型と union 型: 取りうる値を限る

出典: [src/game/snake.ts:9](src/game/snake.ts#L9), [src/game/game.ts:7](src/game/game.ts#L7)

```ts
export type Direction = "up" | "down" | "left" | "right";
export type Action = "up" | "down" | "left" | "right" | "boost" | "use_item";
```

`|` は「どれか 1 つ」を表します (union 型)。`Direction` 型の変数には、この 4 つの文字列しか入れられません。`"UP"` や `"upp"` と打ち間違えると、実行前にエラーになります。
他の言語の enum に近い使い方です。JSON にそのまま書ける文字列なので、Rust 側と受け渡すのにも便利です。

形の違うデータをまとめるときは、共通のプロパティ (ここでは `kind`) で見分けます。これを **判別可能な union** (discriminated union) と呼びます。

出典: [src/components/gestures.ts:9-13](src/components/gestures.ts#L9-L13)

```ts
export type Gesture =
  | { kind: "swipe"; direction: Direction }
  | { kind: "long_press" }
  | { kind: "tap" }
  | { kind: "double_tap" };
```

`if (gesture.kind === "swipe")` の中では、TypeScript が「ここでは swipe だ」と分かるので `gesture.direction` を使えます ([src/components/GameScreen.tsx:178](src/components/GameScreen.tsx#L178))。
`kind` を確かめずに `gesture.direction` を使うとエラーになります。Rust の `enum` と `match` に近い考え方です。

### 7.5 null と undefined の扱い

`strict` を有効にすると、`null` かもしれない値をそのまま使えなくなります。確かめ方がいくつかあります。

| 書き方 | 意味 | 例 |
|---|---|---|
| `if (x) ...` / `if (!x) return` | 無ければ先に抜ける | [src/App.tsx:123](src/App.tsx#L123) `if (!entry) return;` |
| `x?.y` | x が null/undefined なら undefined、あれば x.y | [src/App.tsx:140](src/App.tsx#L140) `current?.model.info` |
| `x ?? y` | x が null/undefined なら y | [src/App.tsx:141](src/App.tsx#L141) `info?.difficulty ?? []` |
| `x ??= y` | x が null/undefined のときだけ y を入れる | [src/components/draw.ts:50](src/components/draw.ts#L50) `spritesLoading ??= ...` |
| `x!` | 「必ずある」と言い切る (確かめない) | [src/main.tsx:6](src/main.tsx#L6) `getElementById("root")!` |

`!` は TypeScript の検査を黙らせるだけなので、本当に必ずあるときにだけ使います。

`??` と `||` の違いにも気をつけてください。`||` は `0` や `""` も「無い」とみなします。
[src/components/Board.tsx:48](src/components/Board.tsx#L48) の `window.devicePixelRatio || 1` は、0 のときも 1 にしたいので `||` にしています。

### 7.6 Record と readonly と as const

`Record<K, V>` は「キーが K、値が V のオブジェクト」です。キーを union 型にすると、**全部のキーを書かないとエラー** になるので、書き忘れを防げます。

出典: [src/game/snake.ts:14-19](src/game/snake.ts#L14-L19)

```ts
const DELTA: Record<Direction, Position> = {
  up: { x: 0, y: -1 },
  down: { x: 0, y: 1 },
  left: { x: -1, y: 0 },
  right: { x: 1, y: 0 },
};
```

`Direction` に 5 つ目の向きを足すと、ここがエラーになって直し忘れに気づけます。
アイテムの画像の対応 ([src/components/draw.ts:32-39](src/components/draw.ts#L32-L39)) や、行動の表示名 ([src/components/PlayerPanel.tsx:7-14](src/components/PlayerPanel.tsx#L7-L14)) も同じ書き方です。

`readonly` は「書き換えない」という約束です。

出典: [src/game/game.ts:9](src/game/game.ts#L9)

```ts
export const ACTIONS: readonly Action[] = ["up", "down", "left", "right", "boost", "use_item"];
```

`as const` は、値をそのまま型にします。観測のチャネル名 ([src/ai/observation.ts:10-20](src/ai/observation.ts#L10-L20)) は `as const` を付けているので、型は `string[]` ではなく `readonly ["own_head", "own_body", ...]` になります。

### 7.7 class: データと操作をまとめる

ゲームの状態は class で書いています。

出典: [src/game/snake.ts:43-79](src/game/snake.ts#L43-L79) (抜粋)

```ts
export class Snake {
  /** 先頭が頭 */
  body: Position[];
  dir: Direction = "up";
  lastMovedDir: Direction = "up";
  alive = true;
  score = 0;
  moveCooldown: number;

  constructor(head: Position, rules: Rules) {
    this.body = Array.from({ length: rules.initialLength }, (_, i) => ({ x: head.x, y: head.y + i }));
    this.moveCooldown = rules.normalIntervalTicks;
  }

  get head(): Position {
    return this.body[0];
  }

  get length(): number {
    return this.body.length;
  }

  /** 直前の進行方向と真逆の入力は無視する */
  turn(dir: Direction): void {
    if (dir !== OPPOSITE[this.lastMovedDir]) {
      this.dir = dir;
    }
  }
}
```

- `dir: Direction = "up";` のように、プロパティに初期値を書けます。
- `constructor` は `new Snake(...)` のときに呼ばれます。
- `get head()` は **getter** で、`snake.head` とプロパティのように読めますが、中身は関数です。`body[0]` をいちいち書かずに済みます。
- `private` を付けたものは、クラスの外から使えません ([src/game/game.ts:71](src/game/game.ts#L71) の `applyAction` など)。`readonly` を付けたものは、コンストラクタの外で書き換えられません。
- `static` は、インスタンスではなくクラスそのものに付く関数です。`SnakeModel.load(url)` ([src/ai/model.ts:83](src/ai/model.ts#L83)) は `new` する前に呼ぶので `static` にしています。

React の画面は関数で書き、ゲームの状態は class で書く、という使い分けをしています。
ゲームの状態は 0.1 秒ごとに少しずつ書き換えるので、書き換えやすい class が向いています。

### 7.8 非同期: Promise と async/await

ネットワークや画像の読み込みは時間がかかります。JavaScript は、待っている間も画面を止めないよう、**非同期** に処理します。

```ts
// then でつなぐ書き方 (src/App.tsx:126-134)
SnakeModel.load(entry.url)
  .then((model) => { /* 成功したとき */ })
  .catch((e: unknown) => { /* 失敗したとき */ });

// async/await の書き方 (src/ai/model.ts:83-89)
static async load(url: string): Promise<SnakeModel> {
  const response = await fetch(url);   // 届くまでこの関数の続きを待つ (画面は止まらない)
  ...
}
```

- `async` を付けた関数は、必ず Promise を返します。
- `await` は、Promise の値が届くまで **その関数の続きだけ** を待ちます。その間、ブラウザは他の仕事 (描画やキー入力) を続けます。
- `Promise.all([...])` は、いくつもの Promise を同時に待ちます。画像 6 枚を並行して読むのに使っています ([src/components/draw.ts:50](src/components/draw.ts#L50))。
- `useEffect` に渡す関数は async にできないので、App では `.then` でつないでいます。

### 7.9 import と export

```ts
import { useCallback, useEffect } from "react";     // パッケージから名前を指定して読む
import { Board } from "./Board";                      // 同じフォルダのファイルから読む
import type { SnakeModel } from "../ai/model";        // 型だけを読む
import { GameEnv, type Action } from "../game/game";  // 値と型を一緒に読む
import normalAppleUrl from "../assets/items/normal_apple.png";  // default を読む (Vite が URL にする)
```

- `export` を付けたものだけが、他のファイルから `import` できます。付けていないものはそのファイルの中だけで使えます (Rust の `pub` と同じです)。
- `import type` は型だけを読む指定で、変換後の JS からは消えます。`isolatedModules` を有効にしているので、型だけを使うときは `type` を付けます。
- このリポジトリでは、`export default` はほとんど使わず、名前付きの `export` を使っています。名前が揃うので検索しやすくなります。

### 7.10 配列の操作

配列のメソッドは頻繁に使うので、よく出るものをまとめます。

| メソッド | 何をするか | このリポジトリの例 |
|---|---|---|
| `map` | 各要素を変換した新しい配列 | [src/game/game.ts:99](src/game/game.ts#L99) `this.fields.map((f) => f.isFilled(this.rules))` |
| `filter` | 条件に合う要素だけの新しい配列 | [src/game/field.ts:91](src/game/field.ts#L91) 頭から遠いマスだけ残す |
| `find` / `findIndex` | 最初に条件に合う要素 / その位置 | [src/game/field.ts:139](src/game/field.ts#L139) 頭の位置にあるアイテムを探す |
| `some` / `every` | どれか 1 つ / 全部が条件に合うか | [src/game/field.ts:62](src/game/field.ts#L62) お邪魔ブロックがあるか |
| `forEach` | 各要素で何かする (戻り値なし) | [src/components/GameScreen.tsx:93](src/components/GameScreen.tsx#L93) |
| `flatMap` | 変換して 1 段平らにする。0 個か 1 個を返せば filter と map を同時にできる | [src/ai/catalog.ts:28-32](src/ai/catalog.ts#L28-L32) |
| `splice(0)` | 全要素を取り出して、元の配列を空にする | [src/components/GameScreen.tsx:100](src/components/GameScreen.tsx#L100) たまったキー入力を取り出す |
| `unshift` / `pop` | 先頭に足す / 末尾を取り除く | [src/game/snake.ts:129-133](src/game/snake.ts#L129-L133) ヘビの前進 |

`...` (スプレッド構文) は、配列やオブジェクトを展開します。

出典: [src/game/solo.ts:15-21](src/game/solo.ts#L15-L21)

```ts
export function soloRules(rules: Rules): Rules {
  return {
    ...rules,                                        // rules の全部をコピーして
    poisonApples: rules.poisonApples * SOLO_POISON_MULTIPLIER,  // 2 つだけ上書きする
    poisonApple: { ...rules.poisonApple, grow: SOLO_POISON_GROW },
  };
}
```

元の `rules` は書き換えず、新しいオブジェクトを作っています。これを **イミュータブル** (不変) な更新と呼び、React の state を更新するときに必須の書き方です ([8.4 節](#84-state-変わる値を持つ))。

### 7.11 型付き配列: Float32Array

AI の計算では、普通の配列 (`number[]`) ではなく `Float32Array` を使っています。

出典: [src/ai/model.ts:108](src/ai/model.ts#L108)

```ts
weight: Float32Array.from(layer.weight),
```

- `Float32Array` は 32 ビットの小数だけを並べた配列です。普通の配列より、メモリが少なくて計算が速くなります。
- 長さはあとから変えられません。最初に `new Float32Array(長さ)` で作ると、全部 0 で埋まっています。
- 学習側 (Rust) も `f32` で計算しているので、結果が揃いやすくなります。
- 盤面のマスが空いているかの印には `Uint8Array` (0〜255 の整数の配列) を使っています ([src/game/field.ts:68](src/game/field.ts#L68))。

---

## 8. React の基礎: コンポーネント・props・state

### 8.1 React が解決すること

ブラウザの画面は DOM (Document Object Model) という木構造で、JavaScript から直接書き換えられます。

```js
// React を使わない書き方
const p = document.createElement("p");
p.textContent = `スコア ${score}`;
document.body.appendChild(p);
// score が変わったら、自分で p.textContent を書き換える必要がある
```

画面が大きくなると、「どの値が変わったら、どこを書き換えるか」を全部自分で管理するのは大変です。
React では、**「今の値なら画面はこうなる」** という関数を書くだけです。値が変わると React が関数を呼び直し、前回との差分だけを DOM に反映します。

### 8.2 コンポーネントと props

**コンポーネント** は、画面の部品を返す関数です。名前は大文字で始めます。
**props** は、親から子に渡す引数です。

出典: [src/components/PlayerPanel.tsx:18-30](src/components/PlayerPanel.tsx#L18-L30) (抜粋)

```tsx
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
```

- props の型を `interface Props` で決め、引数で分割代入しています。`?` を付けたものは省略できます。
- `heldItems = true` のように、省略したときの値を書けます。
- 親は HTML のタグのように呼びます。

出典: [src/components/GameScreen.tsx:198-207](src/components/GameScreen.tsx#L198-L207)

```tsx
<PlayerPanel
  title={names[player]}
  field={env.fields[player]}
  rules={rules}
  palette={PLAYER_PALETTES[player]}
  decision={ais[player] ? decisionsRef.current[player] : undefined}
  actions={model.info.actions}
/>
```

`{}` の中には JavaScript の式を書けます。文字列だけなら `title="あなた"` とも書けます ([src/components/SoloScreen.tsx:175](src/components/SoloScreen.tsx#L175))。

### 8.3 JSX の書き方

`.tsx` ファイルの中の `<div>...</div>` は **JSX** で、HTML に似ていますがいくつか違いがあります。

| HTML | JSX | 理由 |
|---|---|---|
| `class="app"` | `className="app"` | `class` は JavaScript の予約語 |
| `onclick="..."` | `onClick={関数}` | 文字列ではなく関数を渡す |
| `style="width: 10px"` | `style={{ width: 10 }}` | オブジェクトで渡す。外側の `{}` は式、内側の `{}` はオブジェクト |
| `<input>` | `<input />` | 閉じタグが無い要素は `/>` で閉じる |

条件によって出し分けるときは、`&&` と三項演算子 `? :` を使います。

出典: [src/App.tsx:233-234](src/App.tsx#L233-L234)

```tsx
{info && <p className="model-info">最終更新: {new Date(info.created_at).toLocaleString()}</p>}
{error && <p className="error">{error}</p>}
```

`info` が無ければ何も出ません。注意点として、左側が `0` だと `0` が画面に出てしまうので、数値を条件にするときは `count > 0 && ...` と書きます。

配列から要素を並べるときは `map` を使い、各要素に `key` を付けます。

出典: [src/App.tsx:154-160](src/App.tsx#L154-L160)

```tsx
<select value={board ?? ""} onChange={(e) => setBoard(e.target.value)}>
  {BUNDLED_MODELS.map((m) => (
    <option key={m.key} value={m.key}>
      {m.width} × {m.height}
    </option>
  ))}
</select>
```

`key` は、描き直したときに「どの要素が前回のどれにあたるか」を React が見分けるための印です。配列の中で重ならない値にします。

`<>...</>` (Fragment) は、余計な `div` を作らずに要素をまとめる書き方です ([src/components/PlayerPanel.tsx:52-57](src/components/PlayerPanel.tsx#L52-L57))。

### 8.4 state: 変わる値を持つ

**state** は、コンポーネントが覚えておく値で、変わると画面が描き直されます。`useState` で作ります。

出典: [src/App.tsx:64-71](src/App.tsx#L64-L71)

```tsx
const [mode, setMode] = useState<PlayMode>("human_vs_ai");
const [difficulty, setDifficulty] = useState("hard");
const [speed, setSpeed] = useState(1);
const [autoRestart, setAutoRestart] = useState(true);
const [layout, setLayout] = useState(initialLayout);
```

- `useState(初期値)` は `[今の値, 値を変える関数]` を返します。
- `setMode("ai_vs_ai")` を呼ぶと、React が `App` を呼び直し、今度は `mode` が `"ai_vs_ai"` になっています。
- `useState(initialLayout)` のように **関数そのもの** を渡すと、最初の 1 回だけその関数を呼んで初期値にします。`initialLayout` は localStorage を読むので、毎回呼ばないようにしています ([src/App.tsx:32-40](src/App.tsx#L32-L40))。

前の値から次の値を作るときは、関数を渡します。

出典: [src/components/GameScreen.tsx:125-129](src/components/GameScreen.tsx#L125-L129)

```tsx
setTally((t) =>
  winner === null
    ? { ...t, draws: t.draws + 1 }
    : { ...t, wins: t.wins.map((n, p) => (p === winner ? n + 1 : n)) },
);
```

- `t` は、その時点での最新の値です。
- `t.draws += 1` のように **中身を書き換えてはいけません**。React は「オブジェクトが別物になったか」で変化を判断するので、同じオブジェクトの中身を変えても気づきません。必ず `{ ...t, draws: ... }` のように新しいオブジェクトを作ります。

### 8.5 入力欄を state とつなぐ (制御されたコンポーネント)

出典: [src/App.tsx:216-224](src/App.tsx#L216-L224)

```tsx
<input
  type="checkbox"
  checked={autoRestart}
  disabled={mode !== "ai_vs_ai"}
  onChange={(e) => setAutoRestart(e.target.checked)}
/>
```

- 表示する値 (`checked`) を state から渡し、変わったら (`onChange`) state を更新します。
- こうすると「画面の値」と「state」が常に一致します。これを **制御されたコンポーネント** (controlled component) と呼びます。
- `disabled` のように、他の state から見た目を決めることもできます。

### 8.6 key で部品を作り直す

`key` は、リストの要素だけでなく、1 つの部品にも付けられます。**key が変わると、React はその部品を捨てて新しく作り直します**。中の state もすべて初期値に戻ります。

出典: [src/App.tsx:248-257](src/App.tsx#L248-L257)

```tsx
<GameScreen
  key={`${current.id}-${mode}-${difficulty}`}
  model={current.model}
  mode={mode}
  ...
/>
```

モデル・モード・難易度のどれかが変わると `key` が変わり、`GameScreen` が新しく作られます。すると、試合の途中の状態や勝敗の数がきれいに消え、新しい設定で最初から始まります。
「props が変わったら state を全部リセットしたい」ときの定番の方法です。

一方、速度 (`speed`) は `key` に入れていません。速度を変えても試合を続けたいからです。

`current.id` は、モデルを読み込むたびに増える番号です ([src/App.tsx:48-53](src/App.tsx#L48-L53))。同じ盤面を読み直しても、画面を作り直せるようにしています。

### 8.7 画面を消さずに隠す

ルール画面を開いている間、ゲーム画面は消さずに `hidden` で隠しています。

出典: [src/App.tsx:146-148](src/App.tsx#L146-L148)

```tsx
{showRules && current && <RulesScreen game={current.model.info.game} onBack={closeRules} />}
{/* ルール画面の間もゲーム画面は消さずに隠し、戻ったら続きから遊べるようにする */}
<div hidden={showRules}>
```

`{showRules ? <RulesScreen/> : <GameScreen/>}` と書くと、ルール画面を開いた時点で `GameScreen` が消え、試合の状態も消えます。
消さずに隠せば、state は残ったままなので、戻ると続きから遊べます。
その代わり、隠れている間も動き続けないよう、`active` という props で止めています ([src/components/GameScreen.tsx:72-76](src/components/GameScreen.tsx#L72-L76))。

---

## 9. React のフック: useEffect・useRef・useMemo・useCallback

`use` で始まる関数を **フック** (hook) と呼びます。フックには 2 つの決まりがあります。

1. コンポーネントの一番外側で呼ぶ。`if` や `for` の中、途中で `return` したあとで呼ばない。
   (React は、フックを「呼ばれた順番」で見分けているため)
2. コンポーネントか、他のフックの中でだけ呼ぶ。

### 9.1 useEffect: 描画のあとに外の世界とやりとりする

コンポーネントの関数は「画面をどう描くか」を返すだけにして、それ以外のこと (通信・タイマー・イベントの登録・Canvas への描画) は `useEffect` の中でします。

```tsx
useEffect(() => {
  // 描画のあとに実行される
  return () => {
    // 次に実行する前と、部品が消えるときに実行される (クリーンアップ)
  };
}, [依存する値]);  // この値が変わったときだけ実行し直す
```

| 第 2 引数 | いつ実行するか |
|---|---|
| `[]` | 最初の描画のあとに 1 回だけ |
| `[a, b]` | 最初と、`a` か `b` が変わったとき |
| 省略 | 毎回の描画のあと (ほとんど使わない) |

このリポジトリの `useEffect` を種類ごとに見てみます。

**イベントの登録と解除**

出典: [src/components/GameScreen.tsx:145-167](src/components/GameScreen.tsx#L145-L167) (要点)

```tsx
useEffect(() => {
  const onKeyDown = (e: KeyboardEvent) => { ... };
  window.addEventListener("keydown", onKeyDown);
  return () => window.removeEventListener("keydown", onKeyDown);
}, [start, ais]);
```

`addEventListener` したら、必ずクリーンアップで `removeEventListener` します。忘れると、画面を作り直すたびに登録が増え、1 回のキー入力が何回も処理されます。

**タイマー**

出典: [src/components/GameScreen.tsx:139-143](src/components/GameScreen.tsx#L139-L143)

```tsx
useEffect(() => {
  if (!active || status !== "over" || !autoRestart || mode !== "ai_vs_ai") return;
  const timer = setTimeout(start, AUTO_RESTART_DELAY_MS);
  return () => clearTimeout(timer);
}, [active, status, autoRestart, mode, start]);
```

AI どうしの試合が終わったら、1.5 秒後に次の試合を始めます。
待っている間に「自動で次の試合」のチェックを外すと、`autoRestart` が変わってクリーンアップが動き、`clearTimeout` でタイマーが取り消されます。

**監視 (ResizeObserver)**

出典: [src/components/Board.tsx:38-44](src/components/Board.tsx#L38-L44)

```tsx
useEffect(() => {
  const canvas = canvasRef.current;
  if (!fill || !canvas) return;
  const observer = new ResizeObserver(([entry]) => setFillWidth(entry.contentRect.width));
  observer.observe(canvas);
  return () => observer.disconnect();
}, [fill]);
```

**通信** は [2.4 節](#24-読み込みの競合を防ぐ)、**ゲームループ** は [11 章](#11-ゲームループ-requestanimationframe-と固定ティック)、**Canvas への描画** は [12 章](#12-描画-canvas-と-dom-の使い分け) で見ます。

### 9.2 useRef: 描き直しを起こさない入れ物

`useRef` は、**変えても描き直しが起きない** 値の入れ物です。`ref.current` で読み書きします。コンポーネントが作り直されるまで、値は残り続けます。
このリポジトリでは 3 つの使い方をしています。

**(1) DOM の要素をつかむ**

出典: [src/components/Board.tsx:26](src/components/Board.tsx#L26), [src/components/Board.tsx:57-59](src/components/Board.tsx#L57-L59)

```tsx
const canvasRef = useRef<HTMLCanvasElement>(null);
// ...
return <canvas ref={canvasRef} ... />;
```

`ref={canvasRef}` と書くと、React が描画のあとに `canvasRef.current` に `<canvas>` の DOM 要素を入れてくれます。Canvas に描くにはこの要素が要ります。

**(2) ゲームの状態を持つ**

出典: [src/components/GameScreen.tsx:57-60](src/components/GameScreen.tsx#L57-L60)

```tsx
const envRef = useRef<GameEnv | null>(null);
if (envRef.current === null) envRef.current = new GameEnv(rules);
const decisionsRef = useRef<(Decision | null)[]>([null, null]);
const inputsRef = useRef<Action[]>([]);
```

ゲームの状態 (`GameEnv`) は、state ではなく ref に入れています。これがこのアプリの重要な設計です。

- `GameEnv` は 0.1 秒ごとに **中身を書き換えて** 進みます。state は中身を書き換えてはいけない ([8.4 節](#84-state-変わる値を持つ)) ので、state に入れるには毎ティック全体をコピーする必要があり、無駄が多くなります。
- ref なら書き換え放題です。描き直しは、代わりに `version` という数字の state を増やして起こします ([11.3 節](#113-描き直しのきっかけ-version))。
- `if (envRef.current === null) envRef.current = new GameEnv(rules);` は、「最初の 1 回だけ作る」書き方です。`useRef(new GameEnv(rules))` と書くと、描き直すたびに `new GameEnv` が呼ばれて (捨てられて) しまいます。
- `inputsRef` は、キー入力を次のティックまでためておく場所です。キーを押すたびに描き直す必要はないので ref にしています。

**(3) イベントハンドラーから最新の state を読む**

出典: [src/components/GameScreen.tsx:61-70](src/components/GameScreen.tsx#L61-L70)

```tsx
const [status, setStatus] = useState<Status>("ready");
const statusRef = useRef(status);
// ...
useEffect(() => {
  statusRef.current = status;
}, [status]);
```

`status` を state と ref の両方に持っています。理由は **古いクロージャ** (stale closure) の問題です。

- 関数は、作られたときの変数を覚えています (クロージャ)。
- キー入力のハンドラーを作ったときの `status` が `"ready"` なら、そのハンドラーの中の `status` はずっと `"ready"` のままです。
- ref は、どの時点で作った関数からも同じ入れ物 (`statusRef`) を見るので、`statusRef.current` は常に最新です。

キーのハンドラーは `statusRef.current` を読んでいます ([src/components/GameScreen.tsx:149](src/components/GameScreen.tsx#L149))。
こうすると、`status` が変わるたびにハンドラーを登録し直さずに済みます。`active` も同じ理由で `activeRef` を持っています ([src/components/GameScreen.tsx:72-76](src/components/GameScreen.tsx#L72-L76))。

### 9.3 useMemo: 重い計算や、作り直したくないものを覚える

出典: [src/components/GameScreen.tsx:47-54](src/components/GameScreen.tsx#L47-L54)

```tsx
const rules = useMemo(() => rulesFromConfig(model.info.game), [model]);
const ais = useMemo(
  () =>
    mode === "human_vs_ai"
      ? [null, new AiPlayer(model, randomActionRate)]
      : [new AiPlayer(model, randomActionRate), new AiPlayer(model, randomActionRate)],
  [model, mode, randomActionRate],
);
```

`useMemo(関数, [依存])` は、依存が変わったときだけ関数を呼び直し、それ以外は前回の結果を返します。

- `rules` は毎回計算しても軽いですが、**毎回別のオブジェクトになる** と、`rules` に依存する `useEffect` や `useCallback` が毎回動いてしまいます。`useMemo` で同じオブジェクトを保つのが主な目的です。
- `AiPlayer` は入力用の `Float32Array` を中に持っているので、毎回作り直さないようにしています。

### 9.4 useCallback: 関数を覚える

`useCallback(関数, [依存])` は `useMemo(() => 関数, [依存])` と同じで、関数を覚えておきます。

出典: [src/components/GameScreen.tsx:78-88](src/components/GameScreen.tsx#L78-L88)

```tsx
const start = useCallback(() => {
  if (envRef.current?.isOver) {
    envRef.current = new GameEnv(rules);
    decisionsRef.current = [null, null];
  }
  inputsRef.current = [];
  (document.activeElement as HTMLElement | null)?.blur?.();
  setStatus("running");
  setVersion((v) => v + 1);
}, [rules]);
```

`start` は、キーのハンドラーの `useEffect` や、自動で次の試合を始める `useEffect` の依存に入っています。
`useCallback` を使わないと、描き直すたびに新しい `start` ができて、それらの `useEffect` が毎回やり直されてしまいます。

**useMemo と useCallback は、「依存配列に入れる値」を安定させるために使う** と覚えておくと迷いません。

### 9.5 自作のフック

`use` で始まる関数を自分で作ると、フックを組み合わせた処理を使い回せます。

出典: [src/components/gestures.ts:118-138](src/components/gestures.ts#L118-L138)

```tsx
export function useGestures(onGesture: (gesture: Gesture) => void) {
  const handler = useRef(onGesture);
  useEffect(() => {
    handler.current = onGesture;
  }, [onGesture]);
  const recognizer = useMemo(() => new GestureRecognizer((g) => handler.current(g)), []);
  useEffect(() => () => recognizer.release(), [recognizer]);

  return {
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      e.currentTarget.setPointerCapture(e.pointerId);
      recognizer.down(e.pointerId, e.clientX, e.clientY, e.timeStamp);
    },
    onPointerMove: ...,
    onPointerUp: ...,
    onPointerCancel: ...,
    onContextMenu: (e: { preventDefault(): void }) => e.preventDefault(),
  };
}
```

- `GestureRecognizer` は 1 回だけ作り (`useMemo` の依存が `[]`)、コールバックは ref 経由で最新のものを呼びます ([9.2 節](#92-useref-描き直しを起こさない入れ物) の (3) と同じ考え方)。
- 部品が消えるときに `recognizer.release()` で長押しのタイマーを止めます。
- 戻り値はイベントハンドラーの詰め合わせで、使う側は `<div {...gestures}>` と展開するだけです ([src/components/GameScreen.tsx:243](src/components/GameScreen.tsx#L243))。
- 対戦画面と一人モードの画面の両方で使っています。

### 9.6 どのフックを使うかの早見表

| やりたいこと | 使うもの |
|---|---|
| 変わったら画面に出したい値 | `useState` |
| 変わっても描き直さなくてよい値、DOM 要素、毎フレーム書き換える状態 | `useRef` |
| 通信・タイマー・イベント登録・Canvas への描画 | `useEffect` (必ずクリーンアップを書く) |
| 他のフックの依存に入れるオブジェクトを安定させたい | `useMemo` |
| 他のフックの依存に入れる関数を安定させたい | `useCallback` |
| 他の値から計算できる値 | フックを使わず、普通に計算する (例: [src/App.tsx:140-142](src/App.tsx#L140-L142)) |

最後の行は大事です。「`difficulty` が変わったら `randomActionRate` を state に入れ直す」のような書き方はせず、描くたびに計算します。state を増やすほど、値が食い違うバグが増えます。

---

## 10. ゲームエンジン: src/game/

この章では、React とは関係の無い「ルールのコード」を読みます。ゲームを作るときに、画面より先に作る部分です。

### 10.1 秒ではなくティックで数える

ゲームの時間は、秒ではなく **ティック** (1 回の更新) で数えます。このアプリでは 1 ティック = 0.1 秒です。
モデル JSON のルールは秒で書いてあるので、最初にティック数に直します。

出典: [src/game/rules.ts:60-73](src/game/rules.ts#L60-L73) (抜粋)

```ts
export function rulesFromConfig(cfg: GameRules): Rules {
  const tick = cfg.grid.tick_seconds;
  // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
  const ticks = (sec: number) => Math.max(0, Math.round(sec / tick));

  return {
    width: cfg.grid.width,
    height: cfg.grid.height,
    tickSeconds: tick,
    timeLimitTicks: Math.max(1, ticks(cfg.grid.time_limit_seconds)),
    normalIntervalTicks: Math.max(1, ticks(cfg.snake.normal_speed_interval_sec)),
    // ...
  };
}
```

ティックで数える理由は次のとおりです。

- **結果が端末の速さに左右されない**: 速い PC でも遅いスマホでも、同じ入力なら同じ結果になります。
- **テストしやすい**: 「3 ティック進めたら頭はここ」と、時間を待たずに確かめられます。
- **学習と同じにできる**: Rust 側も同じティックで進むので、AI が学習した環境と同じ条件で遊べます。

コメントにあるとおり、小数の割り算には誤差があります (`0.6 / 0.1` は `5.999999999999999`)。`Math.floor` で切り捨てると 5 になってしまうので、`Math.round` で四捨五入しています。
Rust 側 ([../learn/src/env/rules.rs](../learn/src/env/rules.rs)) も同じ計算をしています。

### 10.2 3 つのクラスの役割分担

| クラス | ファイル | 持っているもの |
|---|---|---|
| `Snake` | [src/game/snake.ts](src/game/snake.ts) | 体の座標・向き・スコア・所持アイテム・ブーストのタイマー |
| `Field` | [src/game/field.ts](src/game/field.ts) | ヘビ 1 匹と、アイテム・お邪魔ブロック・再出現待ちの一覧 (1 人分の盤面) |
| `GameEnv` | [src/game/game.ts](src/game/game.ts) | `Field` 2 つ、経過ティック、勝敗 (1 試合) |

小さい部品から順に組み立てています。`Snake` は盤面を知らず、`Field` は相手を知らず、`GameEnv` だけが 2 人の関係 (お邪魔を送る・勝敗) を扱います。

### 10.3 1 ティックの処理

出典: [src/game/game.ts:54-69](src/game/game.ts#L54-L69)

```ts
step(inputs: PlayerInputs): void {
  if (this.isOver) return;
  this.tick += 1;

  for (const field of this.fields) {
    field.processPending(this.rules, this.pick);   // 1. アイテムとお邪魔ブロックの出現
  }
  inputs.forEach((actions, player) => {
    for (const action of actions) this.applyAction(player, action);  // 2. 入力を反映
  });
  for (const field of this.fields) {
    field.updateSnake(this.rules);                 // 3. ヘビを動かし、衝突とアイテムを判定
  }

  this.result = this.judge();                      // 4. 勝敗
}
```

この順番は [../learn/RULES.md の 8 章](../learn/RULES.md) で決めていて、Rust 側 ([../learn/src/env/game.rs](../learn/src/env/game.rs)) も同じ順番です。順番が 1 つでも違うと、同じ入力でも結果が変わります。

`inputs` は「このティックに反映する入力」で、プレイヤーごとに **押した順に並べた配列** です。

- 人は、前のティックからの間にキーを何回も押すことがあります (「左 → 上」とすばやく押すなど)。全部を順に反映するので、すばやい操作も取りこぼしません。
- AI は、移動する直前のティックに 1 つだけ行動を決めます ([src/game/game.ts:46-48](src/game/game.ts#L46-L48) の `needsDecision`)。

### 10.4 ヘビの動き

ヘビは毎ティック動くわけではありません。通常は 2 ティックに 1 回、ブースト中は毎ティック動きます。

出典: [src/game/snake.ts:97-113](src/game/snake.ts#L97-L113)

```ts
advanceTimers(rules: Rules): boolean {
  if (this.boostRemaining > 0) {
    this.boostRemaining -= 1;
    if (this.boostRemaining === 0) {
      this.boostCooldown = rules.boostCooldownTicks;
    }
  } else if (this.boostCooldown > 0) {
    this.boostCooldown -= 1;
  }

  this.moveCooldown -= 1;
  if (this.moveCooldown > 0) {
    return false;
  }
  this.moveCooldown = this.boostRemaining > 0 ? rules.boostIntervalTicks : rules.normalIntervalTicks;
  return true;
}
```

「残りティック数を 1 ずつ減らし、0 になったら何かする」という **カウントダウン** は、ゲームで一番よく使う形です。ブーストの持続・再使用までの待ち・アイテムの再出現・お邪魔ブロックが届くまでの時間、すべてこの形で書いています。

進む向きには `dir` と `lastMovedDir` の 2 つがあります ([src/game/snake.ts:46-49](src/game/snake.ts#L46-L49))。

- `dir`: 次に進む向き。入力で変わる。
- `lastMovedDir`: 最後に実際に進んだ向き。

真逆への入力を無視する判定は `lastMovedDir` で行います ([src/game/snake.ts:75-79](src/game/snake.ts#L75-L79))。
`dir` で判定すると、上に進んでいるときに「左 → 下」とすばやく押したとき、「左」で `dir` が左になり、「下」は左の真逆ではないので通ってしまい、次の移動で自分の首にぶつかります。

### 10.5 乱数を差し替えられるようにする

アイテムの出現位置はランダムですが、テストでは決まった位置に出したいことがあります。そこで、「位置の選び方」を関数として外から渡せるようにしています。

出典: [src/game/field.ts:16-20](src/game/field.ts#L16-L20)

```ts
/** 出現位置の選び方。通常はランダムに1つ選ぶ (テストでは決まった位置を返す) */
export type Picker = (candidates: Position[], what: ItemType | "obstacle") => Position | undefined;

export const randomPicker: Picker = (candidates) =>
  candidates.length > 0 ? candidates[Math.floor(Math.random() * candidates.length)] : undefined;
```

`GameEnv` のコンストラクタは `pick: Picker = randomPicker` と、省略したらランダムになるようにしています ([src/game/game.ts:31](src/game/game.ts#L31))。
このように、**外から部品を渡せるようにしておくこと** を依存性の注入 (dependency injection) と呼びます。ランダム・時刻・通信など、テストで困るものは外から渡せるようにしておくと、テストが書きやすくなります。

### 10.6 一人モード: 既存の部品を使い回す

一人モードは Web 版だけのルールですが、新しく書いたのは [src/game/solo.ts](src/game/solo.ts) の 100 行ほどだけです。

- ルールは `soloRules` で対戦のルールを少し書き換えて作ります ([src/game/solo.ts:15-21](src/game/solo.ts#L15-L21))。
- 盤面は同じ `Field` を使い、コンストラクタの第 3 引数 `heldItems = false` で所持アイテムを出さないようにしています ([src/game/field.ts:39-50](src/game/field.ts#L39-L50))。
- `SoloEnv.step` は、`GameEnv.step` と同じ順番で `Field` のメソッドを呼ぶだけです ([src/game/solo.ts:74-89](src/game/solo.ts#L74-L89))。

**ルールの違いを「設定の違い」として表せないか** をまず考えると、コードの重複が減ります。

---

## 11. ゲームループ: requestAnimationFrame と固定ティック

### 11.1 ゲームループとは

ゲームは「少し進める → 描く」を繰り返して動きます。この繰り返しを **ゲームループ** と呼びます。
ブラウザでは、`requestAnimationFrame` (rAF) という関数でループを作ります。

- `requestAnimationFrame(frame)` は「次に画面を描き直す直前に `frame` を呼んで」という予約です。
- 多くの画面では 1 秒に 60 回 (約 16.7 ミリ秒ごと) 呼ばれます。120Hz の画面なら 120 回です。
- タブが裏に回ると呼ばれなくなるので、見ていない間の無駄な計算が止まります。

`setInterval(tick, 100)` でも 0.1 秒ごとに動かせますが、次の問題があります。

- タイマーは正確ではなく、重い処理があると遅れます。遅れはそのまま積み重なります。
- 画面の描き直しとタイミングが合わず、動きがカクつくことがあります。

### 11.2 経過時間をためて、決まった間隔で進める

このアプリのループは、「前のフレームからの経過時間をためておき、0.1 秒分たまるごとに 1 ティック進める」という形です。

出典: [src/components/GameScreen.tsx:104-137](src/components/GameScreen.tsx#L104-L137)

```tsx
// ゲームループ: 経過時間に応じて決まった間隔でティックを進める
useEffect(() => {
  if (status !== "running") return;
  const tickMs = (rules.tickSeconds * 1000) / speed;
  let last = performance.now();
  let pending = 0;
  let frameId = 0;

  const frame = (now: number) => {
    pending += Math.min(now - last, 250);
    last = now;
    const env = envRef.current!;
    let ticked = false;
    while (pending >= tickMs && !env.isOver) {
      pending -= tickMs;
      tick();
      ticked = true;
    }
    if (ticked) setVersion((v) => v + 1);
    if (env.result) {
      // 勝敗を数えて、終わりの状態にする
      setTally(...);
      setStatus("over");
      return;
    }
    frameId = requestAnimationFrame(frame);
  };
  frameId = requestAnimationFrame(frame);
  return () => cancelAnimationFrame(frameId);
}, [status, speed, rules, tick]);
```

1 行ずつ見ていきます。

- `if (status !== "running") return;`: 走っているときだけループを動かします。一時停止すると `status` が変わり、クリーンアップの `cancelAnimationFrame` でループが止まります。
- `tickMs`: 1 ティックの長さ (ミリ秒)。観戦の速度が 2 倍なら半分の 50 ミリ秒です。
- `performance.now()`: ページを開いてからのミリ秒。`Date.now()` より細かく、時計の変更の影響を受けません。rAF のコールバックにも同じ基準の時刻 (`now`) が渡されます。
- `pending += now - last`: 前のフレームからの経過時間をためます。
- `while (pending >= tickMs)`: 1 ティック分以上たまっていれば進めます。60Hz の画面なら、だいたい 6 フレームに 1 回進みます。速度 4 倍 (25 ミリ秒) なら、1 フレームで 0〜1 回進みます。
- `Math.min(now - last, 250)`: 経過時間は最大 250 ミリ秒までにしています。タブを裏に回して戻ったときに、何秒分ものティックを一気に進めてしまわないためです。
- `if (ticked) setVersion(...)`: 進んだときだけ描き直しを頼みます ([11.3 節](#113-描き直しのきっかけ-version))。
- ループを続けるときは、最後にもう一度 `requestAnimationFrame(frame)` を予約します。予約しなければ止まります。

この形は **固定タイムステップ** (fixed timestep) と呼ばれる、ゲームの定番の書き方です。
画面のフレームレートが変わっても、ゲームの進み方 (1 秒に 10 ティック) は変わりません。

### 11.3 描き直しのきっかけ: version

ゲームの状態は ref に入れている ([9.2 節](#92-useref-描き直しを起こさない入れ物)) ので、書き換えても React は描き直しません。
そこで、`version` という数字の state を用意し、ティックが進むたびに 1 増やしています。

```tsx
const [version, setVersion] = useState(0);   // src/components/GameScreen.tsx:63
```

- `setVersion(v => v + 1)` を呼ぶと `GameScreen` が描き直され、`envRef.current` の最新の状態が画面に出ます。
- `Board` にも `version` を渡しています。`Board` の中の Canvas に描く `useEffect` は `version` を依存に入れているので、ティックごとに描き直されます ([src/components/Board.tsx:50-55](src/components/Board.tsx#L50-L55))。
  `field` は同じオブジェクトのまま中身だけが変わるので、`field` を依存にしても変化に気づけません。`version` が「中身が変わった」という合図になっています。

### 11.4 1 ティックの中身

出典: [src/components/GameScreen.tsx:90-102](src/components/GameScreen.tsx#L90-L102)

```tsx
const tick = useCallback(() => {
  const env = envRef.current!;
  const inputs: [Action[], Action[]] = [[], []];
  ais.forEach((ai, player) => {
    if (ai && env.needsDecision(player)) {
      const decision = ai.decide(env, player);
      decisionsRef.current[player] = decision;
      inputs[player].push(decision.action);
    }
  });
  if (!ais[0]) inputs[0] = inputsRef.current.splice(0);
  env.step(inputs);
}, [ais]);
```

- `ais` は `[null, AiPlayer]` (あなた vs AI) か `[AiPlayer, AiPlayer]` (観戦) です。`null` のところは人が操作します。
- AI は、次のティックで動くときだけ推論します。推論は重めなので、毎ティックはしません。
- 判断の結果 (Q 値) は `decisionsRef` に残し、`PlayerPanel` で表示します。
- 人の入力は、キーのハンドラーが `inputsRef` にためておいたものを `splice(0)` で全部取り出して渡します。

### 11.5 状態の移り変わり

画面には 4 つの状態があり、次のように移ります。

```
            Enter / タップ / 開始ボタン
  ready ─────────────────────────────▶ running ◀──────┐
                                       │    ▲          │ P / Esc / 再開ボタン
                           P / Esc     │    │          │
                        (ルール画面も)  ▼    │          │
                                      paused ─────────┘
                                       
  running ── 勝敗がつく ──▶ over ── Enter / タップ / もう一度 (観戦なら 1.5 秒後に自動) ──▶ running
```

`type Status = "ready" | "running" | "paused" | "over";` ([src/components/GameScreen.tsx:14](src/components/GameScreen.tsx#L14)) のように、状態を union 型の 1 つの値で持つのがコツです。
`isRunning`・`isPaused`・`isOver` と真偽値を 3 つ持つと、「走っていて、かつ終わっている」のようなありえない組み合わせが作れてしまいます。

---

## 12. 描画: Canvas と DOM の使い分け

### 12.1 2 種類の描き方

このアプリは、画面の部分によって描き方を変えています。

| 部分 | 描き方 | 理由 |
|---|---|---|
| 盤面 | `<canvas>` に JavaScript で描く | 256 マスを 0.1 秒ごとに描き直す。1 マスずつ DOM 要素にすると数が多く、重くなる |
| スコア・Q 値の表・設定欄・ルール画面 | React (DOM) | 文字や表が中心。選択・コピー・読み上げ・レイアウトはブラウザに任せたい |

**Canvas** は、ピクセルの絵を描くための要素です。描いたものは絵でしかないので、「どこに何があるか」をブラウザは知りません (クリックされた場所の判定なども自分で書く必要があります)。その代わり、たくさんの図形を速く描けます。
**DOM** は、要素の木です。文字の折り返しやレイアウトをブラウザがやってくれますが、要素が数千になると重くなります。

目安として、**毎フレーム変わる大量の図形なら Canvas、文字や操作部品なら DOM** です。

### 12.2 Board: React と Canvas をつなぐ

出典: [src/components/Board.tsx:25-69](src/components/Board.tsx#L25-L69) (抜粋)

```tsx
export function Board({ field, rules, palette, version, cellSize, fill = false }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [fillWidth, setFillWidth] = useState(0);
  const [imagesLoaded, setImagesLoaded] = useState(false);

  // (画像の読み込みと ResizeObserver は省略)

  const cell = fill ? fillWidth / rules.width : (cellSize ?? defaultCellSize(rules));
  // 高解像度の画面でもぼやけないよう、画面の画素数に合わせて描く
  const scale = window.devicePixelRatio || 1;

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx || cell <= 0) return;
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    drawField(ctx, field, rules, cell, palette);
  }, [field, rules, palette, version, cell, scale, imagesLoaded]);

  return (
    <canvas
      ref={canvasRef}
      className={fill ? "board fill" : "board"}
      width={Math.round(rules.width * cell * scale)}
      height={Math.round(rules.height * cell * scale)}
      style={fill ? { aspectRatio: `${rules.width} / ${rules.height}` } : { width: rules.width * cell, height: rules.height * cell }}
    />
  );
}
```

ポイントは次の 4 つです。

1. **React は `<canvas>` 要素を作るだけで、中身は `useEffect` で描く**
   `getContext("2d")` で描画用の道具 (`CanvasRenderingContext2D`) を取り出し、`drawField` に渡します。
2. **高解像度の画面への対応 (`devicePixelRatio`)**
   スマホや Retina の画面では、CSS の 1 ピクセルが実際の画素 2〜3 個にあたります。Canvas の `width`/`height` (画素数) を CSS の大きさと同じにすると、引き伸ばされてぼやけます。
   そこで、画素数を `scale` 倍にし、`setTransform(scale, ...)` で座標も `scale` 倍にしています。描く側 (`drawField`) は CSS ピクセルのつもりで描けば済みます。
3. **携帯版は幅いっぱいに広げる (`fill`)**
   CSS で幅を 100% にし ([src/styles.css:330-334](src/styles.css#L330-L334))、`ResizeObserver` で実際の幅を測って、1 マスの大きさを決めています ([src/components/Board.tsx:38-44](src/components/Board.tsx#L38-L44))。画面を回転させたときも測り直されます。
4. **画像が読み込み終わったら描き直す (`imagesLoaded`)**
   画像はすぐには届かないので、届くまではアイテムを描かずに進めます。届いたら `imagesLoaded` を `true` にして描き直します ([src/components/Board.tsx:30-36](src/components/Board.tsx#L30-L36))。
   `active` という変数で、部品が消えたあとに state を更新しないようにしているのは、[2.4 節](#24-読み込みの競合を防ぐ) の `cancelled` と同じ考え方です。

### 12.3 drawField: Canvas への描き方

出典: [src/components/draw.ts:61-87](src/components/draw.ts#L61-L87)

```ts
export function drawField(ctx, field, rules, cell, palette): void {
  const w = rules.width * cell;
  const h = rules.height * cell;
  ctx.fillStyle = BACKGROUND;
  ctx.fillRect(0, 0, w, h);                 // 1. 背景で全体を塗りつぶす (前のフレームの絵を消す)
  ctx.strokeStyle = GRID_LINE;
  ctx.lineWidth = 1;
  for (let i = 1; i < rules.width; i++) line(ctx, i * cell + 0.5, 0, i * cell + 0.5, h);  // 2. マス目の線
  for (let i = 1; i < rules.height; i++) line(ctx, 0, i * cell + 0.5, w, i * cell + 0.5);

  ctx.imageSmoothingQuality = "high";
  for (const p of field.obstacles) drawSprite(ctx, "obstacle", p.x * cell, p.y * cell, cell);  // 3. お邪魔ブロック
  for (const item of field.items) drawSprite(ctx, item.kind, item.pos.x * cell, item.pos.y * cell, cell);  // 4. アイテム

  drawSnake(ctx, field, cell, palette);     // 5. ヘビ (一番上に描く)
}
```

- Canvas は **あとから描いたものが上に重なります**。背景 → 線 → アイテム → ヘビの順に描いています。
- 毎回、全体を塗りつぶしてから描き直します。前の絵の一部だけを消すより簡単で、この大きさなら十分速いです。
- `+ 0.5` は、1 ピクセルの線をくっきり描くための小技です。座標が整数だと、線が 2 つの画素の境目にかかって、半分ずつの濃さでぼやけます。
- マス `(x, y)` の左上のピクセルは `(x * cell, y * cell)` です。盤面の座標とピクセルの変換はこれだけです。

よく使う Canvas の命令をまとめます。

| 命令 | 何をするか |
|---|---|
| `ctx.fillStyle = "#色"` / `ctx.fillRect(x, y, w, h)` | 塗りの色を決める / 四角を塗る |
| `ctx.strokeStyle`, `ctx.lineWidth` / `beginPath` `moveTo` `lineTo` `stroke` | 線の色と太さ / 線を引く ([src/components/draw.ts:89-94](src/components/draw.ts#L89-L94)) |
| `ctx.arc(cx, cy, r, 0, Math.PI * 2)` / `ctx.fill()` | 円 (ヘビの目) ([src/components/draw.ts:129-131](src/components/draw.ts#L129-L131)) |
| `ctx.drawImage(img, x, y, w, h)` | 画像を描く ([src/components/draw.ts:100](src/components/draw.ts#L100)) |
| `ctx.globalAlpha = 0.5` | 以降の描画の透明度 (尻尾に向かって薄くする) ([src/components/draw.ts:115](src/components/draw.ts#L115)) |
| `ctx.setTransform(...)` | 座標の拡大・移動 |

### 12.4 画像の読み込みを 1 回にまとめる

盤面は 2 つあり、それぞれの `Board` が `loadBoardImages()` を呼びますが、画像を読むのは 1 回だけです。

出典: [src/components/draw.ts:45-59](src/components/draw.ts#L45-L59)

```ts
const sprites = new Map<Sprite, HTMLImageElement>();
let spritesLoading: Promise<void> | undefined;

export function loadBoardImages(): Promise<void> {
  spritesLoading ??= Promise.all(...);
  return spritesLoading;
}
```

- `spritesLoading` はファイルの一番外側 (モジュールのトップレベル) にある変数なので、アプリ全体で 1 つだけです。
- 最初の呼び出しで Promise を作って入れ、2 回目以降は同じ Promise を返します。
- `img.decode()` は、画像を描ける状態になるまで待つ Promise です。失敗しても `catch` で握りつぶし、画像なしで遊べるようにしています。

### 12.5 DOM で描く部分: Q 値の表

AI の判断は表と棒グラフで出しています。棒の長さは CSS の幅で表しています。

出典: [src/components/PlayerPanel.tsx:76-98](src/components/PlayerPanel.tsx#L76-L98) (抜粋)

```tsx
const q = Array.from(decision.qValues);
const min = Math.min(...q);
const range = Math.max(...q) - min || 1;
// ...
<tr key={action} className={action === decision.action ? "chosen" : undefined}>
  <td>{ACTION_LABELS[action]}</td>
  <td className="bar-cell">
    <div className="bar" style={{ width: `${8 + (92 * (q[i] - min)) / range}%` }} />
  </td>
  <td className="num">{q[i].toFixed(3)}</td>
</tr>
```

- 一番小さい Q 値を 8%、一番大きい Q 値を 100% の幅にしています。
- `|| 1` は、全部の Q 値が同じで `range` が 0 のときに 0 で割らないためです。
- 選んだ行動の行には `chosen` クラスを付けて、CSS で色を変えています ([src/styles.css:167-183](src/styles.css#L167-L183))。

小さなグラフなら、ライブラリを入れなくても DOM と CSS で十分描けます。

---

## 13. 入力: キーボードとタッチ

### 13.1 キーボード

出典: [src/components/GameScreen.tsx:145-167](src/components/GameScreen.tsx#L145-L167)

```tsx
useEffect(() => {
  const onKeyDown = (e: KeyboardEvent) => {
    if (!activeRef.current) return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;
    const current = statusRef.current;
    if (e.code === "Enter") {
      e.preventDefault();
      if (current !== "running") start();
      return;
    }
    if (e.code === "KeyP" || e.code === "Escape") {
      if (current === "running") setStatus("paused");
      else if (current === "paused") setStatus("running");
      return;
    }
    const action = KEY_ACTIONS[e.code];
    if (!action || ais[0]) return;
    e.preventDefault();
    if (!e.repeat && current === "running") inputsRef.current.push(action);
  };
  window.addEventListener("keydown", onKeyDown);
  return () => window.removeEventListener("keydown", onKeyDown);
}, [start, ais]);
```

| 書き方 | 理由 |
|---|---|
| `window.addEventListener("keydown", ...)` | ページのどこにフォーカスがあってもキーを受け取れるよう、`window` で受ける |
| `e.code` (`"KeyW"`, `"ArrowUp"`) | キーボード上の **物理的な位置** で判定する。`e.key` (`"w"`) は、日本語入力中や配列の違いで変わる |
| `e.target instanceof HTMLInputElement` | 設定欄 (スライダーやセレクト) を操作しているときは、ゲームに渡さない |
| `e.preventDefault()` | ブラウザの既定の動き (矢印キーやスペースでページがスクロールする) を止める |
| `e.repeat` | キーを押しっぱなしにしたときの連続入力を無視する |
| `inputsRef.current.push(action)` | すぐにゲームを動かさず、次のティックまでためておく ([11.4 節](#114-1-ティックの中身)) |

キーと行動の対応は、表 (`KEY_ACTIONS`) にしておくと、キーを増やすのも一人モードで使い回すのも簡単です ([src/components/GameScreen.tsx:16-29](src/components/GameScreen.tsx#L16-L29)、[src/components/SoloScreen.tsx:134](src/components/SoloScreen.tsx#L134))。

**フォーカスの問題**にも気をつけています。セレクトやボタンをクリックすると、その要素にフォーカスが残ります。そのまま矢印キーを押すと、ゲームではなくセレクトの値が変わってしまいます。
そこで、試合を始めるときにフォーカスを外しています ([src/components/GameScreen.tsx:84-85](src/components/GameScreen.tsx#L84-L85))。

```tsx
// 設定欄などにフォーカスが残っていると矢印キーやスペースを奪われるので外す
(document.activeElement as HTMLElement | null)?.blur?.();
```

一人モードの制限時間のスライダーでも、マウスを離したときにフォーカスを外しています ([src/App.tsx:181-182](src/App.tsx#L181-L182))。

### 13.2 タッチ: Pointer Events

スマホの操作は、スワイプ (向きを変える)・長押し (ブースト)・ダブルタップ (アイテム)・タップ (開始) の 4 つです。
ブラウザは「指が触れた・動いた・離れた」というイベントしか教えてくれないので、ジェスチャーは自分で見分けます。

イベントには **Pointer Events** (`pointerdown` / `pointermove` / `pointerup` / `pointercancel`) を使っています。
マウス・指・ペンを同じイベントで扱えるので、パソコンのマウスでも携帯版を試せます。

見分ける処理は、React を使わないクラスにしています。

出典: [src/components/gestures.ts:50-102](src/components/gestures.ts#L50-L102) (抜粋)

```ts
down(id: number, x: number, y: number, time: number): void {
  if (this.touch) return;  // 2本目以降の指は無視する
  const touch: Touch = { id, x, y, swiped: false, longPressed: false, lastDirection: null,
                         followsTap: time - this.lastTapEnd <= this.options.doubleTapMs };
  this.touch = touch;
  this.timer = setTimeout(() => {
    if (this.touch !== touch || touch.swiped) return;
    touch.longPressed = true;
    this.emit({ kind: "long_press" });
  }, this.options.longPressMs);
}

move(id: number, x: number, y: number): void {
  // ...
  if (Math.max(Math.abs(dx), Math.abs(dy)) < this.options.swipeDistance) return;
  const direction = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? "right" : "left") : dy > 0 ? "down" : "up";
  touch.swiped = true;
  clearTimeout(this.timer);
  // 指を離さずに続けて曲がれるよう、基準を今の位置に移す
  touch.x = x;
  touch.y = y;
  if (direction !== touch.lastDirection) {
    touch.lastDirection = direction;
    this.emit({ kind: "swipe", direction });
  }
}

up(id: number, time: number): void {
  // ...
  if (touch.swiped || touch.longPressed) { /* 何もしない */ }
  else if (touch.followsTap) this.emit({ kind: "double_tap" });
  else this.emit({ kind: "tap" });
}
```

- **スワイプ**: 触れた位置から 24px 以上動いたら、縦と横の大きいほうの向きをスワイプとみなします。基準を今の位置に移すので、指を離さずに「右 → 上」と続けて曲がれます。
- **長押し**: 触れたときにタイマーを仕掛け、350 ミリ秒たっても動いていなければ長押しです。動いたらタイマーを取り消します。
- **タップとダブルタップ**: 離したとき、動いてもおらず長押しでもなければタップです。前のタップから 300 ミリ秒以内に触れ始めていればダブルタップです。
- しきい値 (24px・350ms・300ms) は `DEFAULT_GESTURE_OPTIONS` にまとめています ([src/components/gestures.ts:24](src/components/gestures.ts#L24))。

React とつなぐ `useGestures` ([9.5 節](#95-自作のフック)) には、もう 2 つの工夫があります。

- `setPointerCapture(e.pointerId)`: 指が盤面の外に出ても、その指のイベントを盤面に届け続けてもらいます ([src/components/gestures.ts:129](src/components/gestures.ts#L129))。
- `onContextMenu` で `preventDefault`: 長押しでブラウザのメニューが出ないようにします。

CSS 側でも、ブラウザがタッチを横取りしないようにしています。

出典: [src/styles.css:320-328](src/styles.css#L320-L328)

```css
.stage {
  position: relative;
  margin-bottom: 8px;
  /* スワイプや長押しで、画面のスクロール・拡大・文字の選択・メニューが起きないようにする */
  touch-action: none;
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
}
```

`touch-action: none` が無いと、盤面の上で下にスワイプしたときにページがスクロールしてしまい、`pointercancel` が来てジェスチャーが途切れます。

### 13.3 パソコン版と携帯版をどう決めるか

出典: [src/App.tsx:31-40](src/App.tsx#L31-L40)

```ts
/** 前に選んだ表示があればそれを、無ければ指で操作する端末かどうかで決める */
function initialLayout(): Layout {
  try {
    const saved = localStorage.getItem(LAYOUT_KEY);
    if (saved === "pc" || saved === "mobile") return saved;
  } catch {
    // 保存できない環境では毎回判定する
  }
  return window.matchMedia("(pointer: coarse)").matches ? "mobile" : "pc";
}
```

- `matchMedia("(pointer: coarse)")` は、「主な入力が指のように粗いか」を CSS のメディアクエリで確かめます。画面の幅ではなく **操作方法** で決めているのがポイントです。大きなタブレットでも指で操作するなら携帯版になります。
- 判定が外れることもあるので、画面の「表示」でいつでも切り替えられ、選んだものは localStorage に保存します ([15.1 節](#151-localstorage-ブラウザに少しだけ保存する))。

---

## 14. AI: モデルの読み込みと推論

### 14.1 全体の流れ

```
 モデル JSON ──fetch──▶ SnakeModel (重みを Float32Array に)
                              │
 GameEnv ──encodeObservation──▶ grid (9×16×16) と vector (13) ──predict──▶ Q 値 (6)
                                                                            │
                                                         最大の行動を選ぶ (難易度に応じてランダム)
                                                                            │
                                                                            ▼
                                                               "up" / "boost" / ...
```

| 段階 | ファイル |
|---|---|
| モデルの一覧 | [src/ai/catalog.ts](src/ai/catalog.ts) |
| 読み込みと形式の確認 | [src/ai/model.ts:83-122](src/ai/model.ts#L83-L122) |
| ルールと観測仕様の一致を確かめる | [src/ai/agent.ts:15-17](src/ai/agent.ts#L15-L17), [src/ai/observation.ts:51-58](src/ai/observation.ts#L51-L58) |
| 盤面を数値にする (観測) | [src/ai/observation.ts:61-105](src/ai/observation.ts#L61-L105) |
| 順伝播 (推論) | [src/ai/model.ts:136-158](src/ai/model.ts#L136-L158) |
| 行動を選ぶ | [src/ai/agent.ts:34-47](src/ai/agent.ts#L34-L47) |

### 14.2 読み込みと確認

`SnakeModel` のコンストラクタは、JSON を受け取って次のことをします ([src/ai/model.ts:91-122](src/ai/model.ts#L91-L122))。

1. `format` と `format_version` が期待どおりか確かめる。違えば例外。
2. `network` 以外の情報 (`info`) を取り分ける (`const { network, ...info } = file;`)。
3. 各層の重みの要素数が、層の大きさから計算した数と一致するか確かめる (`expectLength`)。
4. 重みを `number[]` から `Float32Array` に変換する。

さらに App は、読み込んだモデルが **この Web 版の観測の作り方と合っているか** を確かめています ([src/App.tsx:50-53](src/App.tsx#L50-L53) の `loaded` → [src/ai/agent.ts:15-17](src/ai/agent.ts#L15-L17))。
モデルの JSON に入っているチャネル名の並びが、Web 版の `GRID_CHANNEL_NAMES` と一致しなければエラーにします。
学習側で観測の仕様を変えたのに Web 側を直し忘れると、AI はでたらめに動きます。黙っておかしく動くより、読み込み時にはっきり失敗するほうが原因を見つけやすくなります。

### 14.3 観測: 盤面を数値の配列にする

ニューラルネットは数値しか扱えないので、盤面を数値の配列に直します。これを **観測** (observation) と呼びます。

- **grid**: 9 枚の「盤面と同じ大きさの画像」。1 枚目は自分の頭があるマスだけ 1、2 枚目は胴体、3 枚目は普通のリンゴ…と、種類ごとに分けます。
- **vector**: 13 個の数値。今の向き・ブーストの残り・残り時間・相手とのスコア差など、盤面の絵では表しにくい情報です。

出典: [src/ai/observation.ts:61-86](src/ai/observation.ts#L61-L86) (抜粋)

```ts
export function encodeObservation(env, player, spec, grid, vector): void {
  const rules = env.rules;
  const w = rules.width;
  const plane = w * rules.height;
  const field = env.fields[player];
  const snake = field.snake;
  const cell = (channel: number, x: number, y: number) => channel * plane + y * w + x;

  grid.fill(0);
  grid.fill(1, 8 * plane, 9 * plane);  // 9 枚目 (in_bounds) は全部 1

  grid[cell(0, snake.head.x, snake.head.y)] = 1;
  // 胴体は首が 1、尻尾に向かって 1 / (長さ - 1) まで小さくなる
  const len = snake.length;
  for (let i = 1; i < len; i++) {
    const p = snake.body[i];
    grid[cell(1, p.x, p.y)] = (len - i) / (len - 1);
  }
  for (const item of field.items) {
    grid[cell(ITEM_CHANNEL[item.kind], item.pos.x, item.pos.y)] = 1;
  }
  // ...
}
```

- 3 次元の `[チャネル][y][x]` を 1 次元の `Float32Array` に並べています。位置の計算は `channel * plane + y * w + x` です。
- 配列は引数で受け取り、中身を書き換えるだけで、新しく作りません。`AiPlayer` がコンストラクタで 1 回だけ作ったものを使い回します ([src/ai/agent.ts:26-31](src/ai/agent.ts#L26-L31))。0.1 秒ごとに配列を作ると、ゴミ集め (GC) が増えてカクつくことがあるからです。
- **この関数は Rust 側の [../learn/src/env/observation.rs](../learn/src/env/observation.rs) と 1 つずつ同じ順番で同じ値を書く必要があります**。1 つでも違えば、AI は学習したときと違う世界を見ることになります。これが [../CLAUDE.md](../CLAUDE.md) に「観測の仕様を変えたら両方を揃える」と書いてある理由です。

### 14.4 順伝播: 畳み込みと全結合を手で書く

出典: [src/ai/model.ts:136-158](src/ai/model.ts#L136-L158)

```ts
predict(grid: Float32Array, vector: Float32Array): Float32Array {
  // (長さの確認は省略)
  let x = grid;
  let h = this.info.observation.grid_height;
  let w = this.info.observation.grid_width;
  for (const layer of this.conv) {
    [x, h, w] = conv2dRelu(layer, x, h, w);   // 畳み込み層 (3 層)
  }

  const flat = new Float32Array(x.length + vector.length);
  flat.set(x);                  // 畳み込みの結果を平らにして
  flat.set(vector, x.length);   // 後ろに vector をつなぐ
  x = flat;

  this.dense.forEach((layer, i) => {
    x = linear(layer, x, i < this.dense.length - 1);   // 全結合層 (最後の層だけ ReLU なし)
  });
  return x;   // 6 個の Q 値
}
```

16×16 のモデルでは、形は次のように変わります。

```
grid 9×16×16 ─conv(3×3, stride 1)─▶ 16×16×16 ─conv(3×3, stride 2)─▶ 32×8×8 ─conv(3×3, stride 2)─▶ 64×4×4
                                                                                                   │ 平らにする (1024)
                                                                                        + vector 13 = 1037
                                                                                                   │
                                                                         dense 1037→128 (ReLU) → dense 128→6
```

全結合層は「入力のそれぞれに重みを掛けて足し、バイアスを足す」だけです。

出典: [src/ai/model.ts:228-240](src/ai/model.ts#L228-L240)

```ts
function linear(layer: Dense, input: Float32Array, relu: boolean): Float32Array {
  const { inFeatures, outFeatures, weight, bias } = layer;
  const out = new Float32Array(outFeatures);
  for (let o = 0; o < outFeatures; o++) {
    let sum = bias[o];
    const row = o * inFeatures;
    for (let i = 0; i < inFeatures; i++) {
      sum += weight[row + i] * input[i];
    }
    out[o] = relu && sum < 0 ? 0 : sum;
  }
  return out;
}
```

畳み込み ([src/ai/model.ts:186-226](src/ai/model.ts#L186-L226)) も考え方は同じで、「3×3 の窓を盤面の上でずらしながら、窓の中の値に重みを掛けて足す」を、出力チャネルごとに繰り返します。
盤面の外 (パディング) は 0 とみなすので、盤面の外にはみ出す範囲を最初に計算して、ループの範囲から外しています (`oyStart`〜`oyEnd`)。こうすると、ループの中で「盤面の中か」を毎回確かめずに済み、速くなります。
計算式は [../learn/RULES.md の 11 章](../learn/RULES.md) に書いてあり、Rust 側の学習ライブラリの計算と同じ結果になります。

### 14.5 行動を選ぶ: Q 値と難易度

出典: [src/ai/agent.ts:34-47](src/ai/agent.ts#L34-L47)

```ts
decide(env: GameEnv, player: number): Decision {
  encodeObservation(env, player, this.model.info.observation, this.grid, this.vector);
  const qValues = this.model.predict(this.grid, this.vector);
  const actions = this.model.info.actions;

  if (Math.random() < this.randomActionRate) {
    return { action: actions[Math.floor(Math.random() * actions.length)], qValues, random: true };
  }
  let best = 0;
  for (let i = 1; i < qValues.length; i++) {
    if (qValues[i] > qValues[best]) best = i;
  }
  return { action: actions[best], qValues, random: false };
}
```

- Q 値は「その行動をとったら、この先どれくらい得をするか」の見積もりです。一番大きいものを選べば、AI にとって最善の手です。
- 難易度は、一定の確率でわざとランダムな行動をとらせることで作っています (かんたん 60%・ふつう 30%・むずかしい 0%)。確率はモデル JSON の `difficulty` に入っています。
- ランダムな行動のときも Q 値は計算しておき、画面の表に出します。「AI は本当はこうしたかった」が見えます。

### 14.6 推論はどこで動いているか

推論は、ゲームループの `tick` の中 ([11.4 節](#114-1-ティックの中身)) で、**画面を描くのと同じスレッド (メインスレッド)** で動いています。

- JavaScript は、基本的に 1 本のスレッドで、描画・入力・計算を順番にこなします。計算が長引くと、その間は画面が止まります。
- このモデルの推論は数ミリ秒で終わり、1 フレーム (16.7 ミリ秒) に収まるので、メインスレッドで問題ありません。
- もっと大きなモデルで 1 フレームに収まらなくなったら、**Web Worker** (別のスレッドで JavaScript を動かす仕組み) に推論を移します。Worker とは `postMessage` でデータを送り合います。
  そのときは「AI の判断が 1 ティック遅れて届く」ことを考えた設計が要ります。

Performance タブで記録すると、1 フレームの中で `predict` にどれだけ時間がかかっているか見られます。

---

## 15. ブラウザの機能: localStorage と History API

### 15.1 localStorage: ブラウザに少しだけ保存する

出典: [src/App.tsx:77-84](src/App.tsx#L77-L84)

```ts
const changeLayout = (next: Layout) => {
  setLayout(next);
  try {
    localStorage.setItem(LAYOUT_KEY, next);
  } catch {
    // 保存できなくても、この画面では切り替わる
  }
};
```

- `localStorage` は、ブラウザにキーと文字列の組を保存する仕組みです。ページを閉じても残ります。
- 保存先は **そのブラウザの、そのサイト (オリジン) だけ** です。別のブラウザや別の端末には届きません。サーバーにも送られません。
- 容量は 5MB 程度です。設定などの小さな値に向いています。
- プライベートブラウズや、サイトデータを禁止した設定では、読み書きで例外が出ることがあります。必ず `try/catch` で囲み、保存できなくても動くようにします。
- キーは `"duelsnake.layout"` のようにアプリ名を前に付けます ([src/App.tsx:23](src/App.tsx#L23))。同じドメイン (`24-fuji.github.io`) の他のアプリとぶつからないようにするためです。

### 15.2 History API: 戻る操作でルール画面を閉じる

ルール画面は、URL を変えずに画面を切り替えています。そのままだと、スマホで「戻る」操作 (iPhone の横スワイプなど) をすると、ルール画面ではなくアプリそのものから離れてしまいます。
そこで、History API を使って「戻る」でルール画面を閉じられるようにしています。

出典: [src/App.tsx:86-119](src/App.tsx#L86-L119) (抜粋)

```ts
useEffect(() => {
  // ルール画面のまま再読み込みしたときは、ゲーム画面で始める
  if (isRulesState(history.state)) history.replaceState(null, "");
  const onPopState = (e: PopStateEvent) => {
    if (isRulesState(e.state)) {
      history.back();     // 「進む」でルール画面の履歴に入ったら、開かずに戻す
      return;
    }
    setShowRules(false);  // 「戻る」でルール画面を閉じる
  };
  window.addEventListener("popstate", onPopState);
  return () => window.removeEventListener("popstate", onPopState);
}, []);

const openRules = () => {
  (document.activeElement as HTMLElement | null)?.blur?.();
  scrollRef.current = window.scrollY;
  history.pushState(RULES_STATE, "");   // 履歴を 1 つ積む
  setShowRules(true);
};

const closeRules = useCallback(() => {
  if (isRulesState(history.state)) history.back();   // 積んだ履歴を戻る。popstate で閉じる
  else setShowRules(false);
}, []);
```

| API | 何をするか |
|---|---|
| `history.pushState(state, "")` | ページを読み直さずに、履歴を 1 つ積む。`state` に好きな値を入れておける |
| `history.replaceState(state, "")` | 今の履歴の `state` を置き換える |
| `history.back()` | 1 つ戻る (ブラウザの戻るボタンと同じ) |
| `popstate` イベント | 戻る・進むで履歴を移ったときに来る。`e.state` は移った先の `state` |

流れは次のとおりです。

```
  [ゲーム画面] ── ボタン: pushState ──▶ [ゲーム画面][ルール画面]  (showRules = true)
                                                 │
       「← 戻る」ボタン: history.back()  ────────┤
       ブラウザの戻る / 横スワイプ ───────────────┤
                                                 ▼
  [ゲーム画面] ◀── popstate (state は null) ── setShowRules(false)
```

- 「← 戻る」ボタンも `history.back()` を呼ぶだけにして、閉じる処理は `popstate` の 1 か所にまとめています。こうすると、どの方法で戻っても履歴と画面が食い違いません。
- 「進む」でルール画面の履歴に入ったときは、すぐに戻しています。プレー中に右へスワイプしたときに、ルール画面が勝手に開かないようにするためです。
- 画面が増えて URL ごとに画面を分けたくなったら、React Router などのルーティングライブラリを使います。このアプリは画面が 2 つだけなので、History API を直接使いました。

---

## 16. スタイル: CSS とレイアウト

スタイルは [src/styles.css](src/styles.css) の 1 ファイルにまとめ、クラス名で当てています。
CSS Modules や Tailwind CSS などの仕組みは使っていません。この規模なら 1 ファイルで十分です。

### 16.1 よく使っているレイアウト

**Flexbox** (`display: flex`) は、要素を横か縦に並べるときに使います。

出典: [src/styles.css:23-28](src/styles.css#L23-L28)

```css
.settings {
  display: flex;
  flex-wrap: wrap;       /* 幅が足りなければ折り返す */
  gap: 12px 20px;        /* 縦と横の間隔 */
  align-items: center;   /* 縦方向の中央に揃える */
}
```

**Grid** (`display: grid`) は、表のように行と列で並べるときに使います。

出典: [src/styles.css:127-133](src/styles.css#L127-L133)

```css
.panel dl {
  display: grid;
  grid-template-columns: 7em 1fr;   /* 見出しの列は 7 文字分、値の列は残り全部 */
  gap: 2px 8px;
}
```

**重ねる** (`position: absolute`) は、盤面の上に「Enter キーで開始」を重ねるのに使います。

出典: [src/styles.css:86-98](src/styles.css#L86-L98)

```css
.overlay {
  position: absolute;
  inset: 0;                  /* 親 (position: relative の .boards や .stage) いっぱいに広げる */
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 13, 17, 0.6);
  pointer-events: none;      /* タップやクリックは下の盤面に通す */
}
```

`pointer-events: none` が無いと、携帯版で重ねた表示がタップを受け取ってしまい、「タップで開始」が盤面に届きません。

### 16.2 パソコン版と携帯版の切り替え

レイアウトの切り替えは、CSS のメディアクエリ (`@media (max-width: ...)`) ではなく、**一番外側の要素のクラス** で行っています。

```tsx
<div className={`app ${layout}`}>   {/* src/App.tsx:145。"app pc" か "app mobile" */}
```

```css
.app.mobile { padding: 12px; }            /* src/styles.css:264 */
.game.mobile { max-width: 75vh; }         /* src/styles.css:278-282。横向きでも盤面が画面の高さに収まる */
```

利用者が「表示」で選べるようにしたので、画面の幅ではなく state で決めています。
また、携帯版とパソコン版では並べ方 (ワイプに相手を小さく映すかどうか) も違うので、CSS だけでなく JSX も分けています ([src/components/GameScreen.tsx:215-273](src/components/GameScreen.tsx#L215-L273))。

### 16.3 細かい工夫

| 書き方 | 場所 | 効果 |
|---|---|---|
| `color-scheme: dark` | [src/styles.css:2](src/styles.css#L2) | セレクトやスクロールバーなど、ブラウザが描く部品も暗い色にする |
| `font-variant-numeric: tabular-nums` | [src/styles.css:60](src/styles.css#L60) | 数字の幅を揃え、残り時間が変わっても文字がガタガタ動かない |
| `min-width: 5.5em` | [src/styles.css:371](src/styles.css#L371) | 制限時間の表示の長さが変わっても、隣の設定欄がずれない |
| `aspect-ratio` | [src/components/Board.tsx:65](src/components/Board.tsx#L65) | 幅に合わせて、盤面の縦横比のまま高さを決める |
| `vh` | [src/styles.css:280](src/styles.css#L280) | 画面の高さに対する割合 (75vh = 高さの 75%) |

---

## 17. テスト: Vitest

### 17.1 何をテストしているか

| ファイル | 確かめていること |
|---|---|
| [src/game/game.test.ts](src/game/game.test.ts) | 対戦のルール (移動・衝突・アイテム・ブースト・お邪魔・勝敗)。Rust の [../learn/src/env/tests.rs](../learn/src/env/tests.rs) と同じ内容 |
| [src/game/solo.test.ts](src/game/solo.test.ts) | 一人モードのルール |
| [src/ai/agent.test.ts](src/ai/agent.test.ts) | 同梱モデルの一覧、各モデルが Web 版と合うか、AI どうしで最後まで対戦できるか |
| [src/components/gestures.test.ts](src/components/gestures.test.ts) | スワイプ・長押し・タップ・ダブルタップの見分け |

テストしているのは、[5.2 節](#52-依存の向きを一方向にする) で分けた「計算するコード」だけです。
React の画面そのもの (ボタンを押したら何が出るか) はテストしていません。画面のテストは手間の割に壊れやすいので、**ロジックを画面の外に出して、そちらを厚くテストする** のが費用対効果のよいやり方です。

### 17.2 テストの書き方

出典: [src/game/game.test.ts:15-36](src/game/game.test.ts#L15-L36) と [src/game/game.test.ts:65-74](src/game/game.test.ts#L65-L74) (抜粋)

```ts
import { describe, expect, it } from "vitest";
import modelFile from "../../../model/recent-model/snake-model-16x16.json";

const baseRules = (): Rules => rulesFromConfig(modelFile.game as GameRules);

/** アイテムの無い盤面で始める */
function emptyEnv(rules: Rules = baseRules()): GameEnv {
  const env = new GameEnv(rules);
  for (const field of env.fields) field.items = [];
  return env;
}

describe("ルール", () => {
  it("通常は2ティックに1回動き、その直前に行動を決める", () => {
    const env = emptyEnv();
    expect(env.needsDecision(0)).toBe(false);
    env.step(none);
    expect(env.fields[0].snake.head).toEqual(pos(8, 8));
    expect(env.needsDecision(0)).toBe(true);
    env.step(none);
    expect(env.fields[0].snake.head).toEqual(pos(8, 7));
  });
});
```

- `describe` でまとめ、`it` で 1 つのテストを書き、`expect(実際の値).toBe(期待する値)` で確かめます。
- `toBe` は `===` で比べ、`toEqual` はオブジェクトや配列の中身を比べます。座標 `{x, y}` は `toEqual` で比べます。
- ルールの数値は、実際のモデル JSON から読んでいます。学習の設定を変えると、テストも同じ数値で動きます。
- `emptyEnv` のような「テスト用の状態を作る関数」を用意すると、各テストが短くなります。ランダムに出るアイテムを消して、結果を決まったものにしています。

### 17.3 時間を偽物にする

ジェスチャーのテストでは、長押しの 350 ミリ秒を本当に待たずに済むよう、タイマーを偽物にしています。

出典: [src/components/gestures.test.ts:7-15](src/components/gestures.test.ts#L7-L15) と [src/components/gestures.test.ts:39-46](src/components/gestures.test.ts#L39-L46)

```ts
beforeEach(() => {
  vi.useFakeTimers();
  gestures = [];
  recognizer = new GestureRecognizer((g) => gestures.push(g));
});

afterEach(() => {
  vi.useRealTimers();
});

it("動かさずに押し続けると長押しになり、離してもタップにはならない", () => {
  recognizer.down(1, 0, 0, 0);
  vi.advanceTimersByTime(349);
  expect(gestures).toEqual([]);
  vi.advanceTimersByTime(1);        // ちょうど 350ms で長押しになる
  recognizer.up(1, 500);
  expect(gestures).toEqual([{ kind: "long_press" }]);
});
```

`vi.advanceTimersByTime(ms)` で、`setTimeout` の時計だけを進められます。
`GestureRecognizer` は、時刻を `e.timeStamp` から引数で受け取るように作っているので ([src/components/gestures.ts:50](src/components/gestures.ts#L50))、テストでは好きな時刻を渡せます。[10.5 節](#105-乱数を差し替えられるようにする) の Picker と同じ考え方です。

### 17.4 同梱のモデルをすべて試す

出典: [src/ai/agent.test.ts:8-11](src/ai/agent.test.ts#L8-L11) と [src/ai/agent.test.ts:42-51](src/ai/agent.test.ts#L42-L51)

```ts
const files = import.meta.glob<ModelFile>("../../../model/recent-model/snake-model-*.json", {
  eager: true,
  import: "default",
});

describe.each(Object.entries(files))("学習済みモデル %s", (path, file) => {
  const model = new SnakeModel(file);

  it("この Web 版で使え、ファイル名と盤面サイズが一致する", () => {
    expect(() => assertModelCompatible(model)).not.toThrow();
    // ...
  });
```

- アプリでは `query: "?url"` で URL を受け取りましたが ([2.3 節](#23-url-はどこから来るのか))、テストでは付けずに **中身そのもの** を受け取っています。テストでは fetch を使わずに済みます。
- `describe.each` で、見つかったモデルの数だけ同じテストを繰り返します。新しい盤面のモデルを置くと、自動でテストが増えます。
- 学習側が新しいモデルを書き出したときに、Web 版で使えないモデルを公開してしまうのを防げます。

### 17.5 Rust と同じテストを持つ

[src/game/game.test.ts](src/game/game.test.ts) の 1 行目には、次のコメントがあります。

```ts
// learn/src/env/tests.rs と同じ内容のテスト。両方の実装が同じルールで動くことを確かめる
```

2 つの言語で同じルールを書くときは、同じテストを両方に書いておくと、片方だけ直したときにもう片方のテストで気づけます。
ルールを変えるときは、`just test` (Rust) と `just web-test` (Web) の両方を通します ([../CLAUDE.md](../CLAUDE.md))。

---

## 18. ビルドと公開: GitHub Actions と GitHub Pages

### 18.1 全体の流れ

```
 git push (main)
    │  web/ か model/recent-model/ かワークフローが変わっていれば
    ▼
 GitHub Actions: build ジョブ (Ubuntu の仮想マシン)
    1. リポジトリを取ってくる (checkout)
    2. Node.js 22 を入れる (setup-node)
    3. npm ci        … package-lock.json どおりに依存を入れる
    4. npm test      … テストが落ちたらここで止まり、公開しない
    5. npm run build … tsc で型を確かめ、vite build で web/dist/ を作る
    6. web/dist/ を成果物として上げる (upload-pages-artifact)
    │
    ▼
 GitHub Actions: deploy ジョブ
    1. Pages の設定が「GitHub Actions から公開」になっているか確かめる
    2. 成果物を GitHub Pages に公開する (deploy-pages)
    │
    ▼
 https://24-fuji.github.io/duelsnake-ai/
```

### 18.2 ワークフローを読む

出典: [../.github/workflows/deploy-web.yml:4-20](../.github/workflows/deploy-web.yml#L4-L20)

```yaml
on:
  push:
    branches: [main]
    paths:
      - "web/**"
      - "model/recent-model/**"
      - ".github/workflows/deploy-web.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true
```

| 設定 | 意味 |
|---|---|
| `on.push.branches` / `paths` | main への push で、関係するファイルが変わったときだけ動く。`learn/` だけを直したときは動かない |
| `workflow_dispatch` | GitHub の画面から手で動かせるようにする |
| `permissions` | このワークフローに与える権限。Pages に書き込む権限と、公開のための認証トークンを作る権限だけを与える |
| `concurrency` | 続けて push したら、前の実行を取り消して最新だけを公開する |

出典: [../.github/workflows/deploy-web.yml:22-40](../.github/workflows/deploy-web.yml#L22-L40)

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: web
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 22
          cache: npm
          cache-dependency-path: web/package-lock.json
      - run: npm ci
      - run: npm test
      - run: npm run build
      - uses: actions/upload-pages-artifact@v5
        with:
          path: web/dist
```

- `uses:` は、他の人が作った部品 (アクション) を使う書き方、`run:` はシェルのコマンドを動かす書き方です。
- `cache: npm` は、ダウンロードしたパッケージを次回に使い回して速くします。
- テストがビルドより前にあるので、**テストが落ちたら公開されません**。壊れたものを公開しないための安全装置です。
- ビルドには `model/recent-model/` のモデルも入ります ([2.3 節](#23-url-はどこから来るのか))。学習側がモデルをコミットして push すると ([../README.md](../README.md) の「学習」)、このワークフローが動いて、公開版の AI も新しくなります。

deploy ジョブの最初のステップ ([../.github/workflows/deploy-web.yml:50-58](../.github/workflows/deploy-web.yml#L50-L58)) は、リポジトリの Settings → Pages の配信元が「GitHub Actions」になっているかを確かめています。
「Deploy from a branch」のままだと、せっかく公開しても README のページで上書きされてしまうので、原因の分かるエラーで止めています。

### 18.3 GitHub Pages の性質

- **ファイルを置くだけ** のホスティングです。サーバーでプログラムは動かせません (PHP や Node.js のサーバーは置けない)。
- `https://<ユーザー名>.github.io/<リポジトリ名>/` の **サブパス** で公開されます。`base: './'` が要るのはこのためです ([4.3 節](#43-viteconfigjs))。
- 公開リポジトリなら無料です。公開したファイルは誰でもダウンロードできます (モデルも含めて)。
- 同じ種類のサービスに Cloudflare Pages・Netlify・Vercel などがあります。どれも「ビルドした `dist/` を置く」という点は同じです。

---

## 19. 変更するときのレシピ

実際に手を入れるときの手順を、よくある変更ごとにまとめます。

### 19.1 設定欄を 1 つ増やす

例: 「盤面の線を消す」チェックボックスを足す。

1. `App` に state を足す: `const [showGrid, setShowGrid] = useState(true);` ([src/App.tsx:64-73](src/App.tsx#L64-L73) の並び)
2. 設定欄に入力を足す ([src/App.tsx:216-224](src/App.tsx#L216-L224) のチェックボックスをまねる)
3. `GameScreen` と `SoloScreen` の `Props` に `showGrid: boolean` を足し、`Board` に渡す
4. `Board` から `drawField` に渡し、`drawField` の線を引く部分を `if (showGrid)` で囲む
5. `Board` の描画の `useEffect` の依存配列に `showGrid` を足す (忘れると、切り替えても描き直されない)
6. 保存したいなら、`changeLayout` と同じように localStorage に書く ([15.1 節](#151-localstorage-ブラウザに少しだけ保存する))

「試合をやり直すべき設定」なら、`GameScreen` の `key` に入れます ([8.6 節](#86-key-で部品を作り直す))。

### 19.2 新しい盤面サイズを遊べるようにする

Web 側のコードは変えません。

1. `learn/config.yaml` の `game.grid` を変えて学習する (`just train`)
2. `model/recent-model/snake-model-<幅>x<高さ>.json` ができる
3. `just web-dev` を動かし直すと、盤面の選択肢に出る ([2.3 節](#23-url-はどこから来るのか))
4. `just web-test` で、新しいモデルでもテストが通ることを確かめる ([17.4 節](#174-同梱のモデルをすべて試す))

盤面が大きいと 1 マスが小さくなります。大きさは [src/components/Board.tsx:7-11](src/components/Board.tsx#L7-L11) の `BOARD_PIXELS` と `defaultCellSize` で決まります。

### 19.3 ゲームのルールを変える

例: リンゴのスコアを変える、新しいアイテムを足す。

1. [../learn/RULES.md](../learn/RULES.md) を直す (ルールの正はここ)
2. 学習側 `learn/src/env/` と Web 側 `web/src/game/` を同じように直す
3. 両方のテストに同じテストを足す (`learn/src/env/tests.rs` と `src/game/game.test.ts`)
4. `just test` と `just web-test` を両方通す
5. ルール画面 [src/components/RulesScreen.tsx](src/components/RulesScreen.tsx) の説明を直す
6. 観測 (AI の入力) が変わるなら、[src/ai/observation.ts](src/ai/observation.ts) と `learn/src/env/observation.rs` を揃えて、モデルを学習し直す

新しいアイテムを足すときは、`ItemType` ([src/game/field.ts:4](src/game/field.ts#L4)) に足すと、`Record<ItemType, ...>` を使っている場所 (画像の対応・観測のチャネル・取った数) がエラーになって、直すべき場所を TypeScript が教えてくれます ([7.6 節](#76-record-と-readonly-と-as-const))。

### 19.4 画像を差し替える

1. [src/assets/items/](src/assets/items/) の同じ名前の PNG を置き換える。背景を透過し、余白を削った正方形にする ([src/assets/README.md](src/assets/README.md))
2. 新しい画像を足すなら、[src/components/draw.ts](src/components/draw.ts) で `import` し、`SPRITE_URLS` に足す

### 19.5 変更したら確かめること

```sh
just web-test               # テスト
cd web && npm run build     # 型の検査 (tsc) とビルド。公開前に必ず通す
cd web && npm run preview   # ビルドしたものを手元で動かす
```

- パソコン版と携帯版の両方で見る。パソコンで携帯版を試すには、画面の「表示」を切り替えるか、開発者ツールのデバイスモード (Ctrl+Shift+M) を使う
- 本物のスマホで試すには、`npx vite --host` で起動し、同じ Wi-Fi のスマホから `http://<PC の IP アドレス>:3000` を開く

---

## 20. ゼロから同じ構成のアプリを作る

最後に、同じようなアプリ (ブラウザで動くゲームや、学習済みモデルを使う道具) を作るときの手順をまとめます。

### 20.1 プロジェクトを作る

```sh
npm create vite@latest my-app -- --template react-ts
cd my-app
npm install
npm install -D vitest
npm run dev
```

これで、このリポジトリの `web/` とほぼ同じ骨組み (`index.html`・`src/main.tsx`・`vite.config.ts`・`tsconfig.json`) ができます。
`package.json` の `scripts` に `"test": "vitest run"` を足しておきます。
GitHub Pages に置くなら、`vite.config` に `base: './'` を足します。

### 20.2 作る順番

1. **ルールを決めて文章にする**
   [../learn/RULES.md](../learn/RULES.md) のように、画面を作る前に書きます。処理の順番 (1 ティックで何をどの順にするか) まで決めておくと、実装で迷いません。
2. **ロジックを React なしで書き、テストする** (`src/game/` にあたる)
   - 状態を持つクラスと、`step(inputs)` のような「1 回進める」関数を作る
   - 時間はティックで数える ([10.1 節](#101-秒ではなくティックで数える))
   - 乱数や時刻は外から渡せるようにする ([10.5 節](#105-乱数を差し替えられるようにする))
   - この段階でテストを書く。画面が無くてもゲームが正しく動くことを確かめられる
3. **画面の骨組みを作る**
   - `App` に設定の state を置き、ゲーム画面の部品に props で渡す
   - ゲームの状態は `useRef` に置き、`version` で描き直す ([9.2 節](#92-useref-描き直しを起こさない入れ物)、[11.3 節](#113-描き直しのきっかけ-version))
   - rAF の固定タイムステップのループを書く ([11.2 節](#112-経過時間をためて決まった間隔で進める))
4. **描画を作る**
   - 大量に動くものは Canvas、文字や表は DOM ([12.1 節](#121-2-種類の描き方))
   - `devicePixelRatio` に対応する ([12.2 節](#122-board-react-と-canvas-をつなぐ))
5. **入力を作る**
   - キーボードは `window` の `keydown`、`e.code`、`preventDefault`、入力はためておいて次のティックで反映 ([13.1 節](#131-キーボード))
   - タッチは Pointer Events と `touch-action: none` ([13.2 節](#132-タッチ-pointer-events))
6. **AI を足す** (必要なら)
   - 学習側でモデルを JSON などに書き出す。重みだけでなく、ルールと入力の仕様も一緒に入れておく
   - 読み込むときに形式と仕様を確かめる ([14.2 節](#142-読み込みと確認))
   - 小さいモデルなら自分で順伝播を書き、大きいなら ONNX Runtime Web などを使う ([3.4 節](#34-推論をどう実装するかの選択肢))
7. **公開する**
   - GitHub Actions でテスト → ビルド → Pages に公開 ([18 章](#18-ビルドと公開-github-actions-と-github-pages))

### 20.3 最小のゲームループの雛形

この文書の内容を 1 つにまとめた、最小の雛形です。ここから育てていくとよいでしょう。

```tsx
import { useEffect, useRef, useState } from "react";

// --- ロジック (本当は src/game/ に分ける) ---
type Dir = "up" | "down" | "left" | "right";
class Env {
  x = 5;
  y = 5;
  dir: Dir = "right";
  step(inputs: Dir[]) {
    for (const d of inputs) this.dir = d;
    if (this.dir === "up") this.y -= 1;
    if (this.dir === "down") this.y += 1;
    if (this.dir === "left") this.x -= 1;
    if (this.dir === "right") this.x += 1;
  }
}

const KEYS: Record<string, Dir> = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right" };
const TICK_MS = 100;
const CELL = 20;

// --- 画面 ---
export function Game() {
  const envRef = useRef<Env | null>(null);
  if (envRef.current === null) envRef.current = new Env();
  const inputsRef = useRef<Dir[]>([]);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [version, setVersion] = useState(0);

  // 入力: ためておく
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const dir = KEYS[e.code];
      if (!dir) return;
      e.preventDefault();
      if (!e.repeat) inputsRef.current.push(dir);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  // ループ: 固定タイムステップ
  useEffect(() => {
    let last = performance.now();
    let pending = 0;
    let id = 0;
    const frame = (now: number) => {
      pending += Math.min(now - last, 250);
      last = now;
      let ticked = false;
      while (pending >= TICK_MS) {
        pending -= TICK_MS;
        envRef.current!.step(inputsRef.current.splice(0));
        ticked = true;
      }
      if (ticked) setVersion((v) => v + 1);
      id = requestAnimationFrame(frame);
    };
    id = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(id);
  }, []);

  // 描画: version が変わるたびに描き直す
  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx) return;
    const env = envRef.current!;
    ctx.fillStyle = "#111";
    ctx.fillRect(0, 0, 400, 400);
    ctx.fillStyle = "#4fc3f7";
    ctx.fillRect(env.x * CELL, env.y * CELL, CELL, CELL);
  }, [version]);

  return <canvas ref={canvasRef} width={400} height={400} />;
}
```

この雛形に、このリポジトリで行っている次の工夫を 1 つずつ足していくと、同じ水準になります。

- 状態 (`ready`/`running`/`paused`/`over`) と、開始・一時停止 ([11.5 節](#115-状態の移り変わり))
- `devicePixelRatio` への対応 ([12.2 節](#122-board-react-と-canvas-をつなぐ))
- 設定欄と、`key` による作り直し ([8.6 節](#86-key-で部品を作り直す))
- 設定欄のフォーカスを外す ([13.1 節](#131-キーボード))
- タッチ操作 ([13.2 節](#132-タッチ-pointer-events))
- ロジックのテスト ([17 章](#17-テスト-vitest))

### 20.4 作る前のチェックリスト

- [ ] サーバーは要るか ([3.5 節](#35-判断のチェックリスト))
- [ ] ルールと処理の順番を文章にしたか
- [ ] ロジックは React・DOM に依存せずに書けているか
- [ ] 時間はティックで数えているか
- [ ] 乱数・時刻を外から差し替えられるか
- [ ] 外から読むデータ (JSON など) の中身を確かめているか
- [ ] `useEffect` にはすべてクリーンアップがあるか
- [ ] `strict` を有効にしているか
- [ ] テストが落ちたら公開されない仕組みになっているか

---

## 21. よくあるハマりどころ

このリポジトリを作る中で実際に気をつけた点を中心に、よくある問題と対策をまとめます。

| 症状 | 原因 | 対策 (このリポジトリの例) |
|---|---|---|
| キーを押しても前の状態のように動く | 古いクロージャ。ハンドラーが作られたときの state を見ている | ref に最新の値を入れて読む ([9.2 節](#92-useref-描き直しを起こさない入れ物) の (3)) |
| 1 回押しただけなのに 2 回動く | `addEventListener` の解除忘れで、ハンドラーが 2 つ登録されている。開発中は StrictMode が effect を 2 回動かすので見つかりやすい | `useEffect` のクリーンアップで `removeEventListener` する |
| state を変えたのに画面が変わらない | オブジェクトの中身を直接書き換えた | `{ ...t, draws: t.draws + 1 }` のように新しいオブジェクトを作る ([8.4 節](#84-state-変わる値を持つ)) |
| ref の中身を変えたのに Canvas が描き直されない | ref の変化は描き直しを起こさない。同じオブジェクトを依存にしても変化に気づかない | `version` を増やし、描画の effect の依存に入れる ([11.3 節](#113-描き直しのきっかけ-version)) |
| 矢印キーでセレクトの値が変わる、スペースでボタンが押される | 設定欄にフォーカスが残っている | 開始時に `blur()` する、入力欄からのキーは無視する ([13.1 節](#131-キーボード)) |
| 矢印キーでページがスクロールする | ブラウザの既定の動き | `e.preventDefault()` |
| スマホでスワイプするとページがスクロール・拡大する | ブラウザがタッチを横取りしている | `touch-action: none` ([13.2 節](#132-タッチ-pointer-events)) |
| 盤面がぼやける | Canvas の画素数が画面の画素数より少ない | `devicePixelRatio` 倍の大きさで描く ([12.2 節](#122-board-react-と-canvas-をつなぐ)) |
| 時間の計算が 1 ティックずれる | `0.6 / 0.1 = 5.999...` のような小数の誤差 | `Math.round` で四捨五入する ([10.1 節](#101-秒ではなくティックで数える)) |
| タブを戻したらゲームが一気に進んだ | 裏にいた間の経過時間をまとめて処理した | 1 フレームの経過時間に上限を付ける (`Math.min(now - last, 250)`) |
| 盤面を切り替えたのに古い盤面に戻る | 読み込みの競合 | effect のクリーンアップで古い結果を捨てる ([2.4 節](#24-読み込みの競合を防ぐ)) |
| 手元では動くのに、公開したら真っ白 | サブパスで公開され、`/assets/...` が見つからない | `base: './'` ([4.3 節](#43-viteconfigjs))。開発者ツールの Console と Network で 404 を探す |
| 開発では動くのに `npm run build` が失敗する | Vite の開発サーバーは型を検査しない | こまめに `npm run build` か `npm run typecheck` を動かす |
| プライベートブラウズでだけ落ちる | localStorage が例外を投げる | `try/catch` で囲む ([15.1 節](#151-localstorage-ブラウザに少しだけ保存する)) |
| AI がでたらめに動く | 学習時と観測の作り方が違う | 読み込み時に観測仕様を確かめる ([14.2 節](#142-読み込みと確認))。観測を変えたら両方を揃えて学習し直す |

---

## 22. 練習問題

手を動かして理解を確かめるための問題です。易しい順に並べています。

1. **色を変える**: 相手 (AI) のヘビの色を緑にしてください。
   ヒント: [src/components/draw.ts:20-23](src/components/draw.ts#L20-L23)
2. **キーを足す**: テンキーの 8・2・4・6 でも上下左右に動けるようにしてください。
   ヒント: `e.code` は `"Numpad8"` など。[src/components/GameScreen.tsx:16-29](src/components/GameScreen.tsx#L16-L29)
3. **表示を足す**: 対戦画面の上部に、経過ティック数 (`env.tick`) を出してください。なぜ毎ティック表示が更新されるのか説明してください。
   ヒント: [11.3 節](#113-描き直しのきっかけ-version)
4. **設定を保存する**: 難易度を localStorage に保存し、次に開いたときも同じ難易度で始まるようにしてください。
   ヒント: [src/App.tsx:32-40](src/App.tsx#L32-L40) と [src/App.tsx:77-84](src/App.tsx#L77-L84)
5. **最高スコアを残す**: 一人モードの最高スコア ([src/components/SoloScreen.tsx:68](src/components/SoloScreen.tsx#L68)) は、画面を作り直すと消えます。盤面サイズごとに localStorage に残してください。
6. **テストを読んで足す**: [src/game/game.test.ts:265](src/game/game.test.ts#L265) のブーストのテストを読み、何を確かめているか説明してください。そのうえで「クールダウン中にブーストを押しても発動しない」ことを確かめるテストを足してください。
7. **ジェスチャーを足す**: 2 本指でタップしたら一時停止するようにしてください。`GestureRecognizer` に何を足し、テストをどう書くか考えてから実装してください。
8. **推論の時間を測る**: `performance.now()` を使って、`AiPlayer.decide` 1 回にかかる時間をコンソールに出し、パソコンとスマホで比べてください。
9. **Web Worker に移す** (発展): AI の推論を Web Worker に移してください。判断が届くまでの間、ゲームをどう進めるか (待つか、前の判断を使うか) を決める必要があります。
10. **オンライン対戦を設計する** (発展): 人どうしがオンラインで対戦できるようにするには、何をどこで動かし、どう通信すればよいか、[3 章](#3-設計の選び方-どこで動かしどう通信するか) の観点で設計を書いてください。
    考えること: ゲームの進行はどちらの端末 (またはサーバー) で行うか、入力の遅れをどう扱うか、不正をどう防ぐか。

---

## 23. 次に読むもの

- [MDN Web Docs](https://developer.mozilla.org/ja/): HTML・CSS・JavaScript・ブラウザの API (Canvas・Pointer Events・History API・localStorage・fetch) の公式に近い解説。困ったらまずここ
- [React 公式ドキュメント](https://ja.react.dev/): 特に「Learn」の「State の管理」と「避難ハッチ (Escape Hatches)」の章が、この文書の 8〜9 章に対応します
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html): 型の書き方の全体
- [Vite ガイド](https://ja.vite.dev/guide/): `import.meta.glob` や静的アセットの扱い
- [Vitest](https://vitest.dev/): テストの書き方とモック
- 「Fix Your Timestep!」(Glenn Fiedler): ゲームループの固定タイムステップの古典的な解説 ([11 章](#11-ゲームループ-requestanimationframe-と固定ティック))
- [../learn/RUST_TUTORIAL.md](../learn/RUST_TUTORIAL.md): 同じゲームを Rust で書いた学習側の解説。0.3 節に TypeScript との比較があります

---

## 付録: 用語集

| 用語 | 意味 | この文書での主な節 |
|---|---|---|
| SPA | 1 枚の HTML の中で、JavaScript が画面を切り替えるアプリ | 1.1 |
| 静的サイト | サーバーがファイルを返すだけのサイト | 1.1, 18.3 |
| HTTP / GET | ブラウザとサーバーのやりとりの決まり / ファイルを取りに行くリクエスト | 2.1 |
| fetch | ブラウザに組み込みの、HTTP でリクエストを送る関数 | 2.2 |
| Promise / async / await | あとで届く値と、それを待つ書き方 | 7.8 |
| WebSocket | ブラウザとサーバーをつなぎっぱなしにして、双方向に送る仕組み | 3.3 |
| DOM | ブラウザの中の、画面の要素の木 | 8.1 |
| Canvas | JavaScript でピクセルの絵を描く要素 | 12 |
| devicePixelRatio | CSS の 1 ピクセルが、実際の画素いくつにあたるか | 12.2 |
| Node.js | ブラウザの外で JavaScript を動かす実行環境 | 4 |
| npm / package.json | パッケージの管理 / その設定 | 4.1 |
| バンドル | たくさんのファイルを少数の JS にまとめること | 4.4 |
| Vite | 開発サーバーとバンドルの道具 | 4.3, 4.4 |
| HMR | 保存したファイルだけを、ページを読み直さずに差し替える機能 | 4.4 |
| ハッシュ付きファイル名 | 中身から作った文字列をファイル名に入れること。キャッシュのため | 2.5 |
| コンポーネント | 画面の部品を返す関数 | 8.2 |
| props | 親から子の部品に渡す値 | 8.2 |
| state | 部品が覚えておく値。変わると描き直す | 8.4 |
| JSX | JavaScript の中に HTML のように書く書き方 | 8.3 |
| フック | `use` で始まる、React の機能を使う関数 | 9 |
| クリーンアップ | `useEffect` から返す、後片付けの関数 | 9.1 |
| クロージャ | 作られたときの変数を覚えている関数 | 9.2 |
| ティック | ゲームを 1 回進める単位 (このアプリでは 0.1 秒) | 10.1 |
| ゲームループ | 「進める → 描く」の繰り返し | 11 |
| requestAnimationFrame | 次に画面を描き直す直前に関数を呼んでもらう予約 | 11.1 |
| 固定タイムステップ | 画面の速さと関係なく、決まった間隔でゲームを進める方法 | 11.2 |
| 依存性の注入 | 部品 (乱数・時刻など) を外から渡せるようにすること | 10.5 |
| Pointer Events | マウス・指・ペンを同じ形で扱う入力イベント | 13.2 |
| 観測 | AI に渡すために、盤面を数値の配列にしたもの | 14.3 |
| 順伝播 | ニューラルネットに入力を通して出力を計算すること | 14.4 |
| Q 値 | 「その行動をとったらこの先どれだけ得か」の見積もり | 14.5 |
| メインスレッド / Web Worker | 描画と同じスレッド / 別のスレッドで JS を動かす仕組み | 14.6 |
| localStorage | ブラウザにキーと文字列を保存する仕組み | 15.1 |
| History API | ページを読み直さずに履歴を操作する仕組み | 15.2 |
| Flexbox / Grid | CSS で要素を並べる 2 つの方法 | 16.1 |
| CI | push のたびに自動でテストやビルドを動かすこと (ここでは GitHub Actions) | 18 |
