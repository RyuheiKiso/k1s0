// 本ファイルは tier3 Web Golden Path 最小 portal の React エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
// 関連 ID: ADR-DEV-001（Paved Road）

// React の createRoot API を使って Concurrent Mode で root をマウントする。
import { StrictMode } from "react";
// React 18 の root API（hydrate / createRoot）。
import { createRoot } from "react-dom/client";
// アプリ本体コンポーネント。
import { App } from "./App";

// index.html の <div id="root"> を取得する。null の場合は静的エラーで停止させる。
const container = document.getElementById("root");
// container が null（HTML テンプレートが壊れている）場合は早期に panic させる。
if (!container) {
    // 静的エラーとして throw することで開発者に index.html の問題を即座に伝える。
    throw new Error("k1s0 example portal: #root element not found in index.html");
}

// React 18 の createRoot で root を生成し、StrictMode でラップした App をマウントする。
createRoot(container).render(
    <StrictMode>
        <App />
    </StrictMode>,
);
