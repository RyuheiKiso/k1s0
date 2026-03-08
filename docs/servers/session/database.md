# system-session-server データベース設計

## スキーマ

スキーマ名: `session`

```sql
CREATE SCHEMA IF NOT EXISTS session;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| user_sessions | ユーザーセッション |

---

## ER 図

```
user_sessions（単独テーブル、FK なし）
```

---

## テーブル定義

### user_sessions（ユーザーセッション）

ユーザーのログインセッションを管理する。デバイス情報・IP アドレス・有効期限・無効化状態を追跡する。

```sql
CREATE TABLE IF NOT EXISTS session.user_sessions (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID         NOT NULL,
    device_id        VARCHAR(255),
    device_name      VARCHAR(255),
    device_type      VARCHAR(50),
    ip_address       VARCHAR(45),
    user_agent       TEXT,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ  NOT NULL,
    last_accessed_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    revoked          BOOLEAN      NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON session.user_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON session.user_sessions (expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_revoked ON session.user_sessions (revoked);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー（セッション ID） |
| user_id | UUID | NOT NULL | ユーザー ID |
| device_id | VARCHAR(255) | | デバイス ID |
| device_name | VARCHAR(255) | | デバイス名 |
| device_type | VARCHAR(50) | | デバイス種別 |
| ip_address | VARCHAR(45) | | IP アドレス |
| user_agent | TEXT | | User-Agent |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | セッション作成日時 |
| expires_at | TIMESTAMPTZ | NOT NULL | 有効期限 |
| last_accessed_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 最終アクセス日時 |
| revoked | BOOLEAN | NOT NULL, DEFAULT false | 無効化フラグ |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/session-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `session` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_user_sessions.up.sql` | user_sessions テーブル作成 |
| `002_create_user_sessions.down.sql` | テーブル削除 |
