// k1s0 tier3 SPA Vite 設定
// React 18 + TypeScript 向けのビルド設定を定義する
// CSP unsafe-eval / unsafe-inline を使用しない設計（tier3/CLAUDE.md セキュリティ要件）

// Vite の defineConfig ヘルパーをインポートする
import { defineConfig } from "vite";
// React JSX 変換プラグインをインポートする（@vitejs/plugin-react）
import react from "@vitejs/plugin-react";

// Vite 設定オブジェクトをエクスポートする
export default defineConfig({
  // プラグイン: React JSX 変換（babel transform を使用する）
  plugins: [react()],
  // ビルド設定
  build: {
    // ES2022 をターゲットにする（tier3 TypeScript target に合わせる）
    target: "es2022",
    // source map を生成する（デバッグ用）
    sourcemap: true,
    // rollup オプション
    rollupOptions: {
      // vendor chunk を分割してキャッシュ効率を上げる
      output: {
        manualChunks: {
          // React 関連を vendor chunk にまとめる
          vendor: ["react", "react-dom"],
          // react-router-dom を別 chunk にする
          router: ["react-router-dom"],
        },
      },
    },
  },
  // 開発サーバー設定
  server: {
    // ポートを 5173 に固定する
    port: 5173,
    // BFF へのプロキシ設定（開発時に CORS を回避する）
    proxy: {
      // /auth/ 配下のリクエストを BFF（:8080）にプロキシする
      "/auth": {
        target: "http://localhost:8080",
        // リバースプロキシ時に Host ヘッダーを書き換える
        changeOrigin: true,
      },
      // /api/ 配下のリクエストを BFF（:8080）にプロキシする
      "/api": {
        target: "http://localhost:8080",
        // リバースプロキシ時に Host ヘッダーを書き換える
        changeOrigin: true,
      },
    },
  },
});
