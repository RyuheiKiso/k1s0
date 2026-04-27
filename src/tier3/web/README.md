# tier3 Web (pnpm workspace)

docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_web_pnpm_workspace配置.md の正典構成に準拠した React + TypeScript のモノレポ。

## レイアウト

```text
src/tier3/web/
├── README.md
├── package.json                # ルート（private、scripts のみ）
├── pnpm-workspace.yaml
├── tsconfig.base.json          # 共通 strict tsconfig
├── .npmrc
├── .eslintrc.cjs
├── .prettierrc
├── apps/
│   ├── portal/                 # 配信ポータル（Vite + React）
│   ├── admin/                  # 管理画面
│   └── docs-site/              # ドキュメントサイト
├── packages/
│   ├── ui/                     # shadcn/ui 派生コンポーネント
│   ├── api-client/             # k1s0 BFF / SDK ラッパー
│   ├── i18n/                   # 国際化基盤（軽量、i18next 依存なし）
│   └── config/                 # 環境変数 / 設定読込
└── tools/
    └── eslint-config/          # 共通 ESLint 設定
```

## ビルド

```bash
# ルートで全 package / app を一括処理。
pnpm install
pnpm build
pnpm test
pnpm lint
pnpm typecheck

# 特定 package のみ。
pnpm --filter @k1s0/ui build
pnpm --filter @k1s0/portal dev   # Vite dev server
```

## 依存方向

- apps/* → packages/{api-client,ui,i18n,config}
- apps/* → tools/eslint-config（dev only）
- リリース時点 では apps から `@k1s0/sdk`（src/sdk/typescript）を直接 import せず、
  packages/api-client 経由で BFF を呼ぶ
- 運用蓄積後 で gRPC-Web 直接利用も選択肢に追加（02_web_pnpm_workspace配置.md 参照）

## 関連 ID

- IMP-DIR-T3-056 / IMP-DIR-T3-057
- ADR-TIER1-003
