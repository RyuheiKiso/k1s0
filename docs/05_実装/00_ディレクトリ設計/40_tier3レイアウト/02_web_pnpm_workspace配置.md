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

## 依存方向

```
apps/  →  packages/  →  @k1s0/sdk（src/sdk/typescript/）
         ↓
       tools/（devDependency）
```

apps 間の相互依存は禁止。共通ロジックは packages/ に移動する。packages 間の依存は許容するが、循環は禁止。

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

portal / admin の Docker image は以下の構造。

```dockerfile
# apps/portal/Dockerfile
FROM node:20-alpine AS builder
WORKDIR /workspace
RUN npm install -g pnpm@9
COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY apps/portal/package.json apps/portal/
COPY packages/ packages/
RUN pnpm install --frozen-lockfile --filter '@k1s0/portal...'

COPY apps/portal/ apps/portal/
RUN pnpm --filter '@k1s0/portal' build

FROM node:20-alpine AS runtime
WORKDIR /app
COPY --from=builder /workspace/apps/portal/.next .next
COPY --from=builder /workspace/apps/portal/public public
COPY --from=builder /workspace/apps/portal/package.json .
COPY --from=builder /workspace/node_modules node_modules
USER node
EXPOSE 3000
CMD ["node", ".next/standalone/server.js"]
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
