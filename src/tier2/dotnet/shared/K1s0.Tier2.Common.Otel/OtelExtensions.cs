// 本ファイルは tier2 .NET 共通の OpenTelemetry 初期化拡張。
//
// docs 正典:
//   src/tier2/go/shared/otel/init.go (Go 側 Init と同形の挙動を提供)
//   docs/05_実装/60_観測性設計/
//
// 設計動機:
//   tier2 .NET 全 Api 層が `services.AddK1s0Otel(name, version, env)` 1 行で
//   OTel TracerProvider / MeterProvider を初期化できる。
//   OTEL_EXPORTER_OTLP_ENDPOINT 環境変数で OTLP gRPC を有効化し、未設定時は
//   exporter を登録しないため SDK の no-op 経路に乗る (dev / unit test 向け)。

using Microsoft.Extensions.DependencyInjection;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;

namespace K1s0.Tier2.Common.Otel;

/// <summary>
/// OTel 初期化オプション (リリース時点 minimum)。
/// </summary>
public sealed record OtelOptions(
    /// <summary>resource attribute service.name。</summary>
    string ServiceName,
    /// <summary>resource attribute service.version。</summary>
    string ServiceVersion,
    /// <summary>resource attribute deployment.environment.name (dev / staging / prod)。</summary>
    string Environment);

/// <summary>tier2 .NET 共通の OTel セットアップ拡張。</summary>
public static class OtelExtensions
{
    /// <summary>
    /// `services.AddK1s0Otel("invoice-generator", "0.1.0", "dev")` で OTel を有効化する。
    /// OTEL_EXPORTER_OTLP_ENDPOINT 環境変数があれば OTLP gRPC エクスポータを登録する。
    /// 未設定時は SDK は登録するが exporter は無いため no-op 動作 (dev 向け)。
    /// </summary>
    public static IServiceCollection AddK1s0Otel(
        this IServiceCollection services,
        string serviceName,
        string serviceVersion = "0.0.0",
        string environment = "dev")
    {
        var options = new OtelOptions(serviceName, serviceVersion, environment);
        var endpoint = Environment_GetVariable("OTEL_EXPORTER_OTLP_ENDPOINT");
        services.AddSingleton(options);
        services.AddOpenTelemetry()
            .ConfigureResource(rb => rb
                .AddService(serviceName: options.ServiceName, serviceVersion: options.ServiceVersion)
                .AddAttributes(new[] { new KeyValuePair<string, object>("deployment.environment.name", options.Environment) }))
            .WithTracing(t =>
            {
                t.AddAspNetCoreInstrumentation();
                if (!string.IsNullOrEmpty(endpoint))
                {
                    t.AddOtlpExporter(o => o.Endpoint = new Uri(endpoint));
                }
            })
            .WithMetrics(m =>
            {
                m.AddAspNetCoreInstrumentation();
                if (!string.IsNullOrEmpty(endpoint))
                {
                    m.AddOtlpExporter(o => o.Endpoint = new Uri(endpoint));
                }
            });
        return services;
    }

    /// <summary>System.Environment.GetEnvironmentVariable の薄ラッパ (テスト容易性)。</summary>
    private static string? Environment_GetVariable(string name) => System.Environment.GetEnvironmentVariable(name);
}
