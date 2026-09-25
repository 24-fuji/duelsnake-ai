# assets

UI で使う画像 (PNG など)・音声・フォントを置くフォルダです。

コードから import すると、ビルド時にハッシュ付きのファイル名で `dist/` に出力されます。

```ts
import appleUrl from "../assets/apple.png";

const apple = new Image();
apple.src = appleUrl;
```

盤面の描画は今は図形で行っています ([components/draw.ts](../components/draw.ts))。画像に差し替えるときは、ここに PNG を置き、`draw.ts` の各関数を `ctx.drawImage` に置き換えてください。
