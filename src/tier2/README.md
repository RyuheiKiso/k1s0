# src/tier2 — ドメイン共通業務ロジック（C# / Go）

tier1 公開 API（`k1s0.State.Save` / `k1s0.PubSub.Publish` 等）の上に、複数のドメインで共有される
業務ロジック層を構築する。tier1 の SDK のみを通して内部実装に依存し、tier3 から見ても
**Dapr / Rust / Postgres / Kafka などのバックエンドは不可視**。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/`](../../docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/)。

## 配置

```text
tier2/
├── dotnet/services/                       # C# / .NET Aspire ベースのドメインサービス
│   ├── ApprovalFlow/                      # 承認フロー
│   ├── InvoiceGenerator/                  # 請求書生成
│   └── TaxCalculator/                     # 税額計算
├── go/services/                           # Go ベースのドメインサービス
│   ├── notification-hub/                  # 通知配信ハブ
│   └── stock-reconciler/                  # 在庫整合
└── templates/                             # k1s0-scaffold が参照する Backstage Software Template v1beta3
    ├── go-service/
    └── dotnet-service/
```

## 各サービスの構造（DDD レイヤード）

```text
<service>/
├── <Service>.sln                           # ソリューション
├── Dockerfile                              # distroless multi-stage
├── catalog-info.yaml                       # Backstage カタログ
├── README.md
├── src/
│   ├── <Service>.Domain/                   # ドメイン層（Entities / ValueObjects / Events / Interfaces）
│   ├── <Service>.Application/              # アプリケーション層（UseCases）
│   ├── <Service>.Infrastructure/           # インフラ層（Persistence、外部 API 呼び出し）
│   └── <Service>.Api/                      # ASP.NET Core minimal API（または Go cmd）
└── tests/
    ├── <Service>.Domain.Tests/             # xUnit ドメイン単体
    ├── <Service>.Application.Tests/        # ユースケース単体
    └── <Service>.ArchitectureTests/        # NetArchTest によるレイヤ違反検出
```

Go 側は `cmd/<service>/main.go` + `internal/{api,config}/` 構成（より軽量）。

## 言語選択指針（DS-SW-COMP-016）

| 観点 | C# / .NET 採用 | Go 採用 |
|---|---|---|
| 既存 .NET 資産再利用 | ◎ | × |
| 並行処理重視（goroutine 1 万超） | × | ◎ |
| 業務ロジックの重さ | ◎（DDD と相性良好） | △（純粋業務は省略形になりがち） |
| バイナリサイズ最適化 | △ | ◎（distroless 20MB） |

## tier1 SDK の使い方

```csharp
// tier2 / tier3（C#）— Dapr は見えない
await k1s0.State.SaveAsync("orders", orderId, order);
await k1s0.PubSub.PublishAsync("order-events", "created", order);
var decision = await k1s0.Decision.EvaluateAsync("approval/purchase", new { amount, grade });
```

```go
// Go — 同じ動詞、同じ形
k1s0.State.Save(ctx, "orders", orderId, order)
k1s0.PubSub.Publish(ctx, "order-events", "created", order)
```

## 関連設計

- [ADR-TIER1-003](../../docs/02_構想設計/adr/ADR-TIER1-003-language-invisibility.md) — 内部言語不可視化
- [docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/](../../docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/) — DS-SW-COMP-016 系
- [docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/](../../docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/)
