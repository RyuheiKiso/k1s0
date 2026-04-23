# 02. Web pnpm workspace 配置

本ファイルは `src/tier3/web/` 配下の React + TypeScript pnpm workspace 構成を確定する。Shopify / Airbnb のモノレポ事例に倣い、`apps/` と `packages/` の 2 カテゴリで整理する。

## レイアウト

```
src/tier3/web/
├── README.md
├── package.json                    # root package（scripts / private）
├── pnpm-workspace.yaml             # apps/* と packages/* を含む
├── pnpm-lock.yaml
├── tsconfig.base.json              # 共通 tsconfig
├── .npmrc                          # registry 設定
├── .eslintrc.cjs                   # 共通 eslint 設定
├── .prettierrc                     # 共通 prettier 設定
├── turbo.json                      # Phase 1b 以降検討（Turborepo）
├── apps/
│   ├── portal/                     # 配信ポータル
│   │   ├── package.json
│   │   ├── next.config.ts
│   │   ├── tsconfig.json
│   │   ├── src/
│   │   │   ├── app/                # Next.js App Router
│   │   │   ├── components/
│   │   │   ├── lib/
│   │   │   └── styles/
│   │   ├── public/
│   │   ├── Dockerfile
│   │   └── e2e/                    # Playwright
│   ├── admin/                      # 管理画面
│   │   └── ...                     # portal と同構造
│   └── docs-site/                  # Phase 1b 以降のドキュメントサイト
│       └── ...
├── packages/
│   ├── ui/                         # shadcn/ui 派生の共通コンポーネント
│   │   ├── package.json
│   │   ├── src/
│   │   │   ├── components/
│   │   │   └── hooks/
│   │   └── dist/                   # ビルド成果物（.gitignore）
│   ├── api-client/                 # k1s0 SDK wrapper
│   │   ├── package.json
│   │   └── src/
│   │       ├── client.ts
│   │       └── hooks/
│   ├── i18n/                       # 国際化基盤
│   │   ├── package.json
│   │   └── src/
│   │       ├── locales/
│   │       │   ├── ja/
│   │       │   └── en/
│   │       └── i18n.ts
│   └── config/                     # 共通設定
│       ├── package.json
│       └── src/
└── tools/
    └── eslint-config/              # 共通 eslint 設定（package として公開）
        ├── package.json
        └── index.js
```

## pnpm-workspace.yaml の推奨

```yaml
packages:
  - 'apps/*'
  - 'packages/*'
  - 'tools/*'
```

apps / packages / tools の 3 カテゴリで workspace 参加。

## 各カテゴリの役割

### apps/

起動可能なアプリケーション。`package.json` に `"private": true` を設定し、npm publish の対象外とする。

- `portal/`: エンドユーザ向け配信ポータル（Phase 1a の主要成果物）
- `admin/`: 管理画面（テナント管理・監査ログ閲覧）
- `docs-site/`: ドキュメントサイト（Phase 1b 以降、Docusaurus or VitePress）

### packages/

再利用可能な library。workspace 内部で `"workspace:*"` で参照される。Phase 1c 以降に外部 publish を検討する場合は `"private": false` に切り替える。

- `ui/`: Button / Dialog / Form などの共通コンポーネント
- `api-client/`: `@k1s0/sdk`（`src/sdk/typescript/`）をラップし、React Query などと統合
- `i18n/`: i18next ベースの国際化基盤
- `config/`: 環境変数・設定ロード共通化

### tools/

Lint / Formatter 設定を package 化したもの。

- `eslint-config/`: 共通 ESLint 設定（airbnb base + typescript + react hooks）

#### なぜ `tools/` であり `packages/` でも `<repo>/tools/` でもないか

`src/tier3/web/tools/eslint-config/` に置く理由は、以下の 3 つの選択肢を比較した結果の折衷である。

1. **`src/tier3/web/packages/eslint-config/`** — 公開 npm package と並列。pnpm workspace の依存解決としては最も自然だが、「package = 実行時 artifact」という暗黙のメンタルモデルを壊す（eslint-config は devDependency 専用）。将来 `packages/*` を一括 publish する場合に除外条件が必要になるリスクあり
2. **リポジトリルート `tools/eslint-config/`**（横断ツール） — リポジトリ全体の ESLint 設定を集約できるが、tier3-web cone から参照するのにリポジトリルートを cone に追加することになり、sparse-checkout の境界を跨ぐ。Go / .NET / Rust の開発者には不要な TypeScript 設定が混入する
3. **`src/tier3/web/tools/eslint-config/`** — workspace 内部で参照され、tier3-web cone に自然に含まれ、かつ `packages/*` の「公開候補」から切り離される（採用）

`.eslintrc.cjs` は `extends: ['@k1s0/eslint-config']` で `tools/eslint-config/` を参照し、pnpm workspace の `workspace:*` 依存で解決する。外部公開しないため `"private": true` を明示する。

## 依存方向

Phase 1a は BFF 経由のみ、Phase 1b 以降で直 gRPC-Web も許容するため、`packages/api-client` の下流が Phase により分岐する。

