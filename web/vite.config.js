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
