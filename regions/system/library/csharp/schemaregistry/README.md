# K1s0.System.SchemaRegistry

Confluent Schema Registry クライアントライブラリ (C#/.NET 10)。

## 機能

- **スキーマ登録**: Avro, JSON, Protobuf スキーマの登録
- **スキーマ取得**: ID によるスキーマ取得
- **互換性チェック**: サブジェクトに対するスキーマ互換性検証
- **認証対応**: Basic 認証によるレジストリアクセス
- **DI 統合**: `AddK1s0SchemaRegistry()` による依存注入設定

## インストール

```xml
<ProjectReference Include="..\schemaregistry\K1s0.System.SchemaRegistry.csproj" />
```

## 使い方

### DI 登録

```csharp
var config = new SchemaRegistryConfig
{
    Url = "http://schema-registry:8081",
    CompatibilityMode = CompatibilityMode.Backward,
};

builder.Services.AddK1s0SchemaRegistry(config);
```

### スキーマ登録

```csharp
public class MyService(ISchemaRegistryClient client)
{
    public async Task<int> RegisterAsync(string topic, string schema)
    {
        var subject = SchemaRegistryConfig.SubjectName(topic);
        return await client.RegisterSchemaAsync(subject, schema, SchemaType.Avro);
    }
}
```

### スキーマ取得

```csharp
var schema = await client.GetSchemaByIdAsync(42);
```

### 互換性チェック

```csharp
var compatible = await client.CheckCompatibilityAsync("my-subject", newSchema);
```

## テスト

```bash
dotnet test regions/system/library/csharp/schemaregistry/tests/
```

## 主要な型

| 型 | 説明 |
|---|---|
| `SchemaRegistryConfig` | URL, 認証情報, 互換性モード設定 |
| `ISchemaRegistryClient` | スキーマ操作インターフェース |
| `ConfluentSchemaRegistryClient` | Confluent SDK ベースの実装 |
| `RegisteredSchema` | 登録済みスキーマ (ID, Version, SchemaString, SchemaType) |
| `SchemaType` | Avro, Json, Protobuf |
| `CompatibilityMode` | Backward, Forward, Full, None |
| `SchemaRegistryException` | エラー例外 (Code プロパティ付き) |
