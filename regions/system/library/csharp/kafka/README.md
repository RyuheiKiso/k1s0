# K1s0.System.Kafka

Kafka 接続設定・管理・ヘルスチェックライブラリ (C#/.NET 10)。

## 機能

- **KafkaConfig**: ブローカー、セキュリティプロトコル (TLS/SASL)、タイムアウト設定
- **KafkaConfigBuilder**: ビルダーパターンによる設定構築
- **KafkaHealthCheck**: Confluent.Kafka を使ったクラスターヘルスチェック
- **TopicConfig**: トピック設定と k1s0 命名規則検証
- **TopicPartitionInfo**: パーティション情報 (リーダー、レプリカ、ISR)

## インストール

```xml
<ProjectReference Include="..\kafka\K1s0.System.Kafka.csproj" />
```

## 使い方

### 設定構築

```csharp
var config = new KafkaConfigBuilder()
    .WithBrokers("kafka1:9092", "kafka2:9092")
    .WithSecurityProtocol("SASL_SSL")
    .WithSasl("PLAIN", "user", "password")
    .WithConsumerGroup("my-service-group")
    .Build();
```

### DI 登録

```csharp
builder.Services.AddK1s0Kafka(config);
```

### ヘルスチェック

```csharp
public class MyService(IKafkaHealthCheck healthCheck)
{
    public async Task<bool> IsKafkaHealthy()
    {
        var result = await healthCheck.CheckHealthAsync();
        return result == HealthCheckResult.Healthy;
    }
}
```

### トピック命名規則検証

```csharp
var topic = new TopicConfig
{
    TopicName = "k1s0.system.auth.user-created.v1",
    NumPartitions = 3,
    ReplicationFactor = 3,
};
bool valid = topic.ValidateName(); // true
```

## テスト

```bash
dotnet test regions/system/library/csharp/kafka/tests/
```
