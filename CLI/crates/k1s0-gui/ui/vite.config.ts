import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  // Tauri expects a fixed port
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    // M-26 監査対応: rolldown 1.0.0-rc.10 の日本語バイト境界パニック回避。
    // hash_placeholder.rs:56 で日本語文字（3バイト UTF-8）を含むチャンクに対して
    // バイトオフセット計算が誤り Rust パニックが発生する。
    // Tauri デスクトップアプリはローカルファイルシステムから配信するため、
    // CDN キャッシュバスティング目的のコンテンツハッシュは不要。
    // チャンクファイル名のハッシュを除去することで hash_placeholder.rs を経由しなくなる。
    rolldownOptions: {
      output: {
        // [hash] を除去してハッシュプレースホルダー処理を回避する
        chunkFileNames: 'assets/[name].js',
        entryFileNames: '[name].js',
        assetFileNames: 'assets/[name][extname]',
      },
    },
  },
})
