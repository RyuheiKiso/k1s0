# ADR-TIER3-002: tier3 Web を React + Vite SPA + Go BFF で構成する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: tier3 Web 開発チーム / 採用検討組織 / DevEx チーム

## コンテキスト

ADR-TIER3-001 で「tier3 client ごとに専用 BFF を配置する」決定をした上で、tier3 Web 側の具体的な実装方式を決める必要がある。Web フロントエンドの技術選定は採用組織の Web エンジニア人材プールと直結し、ここを誤ると採用初期から「フロントエンド人材を集めにくい」「学習コストが高い」という問題が発生する。

現代の Web フロントエンドの選択肢は大きく 3 系統に分けられる。

1. **SPA（Single Page Application）**: ブラウザで JavaScript が画面遷移を含む全描画を担う（React / Vue / Svelte / Solid）
2. **SSR（Server-Side Rendering）+ ハイドレーション**: サーバで初回 HTML を生成しブラウザで JS が引き継ぐ（Next.js / Remix / Nuxt / SvelteKit / Astro）
3. **MPA（Multi Page Application）**: 古典的な ページ単位 HTTP（Spring MVC / Rails ERB / Django Templates）

加えて k1s0 の前提として、

- **採用組織の人材プール**: 日本企業で標準的に得られるフロントエンド人材は React + TypeScript の組み合わせが最大プール。Vue は一定数、Svelte / Solid は少数
- **業務 UI の特性**: portal（テナント業務 UI）/ admin（運用管理 UI）/ docs-site（ドキュメント閲覧）。docs-site 以外は SEO 不要、認証必須
- **BFF パターン（ADR-TIER3-001）との整合**: BFF が GraphQL + REST を出すため、Web 側はそれを消費する client が必要
- **gRPC-Web translator が tier1 直アクセスでは必要**（SHIP_STATUS F2）だが、BFF を介すれば translator は BFF 内に閉じる
- **i18n 必須**（ja / en）

選択は採用組織の Web 開発者体験 / リクルーティング / 10 年保守に直接影響するため、**two-way door** 寄りだが移行コスト大。リリース時点で確定する。

## 決定

**tier3 Web は React + Vite SPA + 共有パッケージ（pnpm workspace）+ Go BFF（GraphQL + REST）の構成を採用する。**

- React 18+ + TypeScript + Vite（ビルドツール、開発サーバ）
- pnpm workspace で `apps/{portal,admin,docs-site}` と `packages/{ui,api-client,i18n,config}` の monorepo 構成
- 状態管理は React Hooks + 必要に応じて軽量ライブラリ（global state は最小）
- API client は BFF の GraphQL（複合 query 用）+ REST（CRUD 単純系）を併用、`packages/api-client/` で統一インタフェース
- i18n は `packages/i18n/` で ja / en の minimum 対応
- ビルド成果物は nginx-distroless で配信、`/api/*` は同一 host 上の BFF にリバースプロキシ（CORS 不要、`deploy/charts/tier3-web-app/`）
- vitest で単体テスト、Playwright で E2E（採用初期）
- SSR（Next.js / Remix）は採用しない。docs-site のみ Astro による pre-rendering を将来検討（採用後の運用拡大時）

`src/tier3/web/apps/{portal,admin,docs-site}/` と `src/tier3/web/packages/{ui,api-client,i18n,config}/` で確定（既存実装あり、SHIP_STATUS § tier3）。

## 検討した選択肢

### 選択肢 A: React + Vite SPA + Go BFF（採用）

- 概要: ブラウザ側の JS で完結する SPA、サーバは静的 nginx + BFF
- メリット:
  - **React + TypeScript は採用組織が確保できる人材プール最大**
  - Vite は esbuild ベースで起動 / HMR が高速、開発体験が良好
  - SSR 不要のため運用が単純（静的ファイル + BFF のみ、Node.js ランタイムを web 側に持たない）
  - pnpm workspace で apps / packages を共通化、Lerna / Nx より軽量
  - vitest が Vite と統合され、Jest より高速
- デメリット:
  - SEO が必要なページに向かない（k1s0 は portal / admin が SEO 不要、docs-site は将来 pre-rendering で対応）
  - 初回ロードが SSR より遅い（CDN キャッシュで軽減可能）
  - BFF への直接依存（BFF 障害時に Web も停止）

### 選択肢 B: Next.js（SSR）+ API Routes

- 概要: Next.js を BFF と SSR の両方として使う
- メリット:
  - SEO / 初回ロード性能が良い
  - フレームワーク統合（Routing / API / 状態管理）
  - Vercel / Netlify 等のエコシステム
- デメリット:
  - **Next.js 自身が BFF も兼ねるため、ADR-TIER3-001 の責務分離（Web 用 BFF は Go）と乖離**
  - SSR 用の Node.js ランタイムが Web 配信側に必要、運用 component が増える
  - Next.js のメジャーバージョン追従コスト（App Router / Pages Router の互換性破壊）
  - Vercel に最適化された機能（ISR / Edge Functions）はオンプレでは活用困難

### 選択肢 C: Remix（SSR）

- 概要: React Router 系の SSR フレームワーク
- メリット:
  - Web 標準志向（fetch / FormData / Response）
  - React 親和性
- デメリット:
  - Shopify 買収後の方向性が不透明
  - 採用組織の人材プールが Next.js より小さい
  - SSR ランタイムが必要な点は B と同じ

