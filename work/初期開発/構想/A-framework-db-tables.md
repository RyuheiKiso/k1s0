## 付録A. framework 共通テーブル（仕様メモ）

注意: DDL の正本は各サービスの `migrations/*.sql`。この付録は読みやすさのための要約であり、実装・レビューは SQL を基準に行う。

- auth-service: `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission`
- config-service: `fw_m_setting`, `fw_h_setting`
- endpoint-service: `fw_m_endpoint`, `fw_m_service_address`, `fw_m_endpoint_permission`

---

## auth-service 所有テーブル

### fw_m_user

| カラム名            | データ型     | NULL許可 | デフォルト値      | 説明                                   |
| ------------------- | ------------ | -------- | ----------------- | -------------------------------------- |
| user_id             | BIGSERIAL    | NO       |                   | ユーザーID (PK)                        |
| login_id            | VARCHAR(255) | NO       |                   | ログインID (ユニーク)                  |
| email               | VARCHAR(255) | NO       |                   | メールアドレス (ユニーク)              |
| display_name        | VARCHAR(255) | NO       |                   | 表示名                                 |
| password_hash       | VARCHAR(255) | NO       |                   | パスワードハッシュ                     |
| status              | SMALLINT     | NO       | 1                 | ステータス (0:無効, 1:有効, 2:ロック)  |
| failed_login_count  | SMALLINT     | NO       | 0                 | ログイン失敗回数                       |
| last_login_at       | TIMESTAMPTZ  | YES      |                   | 最終ログイン日時                       |
| password_changed_at | TIMESTAMPTZ  | YES      |                   | パスワード変更日時                     |
| created_at          | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                               |
| updated_at          | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                               |
| created_by          | BIGINT       | YES      |                   | 作成者ユーザーID                       |
| updated_by          | BIGINT       | YES      |                   | 更新者ユーザーID                       |

インデックス: `idx_fw_m_user_login_id`, `idx_fw_m_user_email`, `idx_fw_m_user_status`

### fw_m_role

| カラム名    | データ型     | NULL許可 | デフォルト値      | 説明                       |
| ----------- | ------------ | -------- | ----------------- | -------------------------- |
| role_id     | BIGSERIAL    | NO       |                   | ロールID (PK)              |
| role_name   | VARCHAR(100) | NO       |                   | ロール名 (ユニーク)        |
| description | TEXT         | YES      |                   | 説明                       |
| is_system   | BOOLEAN      | NO       | FALSE             | システムロール（削除不可） |
| created_at  | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                   |
| updated_at  | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                   |
| created_by  | BIGINT       | YES      |                   | 作成者ユーザーID           |
| updated_by  | BIGINT       | YES      |                   | 更新者ユーザーID           |

インデックス: `idx_fw_m_role_name`

初期データ: admin（システム管理者）, user（一般ユーザー）, viewer（閲覧専用）

### fw_m_permission

| カラム名       | データ型     | NULL許可 | デフォルト値      | 説明                         |
| -------------- | ------------ | -------- | ----------------- | ---------------------------- |
| permission_id  | BIGSERIAL    | NO       |                   | パーミッションID (PK)        |
| permission_key | VARCHAR(255) | NO       |                   | パーミッションキー (ユニーク, 例: "user:read") |
| resource_type  | VARCHAR(100) | NO       |                   | リソースタイプ (例: "user")  |
| operation      | VARCHAR(50)  | NO       |                   | 操作 (例: "read", "write")   |
| description    | TEXT         | YES      |                   | 説明                         |
| service_name   | VARCHAR(100) | YES      |                   | サービススコープ（NULLは全サービス） |
| created_at     | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                     |
| updated_at     | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                     |
| created_by     | BIGINT       | YES      |                   | 作成者ユーザーID             |
| updated_by     | BIGINT       | YES      |                   | 更新者ユーザーID             |

インデックス: `idx_fw_m_permission_key`, `idx_fw_m_permission_resource`, `idx_fw_m_permission_service`

### fw_m_user_role

| カラム名   | データ型    | NULL許可 | デフォルト値      | 説明                         |
| ---------- | ----------- | -------- | ----------------- | ---------------------------- |
| user_id    | BIGINT      | NO       |                   | ユーザーID (PK, FK)          |
| role_id    | BIGINT      | NO       |                   | ロールID (PK, FK)            |
| granted_at | TIMESTAMPTZ | NO       | CURRENT_TIMESTAMP | 付与日時                     |
| granted_by | BIGINT      | YES      |                   | 付与者ユーザーID             |
| expires_at | TIMESTAMPTZ | YES      |                   | 有効期限（NULLは無期限）     |

