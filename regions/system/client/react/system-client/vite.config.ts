import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import dts from 'vite-plugin-dts';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    react(),
    dts({ include: ['src'] }),
  ],
  // 開発サーバーの設定（FE-002 監査対応: セキュリティヘッダーを追加する）
  server: {
    // 開発環境でもセキュリティヘッダーを設定してクリックジャッキング・MIME スニッフィング等を防止する
    headers: {
      // フレーム埋め込みを完全禁止してクリックジャッキング攻撃を防ぐ
      'X-Frame-Options': 'DENY',
      // ブラウザによる MIME タイプ推測を禁止してコンテンツスニッフィング攻撃を防ぐ
      'X-Content-Type-Options': 'nosniff',
      // リファラー情報をオリジンのみに制限してプライバシーリークを防ぐ
      'Referrer-Policy': 'strict-origin-when-cross-origin',
      // 不要なブラウザ機能へのアクセスを制限してフィンガープリント・盗聴リスクを低減する
      'Permissions-Policy': 'camera=(), microphone=(), geolocation=()',
      // 注意: HSTS（Strict-Transport-Security）は localhost では無効なため開発環境では設定しない
    },
  },
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'SystemClient',
      fileName: 'system-client',
    },
    rollupOptions: {
      external: ['react', 'react-dom', 'react/jsx-runtime'],
      output: {
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM',
        },
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './tests/setup.ts',
    css: true,
  },
});
