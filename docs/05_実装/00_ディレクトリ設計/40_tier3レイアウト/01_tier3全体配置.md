# 01. tier3 全体配置

本ファイルは `src/tier3/` の全体構成を確定する。4 カテゴリ（web / native / bff / legacy-wrap）の配置と、それぞれの技術スタック、導入タイミングを規定する。

## レイアウト

```
src/tier3/
├── README.md
├── web/                    # React + TypeScript + pnpm workspace
│   ├── pnpm-workspace.yaml
│   ├── package.json
│   ├── tsconfig.base.json
│   ├── apps/
│   │   ├── portal/         # 配信ポータル
│   │   ├── admin/          # 管理画面
│   │   └── docs-site/      # 運用蓄積後
│   ├── packages/           # 共有ライブラリ
│   │   ├── ui/             # shadcn/ui 派生の共通コンポーネント
│   │   ├── api-client/     # k1s0 SDK wrapper
│   │   ├── i18n/           # 国際化基盤
│   │   └── config/         # 共通設定
│   └── tools/
│       └── eslint-config/
├── native/                 # .NET MAUI
│   ├── Native.sln
│   ├── Directory.Build.props
│   ├── apps/
│   │   ├── K1s0.Native.Hub/
│   │   └── K1s0.Native.Admin/
│   └── shared/
│       └── K1s0.Native.Shared/
├── bff/                    # Backend For Frontend
│   ├── go.mod              # module github.com/k1s0/k1s0/src/tier3/bff
│   ├── cmd/
│   │   ├── portal-bff/
│   │   └── admin-bff/
│   ├── internal/
│   │   ├── graphql/
│   │   ├── rest/
│   │   └── k1s0client/
│   ├── Dockerfile.portal       # portal Web 向け BFF
│   └── Dockerfile.admin        # admin Web 向け BFF
└── legacy-wrap/            # .NET Framework sidecar wrapper
    ├── LegacyWrap.sln
    └── sidecars/
        └── K1s0.Legacy.Sidecar/
```

## 4 カテゴリの技術スタック

### web（Web フロントエンド）

- Language: TypeScript 5.x
- Framework: Next.js（App Router）または Vite + React 18+
- Package manager: pnpm 9.x
- UI library: shadcn/ui + Tailwind CSS
- State management: Zustand / TanStack Query
- gRPC-Web: `@connectrpc/connect-web`
- 認証: `@k1s0/sdk-auth`（Keycloak OIDC 連携）

### native（.NET MAUI モバイル / デスクトップ）

- Language: C# 12 / .NET 8
- Framework: .NET MAUI
- Platforms: iOS / Android / Windows / macOS（リリース時点）
- UI: XAML + MAUI Controls
- 認証: `K1s0.Sdk.Auth`（Keycloak OIDC 連携）

### bff（Backend For Frontend）

- Language: Go 1.22+
- Framework: `chi` + `graphql-go` または `connectrpc/connect-go`
- 役割: Web / Native からの複合クエリを tier1 / tier2 への複数呼び出しに変換し、client 固有のレスポンス形式に加工
- 配置は SDK の使い勝手を高めるため Go を選定（tier3 Web が直接 tier1 を叩くのでも動くが、tier1 呼び出しのオーケストレーションと認可フィルタを 1 箇所に集約する目的）

### legacy-wrap（.NET Framework ラッパー）

- Language: C# 7+（.NET Framework 4.8）
- Framework: ASP.NET Web API + Dapr sidecar
- 役割: 採用側組織の既存 .NET Framework 資産を Dapr sidecar パターンで現行 k1s0 基盤に接続する薄いラッパー
- ADR-MIG-001 に従い、段階的に .NET 8 / .NET MAUI への移行を進める

## 導入タイミング

