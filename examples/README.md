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
| tier1-rust-service | 最小実装 (health stdout 1 行) | 完動 (tonic gRPC server / ZEN Engine 統合) |
| tier1-go-facade | 最小完動 (gRPC server + health protocol + reflection + graceful shutdown) | 完動 (proto handler 登録 / Dapr Go SDK adapter / OTel interceptor) |
| tier3-web-portal | 最小完動 (Vite + React + 1 page) | 完動 (TanStack Query / Apollo / i18n) |
| tier2-dotnet-service | 最小完動 (ASP.NET Core minimal API + JWT + tier1 SDK State.Save) | 完動 (Pact 契約テスト / マルチテナント拡張) |
| tier2-go-service | 最小完動 (HTTP + JWT + tier1 SDK State.Save) | 完動 (Pact 契約テスト / OutBox / Saga) |
| tier3-bff-graphql | 最小完動 (HTTP GraphQL + tier1 SDK State.Get) | 完動 (gqlgen / DataLoader / persisted queries) |
| tier3-native-maui | 最小完動 (MAUI App + ViewModel + tier1 SDK State 呼出) | 完動 (Windows / MacCatalyst 追加 / Native gRPC) |

リリース時点で「最小完動」と分類されるものは、`go run` / `cargo run` / `dotnet run` / `pnpm dev`
で起動でき、tier1 facade に対し最小 1 endpoint で疎通する。Pact / gqlgen / OTel interceptor /
multi-tenant 拡張等は採用初期で順次拡張される。

## 関連 ADR / 要件

- ADR-DEV-001（Paved Road）
- ADR-DEV-001（Golden Path 採用）
- DX-GP-* 系要件（Developer Experience / Golden Path）
