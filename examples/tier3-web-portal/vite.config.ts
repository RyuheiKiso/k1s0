// 本ファイルは tier3 Web Golden Path 最小 portal の Vite 設定。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
// 関連 ID: ADR-DEV-001（Paved Road）

// Vite 本体の defineConfig ヘルパで型安全な設定を書く。
import { defineConfig } from "vite";
// React 用 plugin（JSX 変換 / Fast Refresh）。
import react from "@vitejs/plugin-react";

// 設定をエクスポートする。最小骨格として React plugin のみ有効化。
export default defineConfig({
    // React plugin を登録する。
    plugins: [react()],
    // 開発サーバ既定ポート。Backstage / kind の port mapping と衝突しない範囲で設定する。
    server: {
        // ローカル開発の既定ポート（Vite 既定値）。
        port: 5173,
        // すべての NIC で listen（Dev Container からの接続を許容）。
        host: true,
    },
});