| 適用段階 | web | native | bff | legacy-wrap |
|---|---|---|---|---|
| リリース時点 | 構造のみ | 構造のみ | 構造のみ | 構造のみ |
| リリース時点 | portal（最小配信ポータル） | - | - | - |
| リリース時点 | portal 本格運用 / admin 最小 | K1s0.Native.Hub 最小 | portal-bff 最小 | sidecar 雛形 |
| リリース時点 | 本番品質 / i18n / a11y | 全プラットフォーム対応 | admin-bff | 移行ガイド整備 |
| 採用後の運用拡大時 | マルチテナント UI | - | 高度な aggregator | - |

## 依存方向

tier3 の 4 サブカテゴリは、SDK への依存先が言語ごとに、BFF との関係が用途ごとに異なるため、粗い「tier3 → SDK」ではなく subtier 粒度で規定する。

### web（TypeScript / Next.js・Vite）

- 許可: `src/sdk/typescript/` 経由で BFF の REST / GraphQL / gRPC-Web エンドポイントを呼ぶ
- 禁止: `src/sdk/go/` / `src/sdk/dotnet/` の直接参照、`src/tier1/` / `src/tier2/` / `src/contracts/` の import、`src/tier3/bff/` の Go コードへの直接参照
- リリース時点 は BFF 経由のみ、運用蓄積後で直 gRPC-Web も許容（[02_web_pnpm_workspace配置.md](02_web_pnpm_workspace配置.md) 参照）

### native（.NET MAUI）

- 許可: `src/sdk/dotnet/`（`K1s0.Sdk` / `K1s0.Sdk.Auth`）経由で BFF または tier1 公開 API を呼ぶ
- 禁止: BFF の Go コード直接参照、`src/sdk/go/` / `src/sdk/typescript/` の参照、`src/tier1/` / `src/tier2/` / `src/contracts/` の直接参照

### bff（Go）

- 許可: `src/sdk/go/` 経由で tier1 / tier2 にアクセス
- 禁止: tier1 / tier2 の internal package 直接参照、`src/contracts/` の直接 import（SDK が契約を隠蔽するため）
- リリース時点 では `replace` directive で `src/sdk/go/` を local 参照可（運用蓄積後で module publish に切替、[../30_tier2レイアウト/03_go_services配置.md](../30_tier2レイアウト/03_go_services配置.md) と同方針）

### legacy-wrap（.NET Framework sidecar）

- 許可: `src/sdk/dotnet/` 経由で tier1 公開 API を呼ぶ
- 禁止: 他 tier3 subtier との相互参照、`third_party/` 以外への .NET Framework ライブラリ直接配置（[05_レガシーラップ配置.md](05_レガシーラップ配置.md) 参照）

### subtier 間の関係

`web → bff` の関係は HTTP / GraphQL / gRPC-Web の network 呼び出しに限定する（BFF の Go パッケージ直接 import は依存方向違反）。`native → bff` も同様。`native ↔ legacy-wrap` の直接関係は持たず、両者は独立に tier1 公開 API を呼ぶ。

## CODEOWNERS

```
/src/tier3/                                     @k1s0/tier3-web @k1s0/tier3-native
/src/tier3/web/                                 @k1s0/tier3-web
/src/tier3/native/                              @k1s0/tier3-native
/src/tier3/bff/                                 @k1s0/tier3-web @k1s0/tier2-dev
/src/tier3/legacy-wrap/                         @k1s0/tier3-native @k1s0/platform-team
```

## スパースチェックアウト cone

- `tier3-web-dev` cone : `src/sdk/typescript/` + `src/tier3/web/` + `src/tier3/bff/` + `docs/`
- `tier3-native-dev` cone : `src/sdk/dotnet/` + `src/tier3/native/` + `src/tier3/legacy-wrap/` + `docs/`

BFF は Web / Native 両方の開発者が編集する可能性があるが、BFF 本体の実装は主に Web 側（Go に慣れた tier3-web チームが担当）。Native は BFF が提供する HTTP / gRPC を呼ぶだけ。

## 対応 IMP-DIR ID

- IMP-DIR-T3-056（tier3 全体配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003（内部言語不可視）
- ADR-MIG-001（.NET Framework sidecar）
- FR-\* / DX-GP-\* / DX-CICD-\*
