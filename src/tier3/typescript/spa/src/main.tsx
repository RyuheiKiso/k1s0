// k1s0 tier3 SPA エントリポイント（React 18）
// createRoot + StrictMode で React 18 の concurrent モードを有効化する
// auth guard / i18n / 4 layer state / axe-core / OTel を統合する

// React の createRoot API をインポートする
import { createRoot } from "react-dom/client";
// React の StrictMode と useState / useEffect をインポートする
import React, { StrictMode, useState, useEffect } from "react";
// BrowserRouter と Route を react-router-dom からインポートする
import { BrowserRouter, Route, Routes, Navigate } from "react-router-dom";

// BFF クライアントをインポートする（access_token を tier3 に公開しない設計）
import { defaultBffClient } from "../../packages/auth/src/index.js";
// BFF の AuthState 型をインポートする
import type { AuthState, AuthCheckResult } from "../../packages/auth/src/index.js";
// silent renew ループをインポートする
import { startSilentRenew, DEFAULT_SILENT_RENEW_CONFIG } from "../../packages/auth/src/index.js";
// logout ブロードキャスト受信をインポートする
import { listenLogoutBroadcast } from "../../packages/auth/src/index.js";

// i18n パッケージをインポートする（locale 初期化）
import {
  type SupportedLocale,
  isSupportedLocale,
  JA_JP_CONFIG,
} from "../../packages/i18n/src/index.js";
// dict_loader は DictLocale 型（ja_JP / en_US）を使用する（index.ts の SupportedLocale と別）
import { loadDict, type DictLocale } from "../../packages/i18n/src/dict_loader.js";

// 4 layer state パッケージをインポートする（reducer 初期化）
import { createInitialState } from "../../packages/state/src/index.js";
// ClientState 型をインポートする
import type { ClientState } from "../../packages/state/src/index.js";

// OTel Browser SDK をインポートする（ページロードスパン生成）
import { initOtelBrowser } from "./observability/otel_browser.js";

// production/development 分岐禁止規約: 環境によって動作が変わるコードは禁止する
// OTel Browser SDK を初期化する（main.tsx 最初期に実行する）
initOtelBrowser({
  // サービス名を設定する
  serviceName: "k1s0-tier3-spa",
  // サービスバージョンを設定する（ビルド時に環境変数から注入する）
  serviceVersion: "0.1.0",
  // OTel Collector エンドポイント（BFF 経由）
  collectorUrl: "/api/otel/v1/traces",
  // テナント slug は初期化時点では不明（auth 後に更新する）
  tenantSlug: "unknown",
  // 環境名は "production" 固定（production/development 分岐禁止規約）
  environment: "production",
  // サンプリングレート（production 固定: 10%）
  samplingRate: 0.1,
});

// locale の初期値を navigator.language から決定する
function detectInitialLocale(): SupportedLocale {
  // navigator.language が利用可能かチェックする
  const browserLocale =
    typeof navigator !== "undefined" ? navigator.language : "ja-JP";
  // サポート locale に含まれるか確認する
  if (isSupportedLocale(browserLocale)) {
    // ブラウザ設定の locale を使用する
    return browserLocale;
  }
  // フォールバック: ja-JP を使用する
  return "ja-JP";
}

// BFF auth check 結果を AuthState に変換する
function toAuthState(result: AuthCheckResult): AuthState {
  // result の status に応じて AuthState を生成する
  if (result.status === "authenticated") {
    // 認証済み状態を返す
    return {
      authenticated: true,
      actorDisplayName: result.displayName,
      sessionExpiresAt: result.expiresAt,
    };
  }
  // 未認証 / step-up 必要の場合は未認証状態を返す
  return {
    authenticated: false,
    actorDisplayName: null,
    sessionExpiresAt: null,
  };
}

// auth guard HOC（未認証時は /login にリダイレクトする）
function AuthGuard({ children }: { readonly children: React.ReactNode }): React.JSX.Element {
  // auth state を state として保持する
  const [authState, setAuthState] = useState<AuthState | null>(null);
  // BFF auth check の初期化済みフラグ
  const [authChecked, setAuthChecked] = useState(false);

  // マウント時に BFF auth check を実行する
  useEffect(() => {
    // BFF auth check を実行する
    defaultBffClient
      .check()
      .then((result: AuthCheckResult) => {
        // auth state を更新する
        setAuthState(toAuthState(result));
        // auth check 完了フラグを設定する
        setAuthChecked(true);
      })
      .catch(() => {
        // auth check 失敗は未認証とみなす
        setAuthState({ authenticated: false, actorDisplayName: null, sessionExpiresAt: null });
        // auth check 完了フラグを設定する
        setAuthChecked(true);
      });
  }, []);

  // auth check 完了前はローディング表示する
  if (!authChecked) {
    // ローディング中の aria-live 通知
    return (
      <div role="status" aria-live="polite" aria-label="認証確認中">
        <p>認証確認中...</p>
      </div>
    );
  }

  // 未認証の場合は /login にリダイレクトする
  if (authState === null || !authState.authenticated) {
    // React Router の Navigate コンポーネントでリダイレクトする
    return <Navigate to="/login" replace />;
  }

  // 認証済みの場合は children を表示する
  return <>{children}</>;
}

