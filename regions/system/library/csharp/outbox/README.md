# K1s0.System.Outbox

Transactional Outbox パターンライブラリ。データベーストランザクションとメッセージ発行の原子性を保証する。

## 機能

- `IOutboxStore` - アウトボックスメッセージの永続化抽象
- `PostgresOutboxStore` - PostgreSQL (Npgsql + Dapper) による実装
- `OutboxProcessor` - BackgroundService によるポーリング発行プロセッサ
- 指数バックオフリトライ
- DI 拡張メソッド

## 使い方

```csharp
services.AddK1s0Outbox(new OutboxConfig(
    ConnectionString: "Host=localhost;Database=mydb",
    PollingInterval: TimeSpan.FromSeconds(5),
    MaxRetries: 5,
    BackoffBase: TimeSpan.FromSeconds(1)));
```

`IEventProducer` の実装を別途登録してください。

## テーブルスキーマ

```sql
CREATE TABLE outbox_messages (
    id UUID PRIMARY KEY,
    topic TEXT NOT NULL,
    payload BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    retry_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_error TEXT
);
```

## テスト

```bash
dotnet test tests/
```
