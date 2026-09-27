# assets

UI で使う画像 (PNG など)・音声・フォントを置くフォルダです。

コードから import すると、ビルド時にハッシュ付きのファイル名で `dist/` に出力されます。

```ts
import appleUrl from "../assets/apple.png";

const apple = new Image();
apple.src = appleUrl;
```

`items/` には盤面のアイテムとお邪魔ブロックの画像を置いています ([components/draw.ts](../components/draw.ts) が 1 マスに収めて描きます)。
画像はどれも背景を透過し、余白を削った正方形の PNG (128px) です。差し替えるときも同じ形にしてください。余白が大きいとマスの中で小さく見えます。
