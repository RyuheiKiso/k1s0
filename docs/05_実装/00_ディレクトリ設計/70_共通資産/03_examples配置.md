# 03. examples 配置

本ファイルは `examples/` 配下の配置を確定する。Golden Path（推奨実装）の実稼働版として、新メンバーが実装を真似できる最小動作例を集約する。

## examples/ の役割

scaffold CLI で生成される雛形とは別に、「実際に動作して SLI を満たす」完成済みコード例を置く場所。Airbnb・Shopify の Golden Path 事例に倣う。

目的:

- 新人オンボーディング時の学習教材
- PR レビュー時の「この形が正解」の共通理解
- 雛形 CLI で生成直後に満たすべき最小動作の担保
- CI で週次 E2E 実行し、「example が動くこと」を Golden Path の契約とする

## レイアウト

```
examples/
├── README.md
├── tier1-rust-service/             # Rust 自作領域の最小サービス例
│   ├── README.md
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── service.rs
│   │   └── lib.rs
│   ├── tests/
│   └── Dockerfile
├── tier1-go-facade/                # Go Dapr facade の最小例
│   ├── README.md
│   ├── go.mod
│   ├── cmd/
│   │   └── example-facade/
│   │       └── main.go
│   ├── internal/
│   └── Dockerfile
├── tier2-dotnet-service/           # tier2 .NET サービス完成例
│   ├── README.md
│   ├── ExamplePayrollService.sln
│   ├── src/
│   │   └── Example.Payroll/
│   │       ├── Example.Payroll.Api/
│   │       ├── Example.Payroll.Application/
│   │       ├── Example.Payroll.Domain/
│   │       └── Example.Payroll.Infrastructure/
│   └── tests/
├── tier2-go-service/
│   ├── README.md
│   ├── go.mod
│   ├── cmd/
│   └── internal/
├── tier3-web-portal/               # Next.js 最小 portal
│   ├── README.md
│   ├── package.json
│   ├── src/
│   │   ├── app/
│   │   └── components/
│   └── Dockerfile
├── tier3-bff-graphql/              # portal-bff 最小例
│   ├── README.md
│   ├── go.mod
│   ├── cmd/
│   └── internal/
└── tier3-native-maui/              # MAUI 最小アプリ
    ├── README.md
    ├── Example.Native.sln
    └── apps/Example.Native.Hub/
```

## 各 example の構成

全 example に共通して:

- **README.md**: 何を達成するか、起動方法、参照している tier1 API
- **Docker image**: `Dockerfile` でビルド可能
- **catalog-info.yaml**: Backstage で自動カタログ化
- **CI workflow**: `.github/workflows/example-<name>.yml` で週次実行

## example の導入フェーズ

| Phase | 対象 example |
|---|---|
| Phase 0 | README.md のみ（構造） |
| Phase 1a | tier1-rust-service、tier1-go-facade、tier3-web-portal |
| Phase 1b | tier2-dotnet-service、tier3-bff-graphql |
| Phase 1c | tier2-go-service、tier3-native-maui |
| Phase 2 | マルチテナント対応版 example |

## example と scaffold の差分

| 観点 | scaffold（tools/codegen/scaffold/） | example（examples/） |
|---|---|---|
| 目的 | 新サービス作成の起点 | 学習教材・動作保証 |
| 内容 | 空のテンプレート（プレースホルダ） | 実装済みの完動例 |
| 更新頻度 | コーディング規約変更時 | tier1 API 変更時 |
| CI 検証 | 生成テスト（golden test） | E2E 動作検証（週次） |

## 対応 IMP-DIR ID

- IMP-DIR-COMM-113（examples 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DEVEX-004（Golden Path 採用）
- DX-GP-\*
