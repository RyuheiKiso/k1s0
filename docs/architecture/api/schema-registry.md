# Schema Registry 統合設計

## 概要
k1s0プラットフォームは Kafka メッセージングのスキーマ管理に Confluent Schema Registry を使用する。
Proto + Schema Registry の連携により、メッセージスキーマの後方互換性を保証する。

## アーキテクチャ

### Schema Registry の役割
- **スキーマ登録**: Kafka メッセージの Proto スキーマを中央管理
- **互換性検証**: Producer/Consumer 間のスキーマ互換性を自動検証
- **バージョン管理**: スキーマの変更履歴を追跡

### 統合パターン

```
Producer                    Schema Registry             Consumer
   |                               |                        |
   |-- POST /subjects/{topic}/versions --> |                |
   |<-- schema_id: 42 ------------ |                        |
   |-- Kafka (schema_id:42 + data) -----------------------> |
   |                               |                        |
   |                               | <-- GET /schemas/ids/42|
   |                               | --> Proto schema -----> |
```

## Proto スキーマ管理

### Subject 命名規則
```
{topic-name}-value   # メッセージ値のスキーマ
{topic-name}-key     # メッセージキーのスキーマ（使用する場合）
```

### トピックとスキーマのマッピング

| トピック | Subject | Proto型 |
|---------|---------|---------|
| k1s0.system.saga.events | k1s0.system.saga.events-value | SagaEvent |
| k1s0.system.auth.events | k1s0.system.auth.events-value | AuthEvent |
| k1s0.system.tenant.events | k1s0.system.tenant.events-value | TenantEvent |

## 互換性ポリシー

### デフォルト互換性: BACKWARD
新しいスキーマで旧メッセージを読めることを保証（Consumer を先にデプロイ可能）。

### フィールド変更規則
- **追加可能**: 新フィールドは `optional` で追加（デフォルト値設定必須）
- **削除禁止**: フィールドは削除せず `reserved` に移行（移行計画は `types.proto` 参照）
- **型変更禁止**: フィールドの型変更は後方互換性を破壊するため禁止

### 破壊的変更の対処
1. 新しいトピック名で新スキーマを登録
2. デュアルライト期間（旧新両方にPublish）
3. 全Consumerの移行を確認後、旧トピックをDeprecate

## ライブラリ統合

### k1s0-schemaregistry ライブラリ
`regions/system/library/*/schemaregistry/` に実装。

```go
// Golang: スキーマ登録付きProducer
client := schemaregistry.NewClient(schemaRegistryURL)
encoder := schemaregistry.NewProtoEncoder(client, subject)
msg, _ := encoder.Encode(protoMessage)
```

```rust
// Rust: スキーマ検証付きConsumer
let client = SchemaRegistryClient::new(schema_registry_url);
let decoder = ProtoDecoder::new(client);
let event: SagaEvent = decoder.decode(&kafka_message).await?;
```

## CI/CD統合

### スキーマ変換検証（buf breaking）
```yaml
# .github/workflows での buf breaking check
- name: Proto 破壊的変更チェック
  run: buf breaking --against ".git#branch=main"
```

### Schema Registry への自動登録
main ブランチへのマージ後、CI が自動的に Schema Registry にスキーマを登録する。
登録失敗はデプロイをブロックする。

## 段階的移行計画

### フェーズ 1: 基盤整備（現在）
- Confluent Schema Registry のインフラセットアップ（`infra/docker/schema-registry/`）
- k1s0-schemaregistry ライブラリの実装

### フェーズ 2: 既存トピックの移行
- 優先度: Saga イベント → Auth イベント → Tenant イベント
- 各サービスで schemaregistry ライブラリを依存追加

### フェーズ 3: 全トピック対応
- 全 Kafka トピックのスキーマ登録完了
- CI での自動互換性チェック有効化

## 参照

- `api/proto/k1s0/system/common/v1/types.proto` — Timestamp型移行計画コメント
- `regions/system/library/*/schemaregistry/` — ライブラリ実装
- `docs/architecture/api/` — API設計全般

## 更新履歴

| 日付 | 変更内容 |
|------|---------|
| 2026-03-21 | 初版作成（技術品質監査対応 P2-29） |
