# Outbox ライブラリ

## 概要

Outbox ライブラリは、Transactional Outbox パターンを実装する Rust クレートである。
ドメインイベントをデータベーストランザクション内で永続化し、非同期でメッセージブローカー（Kafka）へ配信する。

## 配信セマンティクス：At-Least-Once（最低1回）

### 設計方針

本ライブラリは **at-least-once（最低1回）配信** を採用している。
イベントはメッセージブローカーへのディスパッチ **成功後に** パブリッシュ済みとしてマークされる。

```
1. 未パブリッシュイベントを SELECT ... FOR UPDATE SKIP LOCKED で取得
2. イベントを Kafka へ publish
3. publish 成功したイベントのみを published_at = NOW() でマーク
4. publish 失敗したイベントは mark されず、次回ポーリングでリトライ
```

### セマンティクス比較

| 観点 | At-Most-Once（旧実装） | At-Least-Once（現在の実装） |
|------|------------------------|----------------------------|
| 重複イベント | 発生しない | 発生しうる（Kafka 障害後のリトライ時） |
| イベントロスト | Kafka 障害時に発生しうる | 発生しない |
| コンシューマーの要件 | 冪等性不要 | 冪等性の実装を推奨 |
| 実装の複雑性 | シンプル | やや複雑（fetch と mark が分離） |

at-least-once を採用した主な理由：

1. **イベントロストの防止**: Kafka 障害・ネットワーク断でもイベントが失われない
2. **at-least-once の重複は `idempotency_key` で排除可能**: outbox の `id` または `idempotency_key` による冪等性チェックで重複排除できる
3. **projection / saga / billing への安全性**: イベント欠損によるデータ不整合を防ぐ

### トレードオフ

- **メリット**: Kafka 障害でもイベントが失われない。at-least-once の信頼性
- **デメリット**: 重複イベントが発生しうる。コンシューマーに冪等性の考慮が必要

## Fetch-Then-Mark パターン

イベントの取得と mark を分離することで、publish 成功後のみ mark するフローを実現する。

### 動作の詳細

1. `SELECT ... FOR UPDATE SKIP LOCKED` で未パブリッシュイベントを排他的に取得する（mark は行わない）
2. 取得したイベントを Kafka へ publish する
3. publish 成功したイベントの ID を収集する
4. 成功した ID に対してのみ `UPDATE outbox_events SET published_at = NOW()` を実行する
5. publish 失敗したイベントは `published_at` が NULL のまま残り、次回ポーリングで自動リトライ

`FOR UPDATE SKIP LOCKED` により、複数のポーラーが同時に動作しても、同じイベントが二重に取得されることはない。

### コード上の実装箇所

- `OutboxEventSource::fetch_unpublished_events` — イベントの取得インターフェース（mark なし）
- `OutboxEventSource::mark_events_published` — publish 成功後の mark インターフェース
- `OutboxEventPoller::poll_and_publish` — ポーリングループの本体。fetch → dispatch → mark のフロー
- `OutboxEventHandler::handle_event` — イベント種別ごとの変換・Kafka publish ロジック

## idempotency_key による冪等性保証

outbox_events テーブルには `idempotency_key` カラム（NOT NULL, UNIQUE）が設定されている。

### INSERT 時の動作

各サービスの repository は outbox 行を INSERT する際に `ON CONFLICT (idempotency_key) DO NOTHING` を指定する。

```sql
INSERT INTO outbox_events (id, ..., idempotency_key)
VALUES ($1, ..., $7)
ON CONFLICT (idempotency_key) DO NOTHING
```

`idempotency_key` には `event_id`（UUID）を文字列化したものを使用する。

### コンシューマーの冪等性

at-least-once では同一イベントが複数回配信される可能性がある（Kafka 障害後のリトライ時）。
コンシューマーは以下のいずれかで冪等性を担保すること：

- イベント ID をキーとした処理済み判定（処理済みイベント ID テーブル or キャッシュ）
- または、イベントのバージョン番号による楽観的ロック

## スキーマ要件

### search_path の設定

service 系サーバーの DB 接続は、接続 URL に `options=-c search_path%3D{schema}` を指定して、
runtime SQL が正しいスキーマのテーブルを参照するようにすること。

```rust
// config.rs の connection_url() の実装例
format!(
    "postgresql://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path%3D{}",
    user, password, host, port, name, ssl_mode, schema
)
```

各サービスのデフォルトスキーマ：
- order-server: `order_service`
- inventory-server: `inventory_service`
- payment-server: `payment_service`
