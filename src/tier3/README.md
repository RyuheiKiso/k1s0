# src/tier3 — UI / BFF / Native / Legacy Wrap

エンドユーザに最も近い層。Web SPA・Native アプリ・BFF（Backend for Frontend）・
.NET Framework モノリスのサイドカー方式相乗りを束ねる。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/`](../../docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/)。

## 配置

```text
tier3/
├── web/                                   # React + Vite + pnpm（pnpm workspace）
│   ├── apps/
│   │   ├── portal/                        # 利用者ポータル
│   │   ├── admin/                         # 管理者画面
│   │   └── docs-site/                     # 採用ドキュメントサイト
│   ├── packages/                          # 共通ライブラリ
│   │   ├── ui/                            # 共通 React コンポーネント（Button / Card / Spinner ...）
│   │   ├── api-client/                    # tier1 SDK（TypeScript）への薄ラッパ
│   │   ├── i18n/                          # 多言語対応（ja / en の最低 2 ロケール）
│   │   └── config/                        # 環境別設定（VITE_BFF_URL 等）
│   └── tools/eslint-config/               # 共通 ESLint config
├── bff/                                   # Go BFF（GraphQL）
│   ├── go.mod
│   ├── cmd/{portal-bff,admin-bff}/main.go
│   └── internal/{auth,config,graphql,k1s0client,rest,shared}/
├── native/                                # .NET MAUI（Android / iOS / macOS / Windows）
│   ├── Native.sln
│   ├── apps/
│   │   ├── K1s0.Native.Hub/               # 一般利用者向け
│   │   └── K1s0.Native.Admin/             # 管理者向け
│   └── shared/K1s0.Native.Shared/         # 共通ロジック（Converter / Extension）
├── legacy-wrap/                           # .NET Framework モノリスのサイドカー方式
│   ├── LegacyWrap.sln
│   ├── sidecars/K1s0.Legacy.Sidecar/      # ASP.NET Web API サイドカー
│   └── migration-guide/                   # 段階的移行ガイド
└── templates/                             # k1s0-scaffold 用テンプレ
    ├── bff/
    └── web/
```

## アーキテクチャ要点

- **BFF パターン**: web / native は BFF（`portal-bff` / `admin-bff`）経由で tier1 を叩く。
  認証（Keycloak OIDC）と aggregation を BFF に集約し、フロントは画面遷移と表示に集中する。
- **k1s0.io annotation prefix**: catalog-info.yaml で `k1s0.io/component` 等の独自 annotation を使用。
  Backstage プラグイン `k1s0-catalog` がこれをサーチ・表示する。
- **OIDC bearer の中継**: BFF は incoming `Authorization: Bearer <token>` を tier1 へ伝搬する
  pass-through middleware を提供（`internal/auth/middleware.go`）。

## ローカル起動

```sh
# Web（pnpm 必須）
cd src/tier3/web
pnpm install
pnpm --filter portal dev          # http://localhost:5173

# BFF（Go）
cd src/tier3/bff
go run ./cmd/portal-bff           # http://localhost:8080

# Native（dotnet 8.0 + MAUI workload）
cd src/tier3/native
dotnet build apps/K1s0.Native.Hub
```

## 関連設計

- [docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/](../../docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/) — DS-SW-COMP-020 系
- [docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/](../../docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/)
- [ADR-MIG-001](../../docs/02_構想設計/adr/ADR-MIG-001-dotnet-sidecar.md) — .NET Framework サイドカー