```
                             （Phase 1a）
apps/*  →  packages/api-client  ─────►  BFF（HTTP/REST・GraphQL）
            │                             → tier2 / tier1（SDK 経由）
            │     （Phase 1b 以降、選択的に）
            └─────►  @k1s0/sdk（src/sdk/typescript/）── gRPC-Web ──► tier1

apps/*  →  packages/ui / i18n / config
apps/*  →  tools/eslint-config（devDependency）
```

- Phase 1a: `apps/*` は `packages/api-client` を通じて BFF の REST / GraphQL のみを叩く。`@k1s0/sdk` は SDK 側の構造確立のみで、Web からは使わない
- Phase 1b 以降: 一部 API（軽量 read-only など）で `packages/api-client` が `@k1s0/sdk` の gRPC-Web クライアントを直接使う構成も許容
- apps 間の相互依存は禁止。共通ロジックは packages/ に移動する。packages 間の依存は許容するが、循環は禁止
- `apps/*` から `@k1s0/sdk` を直接依存することは禁止（api-client 経由で抽象化し、Phase 1a/1b の切替をまとめて行えるようにする）

## ビルドとキャッシュ

### Phase 1a: 素の pnpm + tsc

各 package は `pnpm build` で TypeScript を compile。apps は Next.js / Vite の CLI で build。CI は `pnpm --filter <pkg>...` で変更影響範囲のみビルド。

### Phase 1b 以降: Turborepo 検討

Turborepo 導入により、ビルド依存関係グラフとキャッシュ（remote cache）を活用する。`turbo.json` で `build` / `test` / `lint` タスクの依存関係を宣言する。

```json
{
  "$schema": "https://turbo.build/schema.json",
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**", ".next/**"]
    },
    "test": {
      "dependsOn": ["^build"],
      "outputs": []
    },
    "lint": {
      "dependsOn": []
    }
  }
}
```

Phase 1b の規模（apps 3 個 + packages 10 個超）で turbo 導入の CI 時間短縮効果を判定する。

## Dockerfile

portal / admin の Docker image は Next.js standalone output 前提で構成する。`next.config.ts` に `output: 'standalone'` を宣言した上で、runtime ステージには standalone bundle のみを同梱する（外部 `node_modules` は不要、依存は bundle 内に組み込み済み）。

```dockerfile
# apps/portal/Dockerfile
# 前提: apps/portal/next.config.ts で `output: 'standalone'` を有効化していること
# build context: リポジトリルートからではなく、src/tier3/web/ をルートとして
#   `docker build -f apps/portal/Dockerfile .` を実行する
FROM node:20-alpine AS builder
WORKDIR /workspace
RUN npm install -g pnpm@9
COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY apps/portal/package.json apps/portal/
COPY packages/ packages/
COPY tools/ tools/
RUN pnpm install --frozen-lockfile --filter '@k1s0/portal...'

COPY apps/portal/ apps/portal/
RUN pnpm --filter '@k1s0/portal' build

FROM node:20-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
# Next.js standalone: 依存は standalone 配下に同梱済み。node_modules を別途 COPY しない。
COPY --from=builder /workspace/apps/portal/.next/standalone ./
COPY --from=builder /workspace/apps/portal/.next/static ./.next/static
COPY --from=builder /workspace/apps/portal/public ./public
USER node
EXPOSE 3000
# standalone は server.js をルート直下に吐くため、パスは ".next/standalone/..." ではなく "server.js"
CMD ["node", "server.js"]
```

`next.config.ts` 側の最小宣言例:

```ts
// apps/portal/next.config.ts
import type { NextConfig } from 'next';

const config: NextConfig = {
  output: 'standalone',
  // pnpm workspace 依存を standalone bundle に含めるため、外部の依存解決root を明示
  outputFileTracingRoot: require('path').join(__dirname, '../../'),
};

export default config;
```

## gRPC-Web との連携

tier1 公開 API は gRPC。Web からは以下のいずれかで呼び出す。

- **gRPC-Web（Envoy 経由）**: tier1 の前に Envoy / Istio Ingress が gRPC-Web → gRPC プロキシを行う
- **BFF 経由**: `src/tier3/bff/` が複合クエリを gRPC で tier1 に流し、REST / GraphQL で Web に返す

Phase 1a は BFF 経由のみ。Phase 1b 以降で直 gRPC-Web も提供する。

## TypeScript 設定

`tsconfig.base.json` で厳格モードを有効化。

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": true,
    "verbatimModuleSyntax": true,
    "allowImportingTsExtensions": false,
    "resolveJsonModule": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "jsx": "preserve"
  }
}
```

各 package / app の tsconfig.json は `extends: "../../tsconfig.base.json"` で継承。

## ESLint / Prettier

共通設定を `tools/eslint-config/` に集約。各 package / app は extends で継承。

## テスト戦略

- unit test: Vitest または Jest
- component test: Testing Library
- e2e: Playwright（`apps/<app>/e2e/` に配置）

## 対応 IMP-DIR ID

- IMP-DIR-T3-057（web pnpm workspace 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003
- FR-\* / DX-GP-\* / DX-CICD-\* / NFR-G-PRV-\*