インデックス: `idx_fw_m_user_role_user`, `idx_fw_m_user_role_role`

### fw_m_role_permission

| カラム名      | データ型    | NULL許可 | デフォルト値      | 説明                  |
| ------------- | ----------- | -------- | ----------------- | --------------------- |
| role_id       | BIGINT      | NO       |                   | ロールID (PK, FK)     |
| permission_id | BIGINT      | NO       |                   | パーミッションID (PK, FK) |
| granted_at    | TIMESTAMPTZ | NO       | CURRENT_TIMESTAMP | 付与日時              |
| granted_by    | BIGINT      | YES      |                   | 付与者ユーザーID      |

インデックス: `idx_fw_m_role_permission_role`, `idx_fw_m_role_permission_perm`

---

## config-service 所有テーブル

### fw_m_setting

| カラム名      | データ型     | NULL許可 | デフォルト値      | 説明                                   |
| ------------- | ------------ | -------- | ----------------- | -------------------------------------- |
| setting_id    | BIGSERIAL    | NO       |                   | 設定ID (PK)                            |
| setting_key   | VARCHAR(255) | NO       |                   | 設定キー                               |
| setting_value | TEXT         | NO       |                   | 設定値                                 |
| value_type    | VARCHAR(50)  | NO       | string            | 値の型 (string, number, boolean, json) |
| description   | TEXT         | YES      |                   | 説明                                   |
| category      | VARCHAR(100) | YES      |                   | カテゴリ                               |
| service_name  | VARCHAR(100) | YES      |                   | サービス名（NULLは全サービス共通）     |
| environment   | VARCHAR(50)  | YES      |                   | 環境（NULLは全環境共通）               |
| is_sensitive  | BOOLEAN      | NO       | FALSE             | 機密フラグ（ログ出力時にマスク）       |
| is_readonly   | BOOLEAN      | NO       | FALSE             | 読み取り専用フラグ                     |
| version       | INTEGER      | NO       | 1                 | バージョン                             |
| valid_from    | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 有効開始日時                           |
| valid_to      | TIMESTAMPTZ  | YES      |                   | 有効終了日時（NULLは無期限）           |
| created_at    | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                               |
| updated_at    | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                               |
| created_by    | BIGINT       | YES      |                   | 作成者ユーザーID                       |
| updated_by    | BIGINT       | YES      |                   | 更新者ユーザーID                       |

ユニーク制約: `(setting_key, service_name, environment)`

インデックス: `idx_fw_m_setting_key`, `idx_fw_m_setting_category`, `idx_fw_m_setting_service`, `idx_fw_m_setting_env`, `idx_fw_m_setting_valid`

### fw_h_setting（履歴テーブル）

| カラム名      | データ型     | NULL許可 | デフォルト値      | 説明                           |
| ------------- | ------------ | -------- | ----------------- | ------------------------------ |
| history_id    | BIGSERIAL    | NO       |                   | 履歴ID (PK)                    |
| setting_id    | BIGINT       | NO       |                   | 設定ID                         |
| setting_key   | VARCHAR(255) | NO       |                   | 設定キー                       |
| setting_value | TEXT         | NO       |                   | 設定値                         |
| value_type    | VARCHAR(50)  | NO       |                   | 値の型                         |
| service_name  | VARCHAR(100) | YES      |                   | サービス名                     |
| environment   | VARCHAR(50)  | YES      |                   | 環境                           |
| version       | INTEGER      | NO       |                   | バージョン                     |
| operation     | VARCHAR(20)  | NO       |                   | 操作種別 (INSERT, UPDATE, DELETE) |
| changed_at    | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 変更日時                       |
| changed_by    | BIGINT       | YES      |                   | 変更者ユーザーID               |

インデックス: `idx_fw_h_setting_id`, `idx_fw_h_setting_key`, `idx_fw_h_setting_changed`

---

## endpoint-service 所有テーブル

### fw_m_endpoint

