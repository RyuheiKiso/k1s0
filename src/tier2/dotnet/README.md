# tier2 .NET ソリューション

本ディレクトリは k1s0 tier2 層の .NET サービス群を格納する。`Tier2.sln` 1 本で IDE / 横断ツールから全サービスを開き、各サービス配下の単独 `.sln` で CI の path-filter を生かす二段構成。

## docs 正典

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_solution配置.md`
- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md`
- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/06_依存管理.md`

## レイアウト

```text
src/tier2/dotnet/
├── README.md
├── Tier2.sln                       # 全サービス + shared を含む統合 .sln
├── Directory.Build.props           # 共通プロパティ（TFM net8.0 / Nullable / TWAE）
├── Directory.Packages.props        # Central Package Management
├── nuget.config                    # nuget.org のみ（採用初期 で社内 source 追加）
├── .editorconfig
├── services/
│   ├── ApprovalFlow/               # 承認フロー（Onion 4 層 + 3 種テスト）
│   ├── InvoiceGenerator/           # 帳票生成
│   └── TaxCalculator/              # 税計算
└── shared/
    ├── Tier2.Shared.Dapr/          # Dapr / k1s0 SDK 共通ラッパー
    └── Tier2.Shared.Otel/          # OpenTelemetry 初期化共通
```

## サービス構成（共通: Onion Architecture 4 層）

各サービスは独立にビルド・デプロイ可能。

| 層 | csproj | 依存先 |
|---|---|---|
| Api | `<Service>.Api.csproj` | Application のみ |
| Application | `<Service>.Application.csproj` | Domain のみ |
| Domain | `<Service>.Domain.csproj` | 依存なし |
| Infrastructure | `<Service>.Infrastructure.csproj` | Domain のみ |

依存方向の強制は 2 段構成: MSBuild の `<EnforceProjectReferenceValidation>` + tests/`<Service>.ArchitectureTests/` の NetArchTest。詳細は [04_サービス単位の内部構造.md](../../../docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md) 参照。

## ビルドとテスト

```bash
# 全 csproj 解決を更新する。
dotnet restore Tier2.sln

# 統合ビルド。
dotnet build Tier2.sln -c Release

# 個別サービスのみビルド（CI path-filter 用）。
dotnet build services/ApprovalFlow/ApprovalFlow.sln -c Release

# 全テスト。
dotnet test Tier2.sln

# Architecture テストのみ実行。
dotnet test Tier2.sln --filter "Category=Architecture"
```

## Dockerfile

各サービスの Dockerfile は `src/tier2/dotnet/` を build context のルートとして書いている。CI / 開発者は次のコマンドで build する。

```bash
docker build -f services/ApprovalFlow/Dockerfile -t ghcr.io/k1s0/t2-approval-flow:dev .
```

リポジトリルートから build すると `Directory.Build.props` / `Directory.Packages.props` の解決が壊れる。

## CODEOWNERS

`src/tier2/dotnet/` 全体は `@k1s0/tier2-dev` の所有（CONTRIBUTING.md の CODEOWNERS と整合）。

## 関連 ID

- IMP-DIR-T2-041 / IMP-DIR-T2-042 / IMP-DIR-T2-044 / IMP-DIR-T2-046
- ADR-TIER1-003（内部言語不可視）
- DS-SW-COMP-019
