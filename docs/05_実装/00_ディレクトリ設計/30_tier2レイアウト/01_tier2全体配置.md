# 01. tier2 全体配置

本ファイルは `src/tier2/` の全体構成を確定する。.NET と Go の 2 言語併存、テンプレートディレクトリの位置付け、導入タイミングを規定する。

## レイアウト

```
src/tier2/
├── README.md
├── dotnet/
│   ├── Tier2.sln                   # .NET ソリューション
│   ├── Directory.Build.props       # 共通プロパティ
│   ├── Directory.Packages.props    # central package management
│   └── services/
│       ├── ApprovalFlow/           # サービス 1 例
│       │   ├── src/
│       │   ├── tests/
│       │   ├── ApprovalFlow.sln    # 単独ビルド用
│       │   └── Dockerfile
│       ├── InvoiceGenerator/       # サービス 2 例
│       └── TaxCalculator/          # サービス 3 例
├── go/
│   ├── go.mod                      # module github.com/k1s0/k1s0/src/tier2/go
│   ├── go.sum
│   └── services/
│       ├── stock-reconciler/       # サービス 1 例
│       │   ├── cmd/
│       │   ├── internal/
│       │   ├── tests/
│       │   └── Dockerfile
│       └── notification-hub/       # サービス 2 例
└── templates/
    ├── dotnet-service/             # .NET 新規サービス雛形
    └── go-service/                 # Go 新規サービス雛形
```

## 2 言語併存の理由

tier2 は 採用側組織の既存 .NET 資産を段階的に取り込む受け皿であり、.NET が中心となる。ただし以下の場合は Go を選択する。

- 高スループットが要求される統合・変換サービス（例: 在庫同期バッチ、通知ハブ）
- tier1 Go ファサードと密結合する運用補助（例: Dapr Workflow をフル活用する業務フロー）

両言語が併存するが、同一サービスを両言語で二重実装することは禁止する。サービスごとに 1 言語を選択し、`src/tier2/dotnet/services/<service>/` または `src/tier2/go/services/<service>/` のどちらか一方に配置する。

## サービス単位の独立性

tier2 の各サービスは独立にビルド・デプロイ可能。

- .NET サービスは個別の `.csproj`（+ 単独 `.sln`）を持ち、`dotnet build` で個別ビルド可能
- Go サービスは `src/tier2/go/services/<service>/cmd/main.go` を持ち、`go build ./services/<service>/cmd/` で個別ビルド可能
- container image は各サービスごとに独立（`ghcr.io/k1s0/t2-approval-flow:<tag>` など）

## 導入タイミング

| 適用段階 | 追加内容 |
|---|---|
| リリース時点 | 構造のみ（`services/` 空ディレクトリ + README） |
| リリース時点 | `templates/dotnet-service/` と `templates/go-service/` を整備。雛形 CLI（src/platform/cli/）が参照する |
| リリース時点 | 採用側組織の業務サービス 2-3 個を実装（承認フロー・帳票生成・在庫同期など） |
| リリース時点 | 品質確保（unit / integration / contract テスト拡充） |
| 採用後の運用拡大時 | 追加サービスや外部連携の拡張 |

## 依存方向

- tier2 は `src/sdk/dotnet/` / `src/sdk/go/` を経由して tier1 にアクセス
- tier1 / contracts / tier3 / infra を直接参照することは禁止
- 他の tier2 サービスの内部 package を参照することは禁止（必要に応じて共通 lib を `src/tier2/go/shared/` または `src/tier2/dotnet/shared/` に置く。`shared/` の可視性強制方針は [03_go_services配置.md](03_go_services配置.md) の 3 層防御を参照）

## CODEOWNERS

```
/src/tier2/                                     @k1s0/tier2-dev
/src/tier2/dotnet/                              @k1s0/tier2-dev
/src/tier2/go/                                  @k1s0/tier2-dev
/src/tier2/templates/                           @k1s0/tier2-dev @k1s0/platform-team
```

## スパースチェックアウト cone

- `tier2-dev` cone : `src/contracts/` + `src/tier2/` + `src/sdk/dotnet/` + `src/sdk/go/` + `docs/`

## 対応 IMP-DIR ID

- IMP-DIR-T2-041（tier2 全体配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003（内部言語不可視）
- DS-SW-COMP-019（採用後の運用拡大時 再評価条件）
- FR-\*（業務ユースケース全般）/ DX-GP-\* / DX-CICD-\*