| カラム名            | データ型      | NULL許可 | デフォルト値      | 説明                              |
| ------------------- | ------------- | -------- | ----------------- | --------------------------------- |
| endpoint_id         | BIGSERIAL     | NO       |                   | エンドポイントID (PK)             |
| service_name        | VARCHAR(255)  | NO       |                   | サービス名                        |
| path                | VARCHAR(1024) | NO       |                   | パス                              |
| method              | VARCHAR(20)   | NO       |                   | HTTPメソッド (GET, POST, etc.)    |
| protocol            | VARCHAR(20)   | NO       | http              | プロトコル (http, https, grpc)    |
| description         | TEXT          | YES      |                   | 説明                              |
| version             | VARCHAR(20)   | YES      | v1                | APIバージョン                     |
| is_public           | BOOLEAN       | NO       | FALSE             | 公開フラグ（認証不要）            |
| is_deprecated       | BOOLEAN       | NO       | FALSE             | 非推奨フラグ                      |
| deprecated_at       | TIMESTAMPTZ   | YES      |                   | 非推奨日時                        |
| deprecated_message  | TEXT          | YES      |                   | 非推奨メッセージ                  |
| rate_limit_per_minute | INTEGER     | YES      |                   | レート制限（/分、NULLは無制限）   |
| timeout_ms          | INTEGER       | YES      | 30000             | タイムアウト（ミリ秒）            |
| retry_count         | INTEGER       | YES      | 0                 | リトライ回数                      |
| metadata            | JSONB         | YES      |                   | 追加メタデータ（JSON）            |
| created_at          | TIMESTAMPTZ   | NO       | CURRENT_TIMESTAMP | 作成日時                          |
| updated_at          | TIMESTAMPTZ   | NO       | CURRENT_TIMESTAMP | 更新日時                          |
| created_by          | BIGINT        | YES      |                   | 作成者ユーザーID                  |
| updated_by          | BIGINT        | YES      |                   | 更新者ユーザーID                  |

ユニーク制約: `(service_name, path, method, version)`

インデックス: `idx_fw_m_endpoint_service`, `idx_fw_m_endpoint_path`, `idx_fw_m_endpoint_method`, `idx_fw_m_endpoint_protocol`, `idx_fw_m_endpoint_public`, `idx_fw_m_endpoint_deprecated`

### fw_m_service_address

| カラム名          | データ型     | NULL許可 | デフォルト値      | 説明                           |
| ----------------- | ------------ | -------- | ----------------- | ------------------------------ |
| address_id        | BIGSERIAL    | NO       |                   | アドレスID (PK)                |
| service_name      | VARCHAR(255) | NO       |                   | サービス名                     |
| protocol          | VARCHAR(20)  | NO       |                   | プロトコル                     |
| address           | VARCHAR(512) | NO       |                   | アドレス                       |
| use_tls           | BOOLEAN      | NO       | FALSE             | TLS使用フラグ                  |
| environment       | VARCHAR(50)  | YES      |                   | 環境（NULLは全環境共通）       |
| priority          | INTEGER      | NO       | 0                 | 優先度（高い値が優先）         |
| is_active         | BOOLEAN      | NO       | TRUE              | 有効フラグ                     |
| health_check_path | VARCHAR(255) | YES      |                   | ヘルスチェックパス             |
| created_at        | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                       |
| updated_at        | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 更新日時                       |
| created_by        | BIGINT       | YES      |                   | 作成者ユーザーID               |
| updated_by        | BIGINT       | YES      |                   | 更新者ユーザーID               |

ユニーク制約: `(service_name, protocol, environment, priority)`

インデックス: `idx_fw_m_service_address_service`, `idx_fw_m_service_address_protocol`, `idx_fw_m_service_address_env`, `idx_fw_m_service_address_active`

### fw_m_endpoint_permission

| カラム名       | データ型     | NULL許可 | デフォルト値      | 説明                      |
| -------------- | ------------ | -------- | ----------------- | ------------------------- |
| endpoint_id    | BIGINT       | NO       |                   | エンドポイントID (PK, FK) |
| permission_key | VARCHAR(255) | NO       |                   | パーミッションキー (PK)   |
| created_at     | TIMESTAMPTZ  | NO       | CURRENT_TIMESTAMP | 作成日時                  |
| created_by     | BIGINT       | YES      |                   | 作成者ユーザーID          |

インデックス: `idx_fw_m_endpoint_permission_endpoint`, `idx_fw_m_endpoint_permission_key`

---

## 共通トリガー関数

### fw_update_timestamp()
各テーブルの `updated_at` を自動更新するトリガー関数。

### fw_setting_history()
`fw_m_setting` の INSERT/UPDATE/DELETE を `fw_h_setting` に記録するトリガー関数。

---

