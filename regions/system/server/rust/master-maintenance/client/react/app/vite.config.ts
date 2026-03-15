import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync, existsSync } from "fs";
import { parse } from "yaml";
import deepmerge from "deepmerge";

/// 環境変数から実行環境を取得（デフォルト: development）
const env = process.env.APP_ENV ?? "development";

/// ベース設定を YAML から読み込む
const base = parse(readFileSync("config/config.yaml", "utf-8"));

/// 環境別オーバーレイ設定が存在する場合のみ読み込んでマージする
const overlayPath = `config/config.${env}.yaml`;
const overlay = existsSync(overlayPath) ? parse(readFileSync(overlayPath, "utf-8")) : {};
const config = deepmerge(base, overlay);

/// Viteビルド設定: YAML設定からプロキシとアプリ設定を注入（ポート3011を維持）
export default defineConfig({
  plugins: [react()],
  define: { __APP_CONFIG__: JSON.stringify(config) },
  server: {
    port: 3011,
    proxy: {
      [config.proxy.path]: { target: config.proxy.target, changeOrigin: true },
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./tests/setup.ts",
    css: true,
  },
});
