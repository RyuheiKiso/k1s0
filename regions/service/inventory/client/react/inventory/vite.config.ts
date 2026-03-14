import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Viteビルド設定: React プラグインとBFFプロキシ
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/bff': 'http://localhost:8080',
    },
  },
});
