> **ステータス: Go 実装済み / Rust 設計のみ**

> **Go 実装**: `regions/system/library/go/codegen/event_codegen.go`
> `k1s0-codegen` Go モジュール (`github.com/k1s0-platform/system-library-go-codegen`) の `GenerateEventCode()` 関数として提供。
>
> **Rust 実装形態**: `k1s0-codegen` クレートの `event-codegen` feature として実装予定。
> 実装パス: `regions/system/library/rust/codegen/src/event_codegen/`（未実装）

# イベントコード生成 (event-codegen)

## 概要

`events.yaml` 定義ファイルから Kafka イベント連携に必要なコードを一括自動生成する CLI コマンド。

## 解決する課題

tier2 (business) 開発者が Kafka イベント連携を実装する際、以下を手動で書く必要があった:
- Producer 設定
- Outbox テーブル定義
- Consumer ハンドラ
- Proto スキーマ
- Schema Registry 設定

設定ステップが多く、トピック命名規約違反・スキーマ不一致・Outbox テーブル定義忘れなどのミスが頻発していた。

## YAML スキーマ

```yaml
# events.yaml
domain: accounting
tier: business
service_name: domain-master
language: rust  # rust | go

events:
  - name: master-item.created     # kebab-case + ドット区切り
    version: 1
    description: "マスタアイテムが作成された時に発行されるイベント"
    partition_key: item_id          # schema.fields 内のフィールド名
    outbox: true                    # default: true
    schema:
      fields:
        - name: item_id
          type: string              # proto3 有効型
          number: 1
          description: "アイテムID"
    consumers:
      - domain: fa
        service_name: asset-manager
        handler: on_accounting_master_item_created  # snake_case
```

## バリデーションルール

| ルール | 詳細 |
|--------|------|
| domain | 非空、kebab-case |
| tier | `system` / `business` / `service` |
| service_name | 非空、kebab-case |
| language | `rust` / `go` |
| events | 1つ以上 |
| event.name | kebab-case + ドット区切り |
| event.version | >= 1 |
| schema.fields | 1つ以上 |
| field.number | >= 1、イベント内で重複なし |
| field.type | proto3 有効型 |
| partition_key | schema 内のフィールド名と一致 |
| consumer.handler | snake_case |
| イベント名 | 重複なし |

## 生成ファイル

### Rust プロジェクト

| ファイル | 内容 |
|----------|------|
| `proto/{domain}/events/v{ver}/{name}.proto` | Proto スキーマ |
| `src/events/mod.rs` | モジュール定義 |
| `src/events/types.rs` | イベント型定義 |
| `src/events/producer.rs` | Producer + Outbox 関数 |
| `src/events/consumers/{handler}.rs` | Consumer ハンドラ |
| `migrations/{n}_create_outbox.up.sql` | Outbox UP |
| `migrations/{n}_create_outbox.down.sql` | Outbox DOWN |
| `config/schema-registry.yaml` | Schema Registry 設定 |

### Go プロジェクト

| ファイル | 内容 |
|----------|------|
| `proto/{domain}/events/v{ver}/{name}.proto` | Proto スキーマ |
| `internal/events/producer.go` | Producer + Outbox 関数 |
| `internal/events/consumers/{handler}.go` | Consumer ハンドラ |
| `migrations/{n}_create_outbox.up.sql` | Outbox UP |
| `migrations/{n}_create_outbox.down.sql` | Outbox DOWN |
| `config/schema-registry.yaml` | Schema Registry 設定 |

## トピック命名規約

```
k1s0.{tier}.{domain}.{name-hyphenated}.v{version}
```

例: `k1s0.business.accounting.master-item-created.v1`

## 使い方

1. プロジェクトルートに `events.yaml` を作成
2. CLI メニューから「イベントコード生成」を選択
3. events.yaml のパスを入力（デフォルト: `events.yaml`）
4. サマリーを確認して実行

## 依存ライブラリ

- `messaging` (k1s0-messaging): EventEnvelope, EventMetadata, KafkaEventProducer, ConsumedMessage
- `outbox` (k1s0-outbox): OutboxMessage, OutboxStore
