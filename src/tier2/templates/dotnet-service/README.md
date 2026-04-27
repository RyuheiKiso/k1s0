# tier2-dotnet-service テンプレート

ASP.NET Core minimal API + K1s0.Sdk.Grpc を使う tier2 .NET サービス雛形。`k1s0-scaffold` / Backstage UI の両経路から呼び出し可能。

## 利用方法

```bash
k1s0-scaffold new tier2-dotnet-service \
  --name approval-flow \
  --namespace K1s0.Payment.ApprovalFlow \
  --owner @k1s0/payment \
  --system k1s0
```

## 生成内容

- `{{name}}/{{namespace}}.csproj` — `Microsoft.NET.Sdk.Web` + K1s0.Sdk.Grpc 参照
- `{{name}}/Program.cs` — minimal API + State.SaveAsync サンプル
- `{{name}}/appsettings.json` — k1s0 接続情報の defaults
- `{{name}}/Dockerfile` — `dotnet/aspnet:8.0` runtime
- `{{name}}/catalog-info.yaml`
- `{{name}}/README.md`

## テンプレート変数

| 変数 | 用途 | 必須 |
|---|---|---|
| `name` | サービス名（kebab-case） | ✅ |
| `namespace` | .NET ルート名前空間（PascalCase） | ✅ |
| `owner` | 所有チーム | ✅ |
| `system` | サブシステム | ✅ (default `k1s0`) |
| `description` | 概要説明 | (default あり) |
| `tier` / `language` | catalog 自動付与（`tier2` / `dotnet`） | 固定 |

## 関連

- [`examples/tier2-dotnet-service/`](../../../../examples/tier2-dotnet-service/)
- [`docs/05_実装/10_ビルド設計/40_dotnet_sln境界/01_dotnet_sln境界.md`](../../../../docs/05_実装/10_ビルド設計/40_dotnet_sln境界/01_dotnet_sln境界.md)
