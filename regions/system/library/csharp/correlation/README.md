# K1s0.System.Correlation

分散トレーシング用の相関 ID / トレース ID 管理ライブラリ。サービス間リクエストの追跡に使用する。

## インストール

```xml
<ProjectReference Include="path/to/K1s0.System.Correlation.csproj" />
```

## 使用例

### コンテキスト生成

```csharp
using K1s0.System.Correlation;

// 新規コンテキスト生成
var ctx = CorrelationContext.New();
Console.WriteLine(ctx.CorrelationId); // UUID v4
Console.WriteLine(ctx.TraceId);       // 32文字 lowercase hex
```

### HTTP ヘッダーでの伝播

```csharp
// リクエスト送信時
httpClient.DefaultRequestHeaders.Add(
    CorrelationHeaders.CorrelationId, ctx.CorrelationId);
httpClient.DefaultRequestHeaders.Add(
    CorrelationHeaders.TraceId, ctx.TraceId);
```

### DI 登録

```csharp
services.AddK1s0Correlation();

// コンストラクタインジェクション（リクエストスコープ）
public class MyHandler(CorrelationContext correlation)
{
    // correlation.CorrelationId, correlation.TraceId
}
```

## ヘッダー定数

| 定数 | 値 |
|------|-----|
| `CorrelationHeaders.CorrelationId` | `X-Correlation-Id` |
| `CorrelationHeaders.TraceId` | `X-Trace-Id` |
| `CorrelationHeaders.RequestId` | `X-Request-Id` |
