# K1s0.System.Telemetry

OpenTelemetry + Serilog ベースの可観測性ライブラリ。トレース、メトリクス（RED メソッド）、構造化ログを提供する。

## インストール

```xml
<ProjectReference Include="path/to/K1s0.System.Telemetry.csproj" />
```

## 使用例

### DI 登録

```csharp
using K1s0.System.Telemetry;

services.AddK1s0Telemetry(new TelemetryConfig
{
    ServiceName = "my-service",
    Version = "1.0.0",
    Tier = "system",
    Environment = "dev",
    Endpoint = "http://localhost:4317",
    SampleRate = 1.0,
    LogLevel = "info",
    LogFormat = "json",
});
```

### HTTP ミドルウェア

```csharp
app.UseMiddleware<HttpTelemetryMiddleware>();
```

### gRPC インターセプター

```csharp
services.AddGrpc(opts =>
{
    opts.Interceptors.Add<GrpcTelemetryInterceptor>();
});
```

### ロガー

```csharp
using K1s0.System.Telemetry;

var logger = K1s0Logger.NewLogger(config);
logger.Information("Service started");
```

## メトリクス

| メトリクス | 種別 | 説明 |
|-----------|------|------|
| `k1s0.request.total` | Counter | リクエスト総数 |
| `k1s0.request.duration` | Histogram | リクエスト処理時間（ms） |
| `k1s0.request.errors` | Counter | エラーリクエスト数（4xx/5xx） |
| `k1s0.request.in_flight` | UpDownCounter | 処理中リクエスト数 |
