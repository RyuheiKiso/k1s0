# system-dlq-database設計

> **ガイド**: 設計背景・実装例は [database.guide.md](./database.guide.md) を参照。

system Tier のデッドレターキュー管理データベース（dlq-db）の設計を定義する。
配置先: `regions/system/database/dlq-db/`

## 概要

dlq-db は system Tier に属する PostgreSQL 17 データベースであり、Kafka メッセージ処理に失敗したメッセージ（デッドレター）の管理・リトライ・アーカイブを担う。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、dlq-db へのアクセスは **system Tier の dlq-manager サーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx（Rust） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER図

```
┌──────────────────────┐
│   dlq_messages       │
├──────────────────────┤
│ id (PK)              │
│ original_topic       │
│ error_message        │
│ retry_count          │
│ max_retries          │
│ payload (JSONB)      │
│ status               │
│ created_at           │
│ updated_at           │
│ last_retry_at        │
└──────────────────────┘

┌──────────────────────┐
│ dlq_messages_archive │
├──────────────────────┤
│ (dlq_messages と     │
│  同一スキーマ)       │
└──────────────────────┘
```

### リレーション

| 関係 | 説明 |
|------|------|
| dlq_messages -> dlq_messages_archive | アーカイブプロシージャにより RESOLVED / DEAD 状態のメッセージを移動 |

---

## テーブル定義

### dlq_messages テーブル

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | メッセージ識別子 |
| original_topic | VARCHAR(255) | NOT NULL | 失敗元の Kafka トピック名 |
| error_message | TEXT | NOT NULL | エラーメッセージ（処理失敗の理由） |
| retry_count | INT | NOT NULL DEFAULT 0 | 現在のリトライ回数 |
| max_retries | INT | NOT NULL DEFAULT 3 | 最大リトライ回数 |
| payload | JSONB | | 元メッセージのペイロード |
| status | VARCHAR(50) | NOT NULL DEFAULT 'PENDING' | メッセージステータス |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時（DLQ 投入日時） |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |
| last_retry_at | TIMESTAMPTZ | | 最終リトライ実行日時 |

**制約**: `status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD')`

### ステータス遷移

| ステータス | 説明 |
|-----------|------|
| PENDING | リトライ待ち。dlq-manager がリトライ対象として取得する |
| RETRYING | リトライ処理中。dlq-manager が再処理を実行している |
| RESOLVED | リトライ成功。30日後にアーカイブへ移動 |
| DEAD | 最大リトライ回数到達。手動対応が必要 |

### dlq_messages_archive テーブル

dlq_messages と同一スキーマを持つアーカイブテーブル。RESOLVED / DEAD 状態で 30 日経過したメッセージが自動的にアーカイブされる。

---

## マイグレーションファイル

配置先: `regions/system/database/dlq-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_schema | pgcrypto 拡張・dlq スキーマ・`update_updated_at()` 関数の作成 |
| 002 | create_dlq_messages | dlq_messages テーブル・インデックス・updated_at トリガー |
| 003 | add_partition_management | dlq_messages_archive テーブル作成・バッチ対応アーカイブプロシージャ |

> マイグレーション SQL 全文は [database.guide.md](./database.guide.md#マイグレーション-sql) を参照。

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| dlq_messages | idx_dlq_messages_original_topic | original_topic | B-tree | トピック別のメッセージ検索 |
| dlq_messages | idx_dlq_messages_status | status | B-tree | ステータス別のメッセージ検索（リトライ対象取得） |
| dlq_messages | idx_dlq_messages_created_at | created_at | B-tree | 作成日時による範囲検索 |

> インデックス設計方針は [database.guide.md](./database.guide.md#インデックス設計方針) を参照。

---

## リテンションポリシー

| 対象 | 保持期間 | 処理 |
|------|---------|------|
| dlq_messages (PENDING / RETRYING) | 無期限 | リトライ処理が完了するまで保持 |
| dlq_messages (RESOLVED / DEAD) | 30日 | アーカイブプロシージャで dlq_messages_archive へ移動 |
| dlq_messages_archive | 365日 | 年次で古いレコードを削除（手動または cron） |

> アーカイブプロシージャの実行例は [database.guide.md](./database.guide.md#アーカイブプロシージャの実行例) を参照。

---

## 接続設定

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/dlq-manager/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/dlq-manager-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/dlq-manager-ro` | TTL: 24時間 |

> config.yaml 例は [database.guide.md](./database.guide.md#接続設定例) を参照。

---

## 関連ドキュメント

- [system-database設計](../_common/database.md) -- auth-db テーブル設計（参照パターン）
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- OpenTelemetry トレース ID 連携
