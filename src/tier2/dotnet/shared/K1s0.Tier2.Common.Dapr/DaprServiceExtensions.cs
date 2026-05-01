// 本ファイルは tier2 .NET 共通の Dapr Client DI 登録拡張。
//
// docs 正典:
//   src/tier2/go/shared/dapr/client.go (Go 側 NewClient と同形の役割)
//
// 設計動機:
//   各 Api / Infrastructure 層が `services.AddK1s0DaprClient()` 1 行で
//   Dapr.Client.DaprClient を Singleton 登録できる。Dapr sidecar が
//   自動 inject する DAPR_HTTP_PORT / DAPR_GRPC_PORT 環境変数を読み、
//   `DaprClientBuilder` で組み立てる。
//
//   Domain / Application 層は本ラッパー (または独自 IRepository) 経由で
//   tier1 公開 API を呼ぶ。SDK 型を Domain に漏らさないために
//   tier1 公開 API ごとの薄いラッパーは Infrastructure 層で書く。

using Dapr.Client;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Tier2.Common.Dapr;

/// <summary>tier2 .NET 共通の Dapr Client セットアップ拡張。</summary>
public static class DaprServiceExtensions
{
    /// <summary>
    /// `services.AddK1s0DaprClient()` で Dapr.Client.DaprClient を Singleton 登録する。
    /// Dapr sidecar が inject する環境変数 DAPR_GRPC_PORT (既定 50001) を読む。
    /// </summary>
    public static IServiceCollection AddK1s0DaprClient(this IServiceCollection services)
    {
        services.AddSingleton(_ =>
        {
            var builder = new DaprClientBuilder();
            // Dapr sidecar inject 値の優先順位:
            //   DAPR_GRPC_ENDPOINT > DAPR_GRPC_PORT (localhost:port) > Dapr SDK 既定。
            var explicitEndpoint = Environment.GetEnvironmentVariable("DAPR_GRPC_ENDPOINT");
            if (!string.IsNullOrEmpty(explicitEndpoint))
            {
                builder.UseGrpcEndpoint(explicitEndpoint);
            }
            else
            {
                var port = Environment.GetEnvironmentVariable("DAPR_GRPC_PORT");
                if (!string.IsNullOrEmpty(port) && int.TryParse(port, out var p) && p > 0)
                {
                    builder.UseGrpcEndpoint($"http://127.0.0.1:{p}");
                }
            }
            return builder.Build();
        });
        return services;
    }
}
