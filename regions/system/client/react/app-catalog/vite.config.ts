import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { readFileSync, existsSync } from 'fs';
import { parse } from 'yaml';
import deepmerge from 'deepmerge';
import path from 'path';

/// 環境変数から実行環境を取得（デフォルト: development）
const env = process.env.APP_ENV ?? 'development';

/// ベース設定を YAML から読み込む
const base = parse(readFileSync('config/config.yaml', 'utf-8'));

/// 環境別オーバーレイ設定が存在する場合のみ読み込んでマージする
const overlayPath = `config/config.${env}.yaml`;
const overlay = existsSync(overlayPath) ? parse(readFileSync(overlayPath, 'utf-8')) : {};
const config = deepmerge(base, overlay);

// MED-18 監査対応: VITE_API_BASE_URL 環境変数が設定されている場合は YAML の base_url を上書きする。
// コンテナデプロイ時にビルド済みイメージのAPIエンドポイントを変更できるようにする。
if (process.env.VITE_API_BASE_URL) {
  config.api.base_url = process.env.VITE_API_BASE_URL;
  config.proxy.target = process.env.VITE_API_BASE_URL;
}

/// Viteビルド設定: YAML設定からプロキシとアプリ設定を注入（ポート5173を維持）
export default defineConfig({
  plugins: [react()],
  resolve: {
    // system-client パッケージへのエイリアス（相対パス依存を排除）
    alias: {
      'system-client': path.resolve(__dirname, '../system-client/src'),
    },
  },
  define: { __APP_CONFIG__: JSON.stringify(config) },
  server: {
    port: 5173,
    proxy: {
      [config.proxy.path]: { target: config.proxy.target, changeOrigin: true },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './tests/setup.ts',
    css: true,
  },
});
