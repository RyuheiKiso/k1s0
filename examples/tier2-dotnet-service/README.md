# `examples/tier2-dotnet-service/` — tier2 .NET サービス完動例

tier2 ドメイン共通サービスの .NET 8 / Clean Architecture 構造を示す例。

## 目的

- `src/tier2/dotnet/services/{ApprovalFlow, InvoiceGenerator, TaxCalculator}` と同じ
  ソリューション構成（API / Application / Domain / Infrastructure 4 プロジェクト）
- `K1s0.Sdk` NuGet を経由して tier1 gRPC を呼ぶ典型例
- Pact 契約テスト（消費者駆動）の最小例

## scope

| 段階 | 提供範囲 |
|---|---|
| リリース時点 | 最小完動: `Program.cs` (ASP.NET Core minimal API + JWT 認証 + tier1 SDK State.Save) + `ExampleDotnetService.csproj` + `Dockerfile` + `catalog-info.yaml` + `appsettings.json` |
| 採用初期 | `Example.Payroll.sln` + 4 プロジェクト分離 (Api / Application / Domain / Infrastructure) + Pact 契約テスト |
| 採用後の運用拡大時 | マルチテナント拡張・OutBox パタン・Saga（Dapr Workflow） |

`dotnet run --project ExampleDotnetService.csproj` で起動できる。
`/healthz` `/readyz` (認証不要) と `/sample-write` (`Authorization: Bearer ...` 必須、
`T2_AUTH_MODE=off/hmac/jwks` で切替) を露出する。

## 想定構成（採用初期）

```text
tier2-dotnet-service/
├── README.md                          # 本ファイル
├── Example.Payroll.sln
├── src/
│   └── Example.Payroll/
│       ├── Example.Payroll.Api/         # gRPC / HTTP entry point
│       ├── Example.Payroll.Application/ # use case / orchestration
│       ├── Example.Payroll.Domain/      # entity / value object
│       └── Example.Payroll.Infrastructure/ # K1s0.Sdk wrapping / outbox
├── tests/
│   ├── Example.Payroll.UnitTests/
│   └── Example.Payroll.ContractTests/   # Pact consumer side
├── Dockerfile
└── catalog-info.yaml
```

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_services配置.md`
- ADR-MIG-001（.NET Framework サイドカー）
- ADR-DEV-001（Paved Road）

## 参照する tier1 API（採用初期想定）

- StateService（給与計算結果の永続化）
- AuditService（給与決裁の監査ログ）
- DecisionService（給与計算ルールの評価）
- WorkflowService（承認フロー駆動）
