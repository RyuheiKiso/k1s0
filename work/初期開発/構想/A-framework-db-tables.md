## 付録A. framework 共通テーブル（仕様メモ）

注意: DDL の正本は `framework/database/table/*.sql`。この付録は読みやすさのための要約であり、実装・レビューは SQL を基準に行う。

### fw_m_setting

| カラム名       | データ型     | NULL許可 | デフォルト値      | 説明                                     |
| ------------ | ------------ | -------- | ----------------- | ---------------------------------------- |
| setting_id   | BIGSERIAL    | NO       |                   | 設定ID (PK)                              |
| service_name | VARCHAR(100) | NO       |                   | サービス名（設定のスコープ）             |
| env          | VARCHAR(20)  | NO       | default           | 環境（例: dev / stg / prod / default）   |
| setting_key  | VARCHAR(150) | NO       |                   | 設定キー（例: feature.flag_x / timeout） |
| value_type   | VARCHAR(30)  | NO       | string            | 値の型（string / int / bool / json 等）  |
| setting_value| TEXT         | YES      |                   | 設定値（型は value_type で解釈）         |
| description  | TEXT         | YES      |                   | 説明                                     |
| status       | SMALLINT     | NO       | 1                 | 状態（1:有効, 0:無効）                   |
| created_at   | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                                 |
| updated_at   | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                                 |
| deleted_at   | TIMESTAMPTZ  | YES      |                   | 削除日時（論理削除）                     |

### fw_m_user

| カラム名             | データ型     | NULL許可 | デフォルト値      | 説明                                    |
| ------------------- | ------------ | -------- | ----------------- | --------------------------------------- |
| user_id             | BIGSERIAL    | NO       |                   | ユーザーID (PK)                         |
| login_id            | VARCHAR(100) | NO       |                   | ログインID (ユニーク)                   |
| email               | VARCHAR(255) | NO       |                   | メールアドレス (ユニーク)               |
| password_hash       | TEXT         | YES      |                   | パスワードハッシュ（OAuth等の場合NULL） |
| display_name        | VARCHAR(100) | NO       |                   | 表示名                                  |
| status              | SMALLINT     | NO       | 1                 | 状態（1:有効, 0:無効）                  |
| last_login_at       | TIMESTAMPTZ  | YES      |                   | 最終ログイン日時                        |
| password_updated_at | TIMESTAMPTZ  | YES      |                   | パスワード更新日時                      |
| created_at          | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                                |
| updated_at          | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                                |
| deleted_at          | TIMESTAMPTZ  | YES      |                   | 削除日時（論理削除）                    |

### fw_m_permission

| カラム名        | データ型     | NULL許可 | デフォルト値      | 説明                         |
| -------------- | ------------ | -------- | ----------------- | ---------------------------- |
| permission_id  | BIGSERIAL    | NO       |                   | 権限ID (PK)                  |
| service_name   | VARCHAR(100) | NO       |                   | サービス名（権限のスコープ） |
| permission_key | VARCHAR(150) | NO       |                   | 権限キー（例: user:read）    |
| description    | TEXT         | YES      |                   | 説明                         |
| status         | SMALLINT     | NO       | 1                 | 状態（1:有効, 0:無効）       |
| created_at     | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                     |
| updated_at     | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                     |
| deleted_at     | TIMESTAMPTZ  | YES      |                   | 削除日時（論理削除）         |

### fw_m_role

| カラム名     | データ型     | NULL許可 | デフォルト値      | 説明                   |
| ----------- | ------------ | -------- | ----------------- | ---------------------- |
| role_id     | BIGSERIAL    | NO       |                   | ロールID (PK)          |
| role_name   | VARCHAR(100) | NO       |                   | ロール名（ユニーク）   |
| description | TEXT         | YES      |                   | 説明                   |
| status      | SMALLINT     | NO       | 1                 | 状態（1:有効, 0:無効） |
| created_at  | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時               |
| updated_at  | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時               |
| deleted_at  | TIMESTAMPTZ  | YES      |                   | 削除日時（論理削除）   |

### fw_m_user_role

| カラム名   | データ型    | NULL許可 | デフォルト値      | 説明                         |
| ---------- | ----------- | -------- | ----------------- | ---------------------------- |
| user_id    | BIGINT      | NO       |                   | ユーザーID（fw_m_user 参照） |
| role_id    | BIGINT      | NO       |                   | ロールID（fw_m_role 参照）   |
| created_at | TIMESTAMPTZ | NO       | CURRENT_TIMESTAMP | 作成日時                     |

### fw_m_role_permission

| カラム名      | データ型    | NULL許可 | デフォルト値      | 説明                           |
| ------------- | ----------- | -------- | ----------------- | ------------------------------ |
| role_id       | BIGINT      | NO       |                   | ロールID（fw_m_role 参照）     |
| permission_id | BIGINT      | NO       |                   | 権限ID（fw_m_permission 参照） |
| created_at    | TIMESTAMPTZ | NO       | CURRENT_TIMESTAMP | 作成日時                       |

### fw_m_endpoint

| カラム名      | データ型     | NULL許可 | デフォルト値      | 説明                  |
| ------------ | ------------ | -------- | ----------------- | --------------------- |
| endpoint_id  | SERIAL       | NO       |                   | エンドポイントID (PK) |
| service_name | VARCHAR(100) | NO       |                   | サービス名            |
| path         | VARCHAR(255) | NO       |                   | エンドポイントのパス  |
| method       | VARCHAR(10)  | NO       |                   | HTTPメソッド          |
| created_at   | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時              |
| updated_at   | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時              |

---


