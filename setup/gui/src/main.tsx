// Reactアプリケーションのエントリーポイント。DOMへのレンダリングを担当する。

// ReactのコアAPIをインポートする
import React from "react";
// ReactDOMのクライアントAPIをインポートする
import ReactDOM from "react-dom/client";
// ルートコンポーネントをインポートする
import App from "./App";

// ルートDOM要素を取得してReactアプリをマウントする
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  // StrictModeで開発時の潜在的な問題を早期検出する
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