// i18n provider（locale 初期化と辞書ロードを担当する）
function I18nProvider({ children }: { readonly children: React.ReactNode }): React.JSX.Element {
  // locale state を保持する
  const [locale] = useState<SupportedLocale>(detectInitialLocale);
  // 辞書ロード完了フラグ
  const [dictLoaded, setDictLoaded] = useState(false);

  // SupportedLocale を DictLocale に変換するヘルパー（"ja-JP" → "ja_JP"）
  function toDictLocale(loc: SupportedLocale): DictLocale {
    // BCP 47 のハイフンをアンダースコアに変換して DictLocale 形式にする
    return loc.replace("-", "_") as DictLocale;
  }

  // マウント時に辞書をロードする
  useEffect(() => {
    // 検出した locale を DictLocale 形式に変換して辞書を動的ロードする
    loadDict(toDictLocale(locale))
      .then(() => {
        // 辞書ロード完了フラグを設定する
        setDictLoaded(true);
      })
      .catch(() => {
        // 辞書ロード失敗は警告のみ（フォールバック動作を続ける）
        console.warn(`[i18n] Failed to load dictionary for locale: ${locale}`);
        // 失敗時もフラグを設定して UI をブロックしない
        setDictLoaded(true);
      });
  }, [locale]);

  // 辞書ロード完了前はローディング表示する
  if (!dictLoaded) {
    // locale ロード中の aria-live 通知
    return (
      <div role="status" aria-live="polite" aria-label="言語設定を初期化中">
        <p>言語設定を初期化中...</p>
      </div>
    );
  }

  // 辞書ロード完了後は children を表示する
  return <>{children}</>;
}

// アプリケーション全体の state provider
function StateProvider({
  children,
}: {
  readonly children: React.ReactNode;
}): React.JSX.Element {
  // 4 layer state を初期化して state として保持する
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [_clientState] = useState<ClientState<unknown>>(() =>
    // createInitialState で 4 layer を null / 空で初期化する
    createInitialState<unknown>(),
  );

  // silent renew ループを開始する（マウント時）
  useEffect(() => {
    // silent renew ループを開始する（5 分ごとにセッション状態をチェックする）
    const renewHandle = startSilentRenew({
      // BFF クライアントを設定する
      bffClient: defaultBffClient,
      // チェック間隔とマージンはデフォルト値を使用する
      checkIntervalMs: DEFAULT_SILENT_RENEW_CONFIG.checkIntervalMs,
      renewMarginMs: DEFAULT_SILENT_RENEW_CONFIG.renewMarginMs,
      // renew 失敗時はコンソール警告のみ（logout 誘導は上位コンポーネントで行う）
      onFailed: (reason: string) => {
        // renew 失敗理由をコンソールに出力する
        console.warn("[auth] silent renew failed:", reason);
      },
    });
    // logout ブロードキャストを受信するリスナーを登録する
    const logoutHandle = listenLogoutBroadcast(() => {
      // 全タブ logout 通知を受信したらページをリロードして auth guard に委ねる
      window.location.reload();
    });
    // アンマウント時にループとリスナーを停止する
    return () => {
      // silent renew ループを停止する
      renewHandle.stop();
      // logout ブロードキャストリスナーを停止する
      logoutHandle.stop();
    };
  }, []);

  // children をそのまま返す（state は context 経由で子に伝播する実装は別途追加する）
  return <>{children}</>;
}

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: a11y ランドマーク
    <main id="main-content" role="main" aria-label="ホーム画面">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>k1s0 tier3 SPA</h1>
      {/* 初期化完了メッセージ */}
      <p>4 layer client state が初期化されました。</p>
    </main>
  );
}

// ログイン画面コンポーネント（BFF 経由で認証する）
function LoginPage(): React.JSX.Element {
  // ログイン画面の最小 JSX を返す
  return (
    // main 要素: a11y ランドマーク
    <main id="main-content" role="main" aria-label="ログイン">
      {/* ログイン見出し */}
      <h1>ログイン</h1>
      {/* BFF 経由で認証を開始するメッセージ */}
      <p>BFF auth-edge を経由して認証してください。</p>
      {/* BFF 認証開始リンク（BFF の OIDC initiation エンドポイントへ遷移する） */}
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
    // main 要素: a11y ランドマーク
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

// アプリケーションのルートコンポーネントを定義する
function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* I18nProvider: locale 初期化と辞書ロードを担当する */}
      <I18nProvider>
        {/* StateProvider: 4 layer state を初期化する */}
        <StateProvider>
          {/* Routes: 宣言的なルートマッチングコンテナ */}
          <Routes>
            {/* ルートパス: auth guard で保護されたホーム画面 */}
            <Route
              path="/"
              element={
                // AuthGuard: 未認証時は /login にリダイレクトする
                <AuthGuard>
                  <HomePage />
                </AuthGuard>
              }
            />
            {/* ログイン画面（auth guard の外側に置く） */}
            <Route path="/login" element={<LoginPage />} />
            {/* 404 フォールバック */}
            <Route path="*" element={<NotFoundPage />} />
          </Routes>
        </StateProvider>
      </I18nProvider>
    </BrowserRouter>
  );
}

// DOM から root 要素を取得する
const rootElement = document.getElementById("root");
// root 要素が存在しない場合は例外を投げる
if (rootElement === null) {
  // 開発者向けエラーメッセージを設定する
  throw new Error(
    'Root element (#root) not found. index.html に <div id="root"></div> が必要です。',
  );
}

// locale 設定を document の lang 属性に反映する（a11y: スクリーンリーダー向け）
document.documentElement.lang =
  JA_JP_CONFIG.locale;

// React 18 の createRoot API で root を生成する
const root = createRoot(rootElement);
// StrictMode で App をレンダリングする（開発時の副作用二重呼び出し検知を有効化する）
root.render(
  // StrictMode: React の開発ヘルパー（本番ビルドでは無効化される）
  <StrictMode>
    {/* アプリケーションルートコンポーネントをマウントする */}
    <App />
  </StrictMode>,
);
