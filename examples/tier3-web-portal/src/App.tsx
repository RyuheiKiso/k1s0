// 本ファイルは tier3 Web Golden Path 最小 portal の本体コンポーネント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
// 関連 ID: ADR-DEV-001（Paved Road）/ ADR-FM-001（OpenFeature / flagd）
//
// 役割:
//   tier3 Web の「最小だが本番形」を示す。具体的には
//     - React 18 / TS strict / 単一 root コンポーネント
//     - tier1 facade への疎通を後で差し替え可能なフックパタン
//   までを 1 ファイルで読み切れる範囲に収める。
//
// scope（リリース時点）: ホームページ 1 枚（静的テキスト + tier1 health 想定の placeholder）
// 採用初期で拡張: React Router / @k1s0/sdk 連携 / Keycloak OIDC / i18n / Playwright E2E

// Hook（useState）で health 表示の placeholder 値を持つ。
import { useState } from "react";

/**
 * App: 最小 portal のルートコンポーネント。
 *
 * リリース時点では tier1 への実通信を行わず、placeholder の health 値を表示する。
 * 採用初期で `@k1s0/sdk` の HealthService.Check 呼び出しに置換される。
 */
export function App(): JSX.Element {
    // tier1 health の表示文字列を保持する state。採用初期で SDK 呼び出し結果に差し替える。
    const [health] = useState<string>("placeholder (採用初期で @k1s0/sdk 経由の実 check に置換)");

    // 最小 portal の表示。視覚的装飾は採用初期に追加する想定で抑える。
    return (
        <main style={{ fontFamily: "system-ui, sans-serif", padding: "2rem", maxWidth: 720 }}>
            <h1>k1s0 example portal</h1>
            <p>
                Golden Path: tier3 Web 最小例。React + Vite + TypeScript の Paved Road を示す
                サンプル。
            </p>
            <section>
                <h2>tier1 health</h2>
                <p>
                    <code>{health}</code>
                </p>
            </section>
            <section>
                <h2>次のステップ</h2>
                <ul>
                    <li>
                        <code>@k1s0/sdk</code> をインストールして HealthService.Check を呼び出す
                    </li>
                    <li>React Router で複数ルートを追加する</li>
                    <li>Keycloak OIDC を統合する（ADR-SEC-001）</li>
                </ul>
            </section>
        </main>
    );
}