### 選択肢 D: Vue + Vite

- 概要: Vue 3 + Vite SPA
- メリット:
  - 学習曲線が React より緩やか
  - Composition API で TypeScript 親和性向上
- デメリット:
  - **採用組織の人材プールが React より小さい**
  - エコシステム（component ライブラリ / 状態管理 / テスト）の多様性が React に劣る

### 選択肢 E: Svelte / SvelteKit

- 概要: コンパイル時に最適化される反応性フレームワーク
- メリット: bundle size 小、性能良好
- デメリット:
  - 採用組織の人材プールが React / Vue よりさらに小さい
  - 業界での採用事例が日本企業では少なく、10 年保守の人材確保リスク

### 選択肢 F: Astro

- 概要: 静的 site / 部分的 hydration 向けフレームワーク
- メリット: docs-site のような静的中心 UI に最適
- デメリット:
  - 業務 UI（portal / admin）には不向き（インタラクティブ性が要る）
  - 採用組織の業務 UI を全部 Astro にするのは無理がある

### 選択肢 G: MPA（Spring MVC / Rails / Django）

- 概要: サーバ側で完全レンダリング、JS は最小限
- メリット: 古典的、シンプル
- デメリット:
  - **業務 UI に必要な動的インタラクション（リアルタイム更新 / 複雑フォーム）が実装困難**
  - tier3 が backend 言語に縛られる（Go / .NET ではない）
  - 採用組織の Web 開発体験が劣化

## 決定理由

選択肢 A（React + Vite SPA + Go BFF）を採用する根拠は以下。

- **採用組織の人材プール最大**: 日本企業で標準的に得られるフロントエンド人材は React + TypeScript の組み合わせが最大。Vue（D）/ Svelte（E）は人材確保で 10 年保守のリスクが高い
- **責務分離との整合**: ADR-TIER3-001 で BFF を Go で確立した以上、Web 側は SPA で BFF を消費する構造が責務分離に整合する。Next.js（B）/ Remix（C）は自身が BFF を兼ねるため、ADR-TIER3-001 と重複が発生する
- **業務 UI 特性との適合**: portal / admin は SEO 不要・認証必須・インタラクティブ。SPA はこの特性に最適。SEO が要る docs-site のみ Astro pre-rendering を将来検討
- **Vite の開発体験**: esbuild ベースの起動 / HMR が高速、Webpack 系ビルドツールよりも体感的に大幅に速い。採用組織の開発者体験を最大化（ADR-DEV-001 Paved Road の趣旨に整合）
- **monorepo 構造の妥当性**: pnpm workspace は Lerna / Nx より軽量で、apps（3 SPA）+ packages（4 共通ライブラリ）の規模に最適。10 年保守の前提で重い monorepo ツールに依存しないのは合理的
- **SSR ランタイムの不採用**: Next.js / Remix の SSR を採用すると、Web 配信側に Node.js ランタイムを必要とし、運用 component が増え kind / production の差分も増える。SPA + 静的 nginx + BFF は運用 component 数を最小化する
- **退路の確保**: 将来 SSR が必要になれば、docs-site のみ Astro 等の pre-rendering / 部分 hydration を導入する経路を残す。portal / admin の業務 UI を SSR に移行する選択は当面想定しないが、React + TypeScript ベースであれば Next.js / Remix への移植コストは限定的

## 帰結

### ポジティブな帰結

- 採用組織のフロントエンド人材確保が容易（React + TypeScript の最大プール）
- Vite + vitest による開発 / テスト体験が良好
- ADR-TIER3-001 の BFF 責務分離と整合、Go BFF との clean な切り分け
- pnpm workspace で apps / packages を統一管理、Backstage Software Template で雛形展開可能（ADR-DEV-001）
- 静的ファイル + BFF のみで運用 component 最小化

### ネガティブな帰結 / リスク

- SEO / 初回ロード性能が SSR より劣る。docs-site で SEO が要件化したら Astro pre-rendering を別途検討
- 大量 JS を初回ロードする際の Time to Interactive。code splitting / lazy loading で軽減
- React のメジャーバージョン追従コスト（19 → 20 等）。Vite と shared package layer で局所化
- BFF 障害時の Web 停止（BFF 側で Circuit Breaker / Fallback を実装する必要、IMP-OBS-* で運用設計）

### 移行・対応事項

- `src/tier3/web/apps/{portal,admin,docs-site}/` と `packages/{ui,api-client,i18n,config}/` の構成を確定（既存実装あり）
- `deploy/charts/tier3-web-app/` で nginx-distroless + SPA fallback + `/api/` reverse proxy の標準 chart を提供
- vitest 単体 + Playwright E2E のテンプレート整備
- React / Vite / pnpm のバージョン追従手順を Runbook 化
- code splitting / lazy loading のガイドラインを `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/` に明文化

## 関連

- ADR-TIER3-001（BFF パターン）— BFF を Go で実装、本 ADR で SPA 側を確定
- ADR-TIER3-003（.NET MAUI Native）— Native 側は別構成
- ADR-DEV-001（Paved Road）— Backstage Software Template との整合
- ADR-SEC-001（Keycloak）— OIDC ログインフロー（BFF 経由）
- IMP-DIR-INFRA-* — `src/tier3/web/` 配置

## 参考文献

- React 公式: react.dev
- Vite 公式: vitejs.dev
- pnpm Workspace: pnpm.io/workspaces
- ThoughtWorks Technology Radar
