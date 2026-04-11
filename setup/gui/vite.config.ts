// Viteビルドツールの設定ファイル。TauriとReactとの連携設定およびVitest設定を定義する。

// Vitestのグローバル型定義を参照する
/// <reference types="vitest" />
// ViteのdefineConfig関数をインポートする
import { defineConfig } from "vite";
// ReactプラグインをインポートするためにViteプラグインをロードする
import react from "@vitejs/plugin-react";

// Viteの設定をエクスポートする
export default defineConfig(async () => {
  // Tauri開発ホストが設定されている場合のHMR設定を構築する
  const hmrConfig = process.env.TAURI_DEV_HOST
    ? {
        // WebSocketプロトコルを指定する
        protocol: "ws" as const,
        // Tauri開発ホストのアドレスを指定する
        host: process.env.TAURI_DEV_HOST,
        // HMR用WebSocketポートを指定する
        port: 1421,
      }
    : undefined;

  return {
    // Vitestのテスト実行環境を設定する
    test: {
      // jsdomを使ってブラウザ相当の環境でテストを実行する
      environment: "jsdom",
      // jest-domマッチャーをグローバルにセットアップする
      setupFiles: ["./src/test/setup.ts"],
      // describe/it/expectをグローバルに利用可能にする
      globals: true,
    },
    // Reactプラグインを有効にする
    plugins: [react()],
    // Viteのコンソールクリアを無効にしてTauriのログと共存させる
    clearScreen: false,
    // 開発サーバーの設定を定義する
    server: {
      // Tauriが使用するデフォルトポートを設定する
      port: 1420,
      // 指定ポートが使用中の場合はエラーを発生させる
      strictPort: true,
      // 環境変数からTauri開発ホストを取得する
      host: process.env.TAURI_DEV_HOST || false,
      // HMR設定を適用する
      hmr: hmrConfig,
      // ファイル監視設定を定義する
      watch: {
        // Tauriのソースファイル変更はViteの監視対象から除外する
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
