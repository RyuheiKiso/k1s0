# `examples/` — Golden Path 完動例

本ディレクトリは k1s0 の **Golden Path（推奨実装パターン）** の実稼働コード例を集約する。

設計正典: [`docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md`](../docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md)
（IMP-DIR-COMM-113）

## 役割

`tools/codegen/scaffold/`（雛形 CLI のテンプレート）と `src/tier2/templates/`（型付きテンプレート）と
本 `examples/` の 3 系統は責務が異なる:

| 観点 | scaffold | templates | examples（**本ディレクトリ**） |
|---|---|---|---|
| 形式 | Handlebars `.hbs` | コンパイル可能プロジェクト | 実稼働する完動プロジェクト |
| 目的 | 新サービス生成時の雛形源 | scaffold の構造検証参照 | 学習教材 / 動作保証 / E2E 契約 |
| プレースホルダ | あり | なし | なし |
| CI 検証 | golden test | コンパイルのみ | 週次 E2E |

詳細責務分離は `docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md` 参照。

## 配置

```text
examples/
├── README.md                       # 本ファイル
├── tier1-rust-service/             # Rust 自作領域（ZEN / crypto / 雛形 CLI 系）の最小例
├── tier1-go-facade/                # Go Dapr ファサードの最小例
├── tier2-dotnet-service/           # tier2 .NET サービス完動例
├── tier2-go-service/               # tier2 Go サービス完動例
├── tier3-web-portal/               # React (Vite) 最小 portal
├── tier3-bff-graphql/              # portal-bff GraphQL 最小例
└── tier3-native-maui/              # .NET MAUI 最小アプリ
```

## 各 example の構成

全 example に共通して以下を備える（または将来備える）:

- `README.md` — 何を達成するか / 起動方法 / 参照する tier1 API
- `Dockerfile` — image build 可能
- `catalog-info.yaml` — Backstage で自動カタログ化
- `.github/workflows/example-<name>.yml` — 週次 E2E（リポジトリルート側で集約）

## マチュリティ（[`docs/SHIP_STATUS.md`](../docs/SHIP_STATUS.md) と整合）

| example | リリース時点 | 採用初期 |
|---|---|---|
| tier1-rust-service | 最小実装 | 完動 |
| tier1-go-facade | 最小実装 | 完動 |
| tier3-web-portal | 最小実装 | 完動 |
| tier2-dotnet-service | README のみ | 完動 |
| tier2-go-service | README のみ | 完動 |
| tier3-bff-graphql | README のみ | 完動 |
| tier3-native-maui | README のみ | 完動 |

リリース時点で「README のみ」のものは、採用組織の POC で完動が必要になった時点で
組織側が完成させる前提（Golden Path として参照可能な構造のみを提供する）。

## 関連 ADR / 要件

- ADR-DEV-001（Paved Road）
- ADR-DEVEX-004（Golden Path 採用）
- DX-GP-* 系要件（Developer Experience / Golden Path）
