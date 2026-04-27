# tier2-dotnet-service Helm chart

tier2 .NET service（ApprovalFlow / InvoiceGenerator / TaxCalculator 等）の汎用 Helm chart。
各 service は overlay 用 values.yaml で `service.name` / `image.repository` を上書きする。

## 利用例

```sh
helm install approval-flow deploy/charts/tier2-dotnet-service \
  -n tier2-services --create-namespace \
  --set service.name=approval-flow \
  --set image.repository=k1s0/k1s0/tier2-approval-flow \
  --set image.tag=v0.1.0
```

## tier2-go-service との差異

- TargetFramework: net8.0 / netstandard2.1（image base が `mcr.microsoft.com/dotnet/aspnet:8.0`）
- runAsUser / runAsGroup を 1654（aspnet runtime image の nonroot user）に
- env に `ASPNETCORE_URLS` / `DOTNET_ENVIRONMENT` を既定設定
- resources を Go 側より大きめ（.NET runtime メモリ要求のため）

## 関連設計

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_dotnet_services配置.md`
- ADR-TIER1-003
