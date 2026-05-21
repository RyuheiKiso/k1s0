// k1s0 検査 tier3 SPA ルートコンポーネント
// 検査ドメイン UI（品質検査 / 不良票 / 図面 review / 在庫）のルーティングを定義する
// react-router-dom v6 の Routes / Route で宣言的にルートを管理する
// cookie scope: __Host-inspection-access-token（deployment 固有の cookie 分離）

// React をインポートする
import React from "react";
// BrowserRouter / Routes / Route / Navigate を react-router-dom からインポートする
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
// 品質検査結果配信画面をインポートする（検査 scenario 9: 品質検査結果配信）
import { QualityInspectionScreen } from "./screens/QualityInspectionScreen.js";
// 不良票・警報配信画面をインポートする（検査 scenario 10: 不良票 / 警報配信）
import { DefectAlertScreen } from "./screens/DefectAlertScreen.js";
// 図面 collaborative review 画面をインポートする（検査 scenario 11: 図面 collaborative review）
import { DrawingReviewScreen } from "./screens/DrawingReviewScreen.js";
// 在庫最新値表示画面をインポートする（在庫 scenario 12: 在庫最新値表示）
import { InventoryScreen } from "./screens/InventoryScreen.js";

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="検査 tier3 ホーム">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>検査 tier3 — 検査ドメイン UI</h1>
      {/* 利用可能な機能の説明 */}
      <p>品質検査 / 不良票 / 図面 review / 在庫の業務 UI です。</p>
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
      {/* BFF 認証開始リンク（__Host-inspection-access-token cookie を設定するエンドポイントへ遷移する） */}
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

// 検査 tier3 SPA のルートコンポーネント
export function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* Routes: 宣言的なルートマッチングコンテナ */}
      <Routes>
        {/* ルートパス: 品質検査画面へリダイレクトする */}
        <Route path="/" element={<Navigate to="/quality-inspection" replace />} />
        {/* 品質検査結果配信画面（検査 scenario 9） */}
        <Route path="/quality-inspection" element={<QualityInspectionScreen />} />
        {/* 不良票・警報配信画面（検査 scenario 10） */}
        <Route path="/defect-alerts" element={<DefectAlertScreen />} />
        {/* 図面 collaborative review 画面（検査 scenario 11） */}
        <Route path="/drawing-review" element={<DrawingReviewScreen />} />
        {/* 在庫最新値表示画面（在庫 scenario 12） */}
        <Route path="/inventory" element={<InventoryScreen />} />
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
