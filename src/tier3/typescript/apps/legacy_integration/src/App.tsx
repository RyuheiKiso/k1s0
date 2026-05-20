// k1s0 Legacy 連携 tier3 SPA ルートコンポーネント
// ERP 並行運用エントリポイント（レガシー .NET Framework 連携）のルーティングを定義する
// react-router-dom v6 の Routes / Route で宣言的にルートを管理する
// cookie scope: __Host-legacy-access-token（deployment 固有の cookie 分離）
// HTTP/1.1 + SSE legacy listener（8443-legacy ポート）との接続を担当する

// React をインポートする
import React from "react";
// BrowserRouter / Routes / Route / Navigate を react-router-dom からインポートする
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
// ERP 並行運用の entry point 画面をインポートする（legacy 連携 scenario）
import { LegacyBridgeScreen } from "./screens/LegacyBridgeScreen.js";

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="Legacy 連携 tier3 ホーム">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>Legacy 連携 tier3 — ERP 並行運用</h1>
      {/* 利用可能な機能の説明 */}
      <p>レガシー .NET Framework ERP との並行運用エントリポイントです。</p>
    </main>
  );
}

// ログイン画面コンポーネント（BFF 経由で認証する）
function LoginPage(): React.JSX.Element {
  // ログイン画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="ログイン">
      {/* ログイン見出し */}
      <h1>ログイン</h1>
      {/* BFF 経由で認証を開始するメッセージ */}
      <p>BFF auth-edge を経由して認証してください。</p>
      {/* BFF 認証開始リンク（__Host-legacy-access-token cookie を設定するエンドポイントへ遷移する） */}
      <a href="/auth/login" aria-label="BFF 経由でログインする">
        ログイン
      </a>
    </main>
  );
}

// 404 Not Found ページコンポーネント
function NotFoundPage(): React.JSX.Element {
  // 404 ページの最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="ページが見つかりません">
      {/* 404 見出し */}
      <h1>404 — ページが見つかりません</h1>
      {/* ホームへ戻るリンク */}
      <p>
        <a href="/">ホームへ戻る</a>
      </p>
    </main>
  );
}

// Legacy 連携 tier3 SPA のルートコンポーネント
export function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* Routes: 宣言的なルートマッチングコンテナ */}
      <Routes>
        {/* ルートパス: Legacy Bridge 画面へリダイレクトする */}
        <Route path="/" element={<Navigate to="/legacy-bridge" replace />} />
        {/* ERP 並行運用 entry point 画面（legacy 連携 scenario） */}
        <Route path="/legacy-bridge" element={<LegacyBridgeScreen />} />
        {/* ログイン画面（auth guard の外側に置く） */}
        <Route path="/login" element={<LoginPage />} />
        {/* ホーム画面（index）*/}
        <Route path="/home" element={<HomePage />} />
        {/* 404 フォールバック */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </BrowserRouter>
  );
}
