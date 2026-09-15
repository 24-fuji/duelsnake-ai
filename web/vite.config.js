import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
  },
  // ONNX Runtime Web用のWASMファイル読み込み設定
  optimizeDeps: {
    exclude: ['onnxruntime-web'],
  },
});