# system-notification-server データベース設計

## スキーマ

スキーマ名: `notification`

```sql
CREATE SCHEMA IF NOT EXISTS notification;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| channels | 通知チャネル定義（Email, Slack 等） |
| templates | 通知テンプレート |
| notification_logs | 通知送信ログ |

---

## ER 図

```
channels 1──* notification_logs
templates 1──* notification_logs
```

---

## テーブル定義

### channels（通知チャネル）

通知の送信チャネル（Email, Slack, Webhook 等）を定義する。config にチャネル固有の設定を保持する。

```sql
CREATE TABLE IF NOT EXISTS notification.channels (
    id           VARCHAR(64)  PRIMARY KEY,
    name         VARCHAR(255) NOT NULL,
    channel_type VARCHAR(50)  NOT NULL,
    config       JSONB        NOT NULL DEFAULT '{}',
    enabled      BOOLEAN      NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_channels_channel_type ON notification.channels (channel_type);
CREATE INDEX IF NOT EXISTS idx_channels_enabled ON notification.channels (enabled);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `ch_`） |
| name | VARCHAR(255) | NOT NULL | チャネル名 |
| channel_type | VARCHAR(50) | NOT NULL | チャネル種別（email, slack 等） |
| config | JSONB | NOT NULL, DEFAULT '{}' | チャネル設定 |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### templates（テンプレート）

通知メッセージのテンプレートを管理する。subject_template と body_template でテンプレート本体を保持する。

```sql
CREATE TABLE IF NOT EXISTS notification.templates (
    id               VARCHAR(64)  PRIMARY KEY,
    name             VARCHAR(255) NOT NULL,
    channel_type     VARCHAR(50)  NOT NULL,
    subject_template TEXT         NOT NULL DEFAULT '',
    body_template    TEXT         NOT NULL,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_templates_channel_type ON notification.templates (channel_type);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `tpl_`） |
| name | VARCHAR(255) | NOT NULL | テンプレート名 |
| channel_type | VARCHAR(50) | NOT NULL | チャネル種別 |
| subject_template | TEXT | NOT NULL, DEFAULT '' | 件名テンプレート |
| body_template | TEXT | NOT NULL | 本文テンプレート |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### notification_logs（通知ログ）

通知の送信履歴とステータスを記録する。リトライ回数・送信日時も管理する。

```sql
CREATE TABLE IF NOT EXISTS notification.notification_logs (
    id            VARCHAR(64)  PRIMARY KEY,
    channel_id    VARCHAR(64)  NOT NULL REFERENCES notification.channels(id),
    template_id   VARCHAR(64)  REFERENCES notification.templates(id),
    recipient     TEXT         NOT NULL,
    subject       TEXT         NOT NULL DEFAULT '',
    body          TEXT         NOT NULL DEFAULT '',
    status        VARCHAR(50)  NOT NULL DEFAULT 'pending',
    error_message TEXT,
    retry_count   INT          NOT NULL DEFAULT 0,
    sent_at       TIMESTAMPTZ,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_notification_logs_status CHECK (status IN ('pending', 'sent', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_notification_logs_channel_id ON notification.notification_logs (channel_id);
CREATE INDEX IF NOT EXISTS idx_notification_logs_status ON notification.notification_logs (status);
CREATE INDEX IF NOT EXISTS idx_notification_logs_created_at ON notification.notification_logs (created_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `notif_`） |
| channel_id | VARCHAR(64) | FK → channels.id, NOT NULL | チャネル ID |
| template_id | VARCHAR(64) | FK → templates.id | テンプレート ID |
| recipient | TEXT | NOT NULL | 送信先 |
| subject | TEXT | NOT NULL, DEFAULT '' | 件名 |
| body | TEXT | NOT NULL, DEFAULT '' | 本文 |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'pending' | ステータス（pending/sent/failed） |
| error_message | TEXT | | エラーメッセージ |
| retry_count | INT | NOT NULL, DEFAULT 0 | リトライ回数 |
| sent_at | TIMESTAMPTZ | | 送信日時 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/notification-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `notification` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_channels.up.sql` | channels テーブル作成 |
| `002_create_channels.down.sql` | テーブル削除 |
| `003_create_templates.up.sql` | templates テーブル作成 |
| `003_create_templates.down.sql` | テーブル削除 |
| `004_create_notification_logs.up.sql` | notification_logs テーブル作成 |
| `004_create_notification_logs.down.sql` | テーブル削除 |
| `005_add_retry_count_to_notification_logs.up.sql` | retry_count カラム追加 |
| `005_add_retry_count_to_notification_logs.down.sql` | カラム削除 |
| `006_add_template_id_index.up.sql` | template_id インデックス追加 |
| `006_add_template_id_index.down.sql` | インデックス削除 |
| `007_add_sent_at_to_notification_logs.up.sql` | sent_at カラム追加 |
| `007_add_sent_at_to_notification_logs.down.sql` | カラム削除 |
| `008_convert_prefixed_ids.up.sql` | UUID → プレフィックス付き VARCHAR(64) ID に変換 |
| `008_convert_prefixed_ids.down.sql` | ID 変換復元 |
| `009_add_template_fk_on_delete.up.sql` | template_id FK に ON DELETE 設定追加 |
| `009_add_template_fk_on_delete.down.sql` | FK 制約復元 |
| `010_add_composite_indexes.up.sql` | 複合インデックス追加 |
| `011_encrypt_channel_config.up.sql` | channels.config 暗号化 |
| `012_add_tenant_id_rls.up.sql` | channels に `tenant_id TEXT NOT NULL` と RLS ポリシー追加（H-012 / H-010 対応） |
| `013_fix_tenant_id_rls_cast.up.sql` | channels の RLS ポリシーに ::TEXT キャスト追加 |
| `014_add_rls_with_check.up.sql` | channels の RLS ポリシーに AS RESTRICTIVE + WITH CHECK 追加 |
| `015_add_templates_logs_tenant_rls.up.sql` | templates / notification_logs に `tenant_id TEXT NOT NULL` と RLS ポリシー追加（HIGH-DB-001 対応） |

> **注記**: マイグレーション番号は C-2（重複番号修正）により 005 以降を再番号付けした（2026-03-19）。旧 005_add_template_id_index → 006、以降連番で繰り上げ。

---

## マルチテナント対応（HIGH-DB-001）

`templates` / `notification_logs` に `tenant_id TEXT NOT NULL` カラムと RLS ポリシーを追加（migration 015）。

```sql
ALTER TABLE notification.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.{table} FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON notification.{table}
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION notification.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_channels_update_updated_at
    BEFORE UPDATE ON notification.channels
    FOR EACH ROW EXECUTE FUNCTION notification.update_updated_at();

CREATE TRIGGER trigger_templates_update_updated_at
    BEFORE UPDATE ON notification.templates
    FOR EACH ROW EXECUTE FUNCTION notification.update_updated_at();

CREATE TRIGGER trigger_notification_logs_update_updated_at
    BEFORE UPDATE ON notification.notification_logs
    FOR EACH ROW EXECUTE FUNCTION notification.update_updated_at();
```
