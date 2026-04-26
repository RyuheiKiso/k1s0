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
├── tier3-web-portal/               # React (Vite) 最小 portal
│   ├── README.md
│   ├── package.json
│   ├── vite.config.ts
│   ├── index.html
│   ├── src/
│   │   ├── main.tsx
│   │   ├── routes/
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

## example の導入段階

| 適用段階 | 対象 example |
|---|---|
| リリース時点 | README.md のみ（構造） |
| リリース時点 | tier1-rust-service、tier1-go-facade、tier3-web-portal |
| リリース時点 | tier2-dotnet-service、tier3-bff-graphql |
| リリース時点 | tier2-go-service、tier3-native-maui |
| 採用後の運用拡大時 | マルチテナント対応版 example |

## 3 系統の使い分け: scaffold / templates / examples

k1s0 にはコード雛形・参照コードに相当するディレクトリが 3 つ存在し、役割が重ならない。混乱を避けるため責務を明示する。

| 観点 | scaffold<br>（`tools/codegen/scaffold/`） | templates<br>（`src/tier2/templates/`） | examples<br>（`examples/`） |
|---|---|---|---|
| 形式 | Handlebars テンプレート（`.hbs`） | コンパイル可能な Rust / Go / .NET プロジェクト | 実稼働する完成プロジェクト |
| 目的 | 新サービス生成時の物理テンプレ源 | scaffold から参照される「プレースホルダ値の元ネタ」（ADR で確定した構造を型付きで保持） | 学習教材・動作保証 |
| 呼び出し元 | `k1s0-scaffold` CLI が `.hbs` をレンダリング | `k1s0-scaffold` CLI が引数 `--reference-template` で構造検証時に参照 | 開発者が手動で閲覧・起動、CI が週次 E2E |
| 内容 | プレースホルダ（`{{service-name}}` 等）を含む原石 | 最小限のエンティティ 1 件程度のみ持つ型付きプロジェクト | 実際の業務を満たす完動コード |
| プレースホルダ | あり | なし | なし |
| 更新頻度 | コーディング規約変更時（四半期） | tier2 Architecture 変更時（半期） | tier1 API 変更時（月次） |
| CI 検証 | 生成 golden test（`tests/golden/`） | `cargo check` / `go build` / `dotnet build` のみ | E2E 動作検証（週次） |
| 参照元 | `scaffold --type tier2-go-service --name foo` | scaffold 実装が構造比較のため読む | 開発者が import / study の対象 |

流れを一文でまとめると、開発者が `k1s0-scaffold` を呼ぶと、scaffold CLI は `tools/codegen/scaffold/handlebars/*.hbs` を展開して新プロジェクトの骨格を作り、その内部構造の妥当性検証のために `src/tier2/templates/` の参照プロジェクトと diff を取る。完動例が欲しくなったら `examples/` を読む、という 3 段構えになる。

## 対応 IMP-DIR ID

- IMP-DIR-COMM-113（examples 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DEVEX-004（Golden Path 採用）
- DX-GP-\*
