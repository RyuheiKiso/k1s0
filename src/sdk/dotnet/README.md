# K1s0 .NET SDK

tier1 公開 12 API（k1s0.tier1.\<api>.v1）の .NET クライアント SDK。

## パッケージ

| NuGet | 説明 |
|---|---|
| [`K1s0.Sdk.Grpc`](https://www.nuget.org/packages/K1s0.Sdk.Grpc) | 高水準 facade（推奨）。利用者は本パッケージのみ参照すれば動作する。 |
| [`K1s0.Sdk.Proto`](https://www.nuget.org/packages/K1s0.Sdk.Proto) | Protobuf / gRPC 生成 stub（K1s0.Sdk.Grpc が内部依存）。直接参照は非推奨。 |

## 対応 .NET

- `netstandard2.1` — .NET Framework 4.6.2+ / Mono / Xamarin
- `net8.0` — .NET 8 LTS

## 使い方（リリース時点 最小、動詞統一 facade はロードマップ #8 で追加）

```csharp
using K1s0.Sdk.Generated.K1s0.Tier1.State.V1;
using Grpc.Net.Client;

using var channel = GrpcChannel.ForAddress("https://tier1.k1s0.example.com");
var client = new StateService.StateServiceClient(channel);
var resp = await client.GetAsync(new GetRequest {
    Store = "valkey-default",
    Key = "user/123",
});
```

## 詳細

- ソース: [github.com/k1s0/k1s0](https://github.com/k1s0/k1s0)
- 設計正典: `docs/05_実装/10_ビルド設計/40_dotnet_sln境界/`
- ライセンス: Apache-2.0
