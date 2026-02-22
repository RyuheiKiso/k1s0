# system-dlq-database設計

system Tier のデッドレターキュー管理データベース（dlq-db）の設計を定義する。
配置先: `regions/system/database/dlq-db/`

## 概要

dlq-db は system Tier に属する PostgreSQL 17 データベースであり、Kafka メッセージ処理に失敗したメッセージ（デッドレター）の管理・リトライ・アーカイブを担う。

[tier-architecture.md](tier-architecture.md) の設計原則に従い、dlq-db へのアクセスは **system Tier の dlq-manager サーバーからのみ** 許可する。

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

Kafka メッセージ処理に失敗したメッセージを格納する。リトライ制御（retry_count / max_retries）とステータス管理（PENDING / RETRYING / RESOLVED / DEAD）を持つ。

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

```
PENDING ──> RETRYING ──> PENDING   (リトライ成功せずカウント+1)
                    ──> RESOLVED  (リトライ成功)
                    ──> DEAD      (max_retries 到達)
```

| ステータス | 説明 |
|-----------|------|
| PENDING | リトライ待ち。dlq-manager がリトライ対象として取得する |
| RETRYING | リトライ処理中。dlq-manager が再処理を実行している |
| RESOLVED | リトライ成功。30日後にアーカイブへ移動 |
| DEAD | 最大リトライ回数到達。手動対応が必要 |

### dlq_messages_archive テーブル

dlq_messages と同一スキーマを持つアーカイブテーブル。RESOLVED / DEAD 状態で 30 日経過したメッセージが自動的にアーカイブされる。

`CREATE TABLE dlq.dlq_messages_archive (LIKE dlq.dlq_messages INCLUDING ALL)` で作成される。

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| dlq_messages | idx_dlq_messages_topic | original_topic | B-tree | トピック別のメッセージ検索 |
| dlq_messages | idx_dlq_messages_status | status | B-tree | ステータス別のメッセージ検索（リトライ対象取得） |
| dlq_messages | idx_dlq_messages_created_at | created_at | B-tree | 作成日時による範囲検索 |

### 設計方針

- **ステータスインデックス**: リトライ対象の PENDING メッセージ取得が最も頻繁なクエリであるため、status インデックスが最重要
- **トピックインデックス**: 特定トピックの障害状況を調査する際に使用する

---

## パーティション戦略

### 現在の設計

dlq_messages テーブルは現時点ではパーティショニングを適用しない。理由は以下の通り:

1. **メッセージ数が限定的**: DLQ は異常系のメッセージのみを格納するため、通常運用ではレコード数が少ない
2. **アーカイブプロシージャ**: 30日経過した RESOLVED / DEAD メッセージは `dlq.archive_old_dlq_messages()` プロシージャで自動的にアーカイブテーブルへ移動される
3. **テーブルサイズ管理**: アーカイブにより本テーブルのサイズが一定範囲に保たれる

### 将来的な拡張

DLQ メッセージ量が増加した場合は、以下のパーティション戦略を検討する:

- **dlq_messages**: created_at による月次レンジパーティショニング
- **dlq_messages_archive**: created_at による四半期レンジパーティショニング

---

## リテンションポリシー

| 対象 | 保持期間 | 処理 |
|------|---------|------|
| dlq_messages (PENDING / RETRYING) | 無期限 | リトライ処理が完了するまで保持 |
| dlq_messages (RESOLVED / DEAD) | 30日 | アーカイブプロシージャで dlq_messages_archive へ移動 |
| dlq_messages_archive | 365日 | 年次で古いレコードを削除（手動または cron） |

### アーカイブプロシージャ

```sql
CREATE OR REPLACE PROCEDURE dlq.archive_old_dlq_messages()
LANGUAGE plpgsql AS $$
BEGIN
    -- RESOLVED / DEAD で30日経過したメッセージをアーカイブ
    INSERT INTO dlq.dlq_messages_archive
        SELECT * FROM dlq.dlq_messages
        WHERE status IN ('RESOLVED', 'DEAD')
          AND updated_at < NOW() - INTERVAL '30 days';

    -- アーカイブ済みメッセージを本テーブルから削除
    DELETE FROM dlq.dlq_messages
    WHERE status IN ('RESOLVED', 'DEAD')
      AND updated_at < NOW() - INTERVAL '30 days';
END;
$$;
```

このプロシージャは CronJob（`infra/kubernetes/system/partition-cronjob.yaml` のパターンを参照）で定期実行する。

---

## 主要クエリパターン

### リトライ対象メッセージの取得

```sql
-- PENDING 状態のメッセージを取得（リトライ対象）
SELECT id, original_topic, error_message, retry_count, max_retries, payload
FROM dlq.dlq_messages
WHERE status = 'PENDING'
  AND retry_count < max_retries
ORDER BY created_at ASC
LIMIT $1
FOR UPDATE SKIP LOCKED;
```

### リトライ実行

```sql
-- ステータスを RETRYING に更新
UPDATE dlq.dlq_messages
SET status = 'RETRYING', last_retry_at = NOW()
WHERE id = $1;

-- リトライ成功
UPDATE dlq.dlq_messages
SET status = 'RESOLVED'
WHERE id = $1;

-- リトライ失敗（カウント+1）
UPDATE dlq.dlq_messages
SET retry_count = retry_count + 1,
    status = CASE WHEN retry_count + 1 >= max_retries THEN 'DEAD' ELSE 'PENDING' END
WHERE id = $1;
```

### トピック別の障害状況

```sql
-- トピック別の DLQ メッセージ数
SELECT original_topic, status, COUNT(*) as count
FROM dlq.dlq_messages
GROUP BY original_topic, status
ORDER BY original_topic, status;
```

---

## 接続設定

### config.yaml（dlq-manager サーバー用）

```yaml
app:
  name: "dlq-manager"
  version: "1.0.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "dlq_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 10
  max_idle_conns: 3
  conn_max_lifetime: "5m"
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/dlq-manager/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/dlq-manager-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/dlq-manager-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [system-database設計](system-database設計.md) -- auth-db テーブル設計（参照パターン）
- [tier-architecture](tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [docker-compose設計](docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](可観測性設計.md) -- OpenTelemetry トレース ID 連携
