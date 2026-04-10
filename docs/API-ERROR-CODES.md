# API エラーコードカタログ

ADR-0005 に基づき、k1s0 の全サービスで使用するエラーコードの一覧と説明を定義する。

## エラーコード体系

```
{SCOPE}_{SERVICE}_{CATEGORY}
```

| フィールド | 説明 | 例 |
|-----------|------|-----|
| SCOPE | `SYS` = システム層, `SVC` = サービス層 | `SYS`, `SVC` |
| SERVICE | サービス略称 | `AUTH`, `TENANT`, `TASK` |
| CATEGORY | エラー種別 | `NOT_FOUND`, `INTERNAL_ERROR` |

## HTTP ステータスコードとの対応

| エラーカテゴリ | HTTP ステータス | 説明 |
|--------------|----------------|------|
| `UNAUTHORIZED` / `MISSING_TOKEN` / `INVALID_TOKEN` | 401 | 認証失敗 |
| `PERMISSION_DENIED` / `FORBIDDEN` / `TIER_FORBIDDEN` | 403 | 認可失敗 |
| `NOT_FOUND` | 404 | リソース未発見 |
| `CONFLICT` / `ALREADY_EXISTS` / `VERSION_CONFLICT` | 409 | 競合 |
| `VALIDATION_FAILED` / `VALIDATION_ERROR` / `INVALID_*` | 400 | 入力値エラー |
| `RATE_EXCEEDED` / `RATE_LIMIT_EXCEEDED` | 429 | レートリミット超過 |
| `INTERNAL_ERROR` | 500 | サーバー内部エラー |

---

## auth サービス (`SYS_AUTH_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_AUTH_INVALID_TOKEN` | 401 | JWT トークンが無効 |
| `SYS_AUTH_TOKEN_EXPIRED` | 401 | JWT トークンが期限切れ |
| `SYS_AUTH_TOKEN_INVALID` | 401 | JWT トークンの形式が不正 |
| `SYS_AUTH_TOKEN_INVALID_AUDIENCE` | 401 | JWT audience が不一致 |
| `SYS_AUTH_MISSING_TOKEN` | 401 | Authorization ヘッダーが未設定 |
| `SYS_AUTH_MISSING_CLAIMS` | 401 | JWT Claims が欠落 |
| `SYS_AUTH_JWKS_FETCH_FAILED` | 503 | JWKS エンドポイントへの接続失敗 |
| `SYS_AUTH_JWKS_ERROR` | 503 | JWKS 取得エラー |
| `SYS_AUTH_JWKS_UNAVAILABLE` | 503 | JWKS が利用不可 |
| `SYS_AUTH_PERMISSION_DENIED` | 403 | RBAC 権限なし |
| `SYS_AUTH_PERMISSION_VALIDATION` | 400 | 権限バリデーションエラー |
| `SYS_AUTH_FORBIDDEN` | 403 | アクセス禁止 |
| `SYS_AUTH_TIER_FORBIDDEN` | 403 | Tier アクセス禁止 |
| `SYS_AUTH_SERVICE_UNAVAILABLE` | 503 | 認証サービス利用不可 |
| `SYS_AUTH_NOT_FOUND` | 404 | リソース未発見 |
| `SYS_AUTH_INTERNAL_ERROR` | 500 | 内部エラー |
| `SYS_AUTH_AUDIT_VALIDATION` | 400 | 監査ログバリデーションエラー |
| `SYS_AUTH_API_KEY_NOT_FOUND` | 404 | API キー未発見 |
| `SYS_AUTH_API_KEY_VALIDATION` | 400 | API キーバリデーションエラー |
| `SYS_AUTH_PEPPER_NOT_CONFIGURED` | 500 | API キーペッパー未設定（サーバー設定エラー）|
| `SYS_AUTH_VALIDATION_FAILED` | 400 | 入力値バリデーションエラー |
| `SYS_AUTH_UNAUTHORIZED` | 401 | 未認証 |
| `SYS_AUTH_USER_NOT_FOUND` | 404 | ユーザー未発見（MED-017 監査対応: auth_handler.rs に実装済みだが未記載だった） |

---

## tenant サービス (`SYS_TENANT_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_TENANT_NOT_FOUND` | 404 | テナント未発見 |
| `SYS_TENANT_ALREADY_EXISTS` / `SYS_TENANT_NAME_CONFLICT` | 409 | テナント名重複 |
| `SYS_TENANT_MEMBER_NOT_FOUND` | 404 | メンバー未発見 |
| `SYS_TENANT_MEMBER_CONFLICT` | 409 | メンバー重複 |
| `SYS_TENANT_INVALID_INPUT` | 400 | 入力値エラー |
| `SYS_TENANT_INVALID_STATUS` | 400 | ステータス遷移エラー |
| `SYS_TENANT_PERMISSION_DENIED` | 403 | 権限なし |
| `SYS_TENANT_VALIDATION_ERROR` | 400 | バリデーションエラー |
| `SYS_TENANT_INTERNAL_ERROR` | 500 | 内部エラー |

---

## config サービス (`SYS_CONFIG_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_CONFIG_KEY_NOT_FOUND` | 404 | 設定キー未発見 |
| `SYS_CONFIG_SCHEMA_NOT_FOUND` | 404 | スキーマ未発見 |
| `SYS_CONFIG_SERVICE_NOT_FOUND` | 404 | サービス設定未発見 |
| `SYS_CONFIG_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_CONFIG_VERSION_CONFLICT` | 409 | バージョン競合 |
| `SYS_CONFIG_INTERNAL_ERROR` | 500 | 内部エラー |

---

## quota サービス (`SYS_QUOTA_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_QUOTA_NOT_FOUND` | 404 | クォータ未発見 |
| `SYS_QUOTA_ALREADY_EXISTS` | 409 | クォータ重複 |
| `SYS_QUOTA_EXCEEDED` | 429 | クォータ超過 |
| `SYS_QUOTA_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_QUOTA_INTERNAL_ERROR` | 500 | 内部エラー |

---

## ratelimit サービス (`SYS_RATELIMIT_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_RATELIMIT_RATE_EXCEEDED` | 429 | レートリミット超過 |
| `SYS_RATELIMIT_RULE_NOT_FOUND` | 404 | ルール未発見 |
| `SYS_RATELIMIT_RULE_EXISTS` | 409 | ルール重複 |
| `SYS_RATELIMIT_NOT_FOUND` | 404 | リソース未発見 |
| `SYS_RATELIMIT_VALIDATION_ERROR` / `SYS_RATELIMIT_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_RATELIMIT_INTERNAL_ERROR` / `SYS_RATELIMIT_ERROR` | 500 | 内部エラー |

---

## session サービス (`SYS_SESSION_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_SESSION_NOT_FOUND` | 404 | セッション未発見 |
| `SYS_SESSION_EXPIRED` | 401 | セッション期限切れ |
| `SYS_SESSION_ALREADY_REVOKED` | 409 | セッション既失効 |
| `SYS_SESSION_MAX_DEVICES_EXCEEDED` | 403 | デバイス数上限超過 |
| `SYS_SESSION_FORBIDDEN` | 403 | アクセス禁止 |
| `SYS_SESSION_UNAUTHORIZED` | 401 | 未認証 |
| `SYS_SESSION_VALIDATION_ERROR` | 400 | バリデーションエラー |
| `SYS_SESSION_INTERNAL_ERROR` | 500 | 内部エラー |

---

## vault サービス (`SYS_VAULT_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_VAULT_ACCESS_DENIED` | 403 | アクセス拒否 |
| `SYS_VAULT_ALREADY_EXISTS` | 409 | シークレット重複 |
| `SYS_VAULT_CACHE_ERROR` | 500 | キャッシュエラー |
| `SYS_VAULT_ENCRYPTION_ERROR` | 500 | 暗号化エラー |
| `SYS_VAULT_NOT_FOUND` | 404 | シークレット未発見（MED-017 監査対応） |
| `SYS_VAULT_VALIDATION_ERROR` | 400 | バリデーションエラー（vault_handler.rs）（MED-017 監査対応） |
| `SYS_VAULT_VALIDATION_FAILED` | 400 | バリデーション失敗（domain/error.rs）（MED-017 監査対応） |
| `SYS_VAULT_UPSTREAM_ERROR` | 502 | 上流 Vault サービスエラー（MED-017 監査対応） |
| `SYS_VAULT_INTERNAL_ERROR` | 500 | 内部エラー（MED-017 監査対応） |

---

## api-registry サービス (`SYS_APIREG_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_APIREG_NOT_FOUND` / `SYS_APIREG_VERSION_NOT_FOUND` / `SYS_APIREG_SCHEMA_NOT_FOUND` | 404 | リソース未発見 |
| `SYS_APIREG_ALREADY_EXISTS` / `SYS_APIREG_CONFLICT` | 409 | 重複・競合 |
| `SYS_APIREG_CANNOT_DELETE_LATEST` | 409 | 最新バージョン削除禁止 |
| `SYS_APIREG_SCHEMA_INVALID` | 400 | スキーマ形式エラー |
| `SYS_APIREG_VALIDATION_ERROR` / `SYS_APIREG_VALIDATOR_ERROR` | 400 | バリデーションエラー |
| `SYS_APIREG_UNAUTHORIZED` | 401 | 未認証 |
| `SYS_APIREG_INTERNAL_ERROR` | 500 | 内部エラー |

---

## app-registry サービス (`SYS_APPREG_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_APPREG_NOT_FOUND` | 404 | アプリ未発見 |
| `SYS_APPREG_ALREADY_EXISTS` | 409 | アプリ重複 |
| `SYS_APPREG_VERSION_CONFLICT` | 409 | バージョン競合 |
| `SYS_APPREG_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_APPREG_INTERNAL_ERROR` | 500 | 内部エラー |

---

## task サービス (`SVC_TASK_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SVC_TASK_NOT_FOUND` | 404 | タスク未発見 |
| `SVC_TASK_INVALID_STATUS_TRANSITION` | 400 | 無効なステータス遷移 |
| `SVC_TASK_VERSION_CONFLICT` | 409 | 楽観的ロック競合 |
| `SVC_TASK_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SVC_TASK_ERROR` / `SVC_TASK_INTERNAL_ERROR` | 500 | 内部エラー |
| `SVC_TASK_AUTH_MISSING_CLAIMS` | 401 | Claims 未設定 |
| `SVC_TASK_AUTH_PERMISSION_DENIED` | 403 | 権限なし |

---

## board サービス (`SVC_BOARD_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SVC_BOARD_COLUMN_NOT_FOUND` | 404 | ボードカラム未発見 |
| `SVC_BOARD_ERROR` | 500 | 内部エラー |
| `SVC_AUTH_MISSING_CLAIMS` | 401 | Claims 未設定 |
| `SVC_AUTH_PERMISSION_DENIED` | 403 | 権限なし |

---

## activity サービス (`SVC_ACTIVITY_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SVC_ACTIVITY_NOT_FOUND` | 404 | アクティビティ未発見 |
| `SVC_ACTIVITY_INVALID_STATUS` | 400 | 無効なステータス |
| `SVC_ACTIVITY_ERROR` | 500 | 内部エラー |

---

## file サービス (`SYS_FILE_*`)

<!-- MED-017 監査対応: file サービスのエラーコードを API カタログに追記 -->

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_FILE_VALIDATION` | 400 | 入力バリデーションエラー（ファイル名不正等） |
| `SYS_FILE_NOT_FOUND` | 404 | ファイル未発見 |
| `SYS_FILE_ALREADY_COMPLETED` | 409 | ファイルアップロード完了済み |
| `SYS_FILE_NOT_AVAILABLE` | 404 | ファイルがまだ利用可能でない |
| `SYS_FILE_ACCESS_DENIED` | 403 | アクセス権限なし |
| `SYS_FILE_SIZE_EXCEEDED` | 413 | ファイルサイズ超過 |
| `SYS_FILE_STORAGE_ERROR` | 502 | ストレージバックエンドエラー |
| `SYS_FILE_UPLOAD_FAILED` | 500 | アップロード開始失敗 |
| `SYS_FILE_GET_FAILED` | 500 | ファイルメタデータ取得失敗 |
| `SYS_FILE_LIST_FAILED` | 500 | ファイル一覧取得失敗 |
| `SYS_FILE_DELETE_FAILED` | 500 | ファイル削除失敗 |
| `SYS_FILE_COMPLETE_FAILED` | 500 | アップロード完了処理失敗 |
| `SYS_FILE_DOWNLOAD_URL_FAILED` | 500 | ダウンロードURL生成失敗 |
| `SYS_FILE_TAGS_UPDATE_FAILED` | 500 | タグ更新失敗 |

---

---

## dlq-manager サービス (`SYS_DLQ_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_DLQ_NOT_FOUND` | 404 | DLQ メッセージ未発見 |
| `SYS_DLQ_CONFLICT` | 409 | メッセージ処理済み（AlreadyProcessed） |
| `SYS_DLQ_VALIDATION_ERROR` | 400 | バリデーションエラー |
| `SYS_DLQ_PROCESS_FAILED` | 500 | メッセージ再処理失敗 |
| `SYS_DLQ_INTERNAL_ERROR` | 500 | 内部エラー |

---

## featureflag サービス (`SYS_FF_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_FF_NOT_FOUND` | 404 | フィーチャーフラグ未発見 |
| `SYS_FF_ALREADY_EXISTS` | 409 | フィーチャーフラグ重複 |
| `SYS_FF_EVALUATE_FAILED` | 500 | フラグ評価失敗 |
| `SYS_FF_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_FF_INTERNAL_ERROR` | 500 | 内部エラー |

---

## event-monitor サービス (`SYS_EVMON_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_EVMON_NOT_FOUND` | 404 | イベント未発見 |
| `SYS_EVMON_ALERT_RULE_NOT_FOUND` | 404 | アラートルール未発見 |
| `SYS_EVMON_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_EVMON_INTERNAL_ERROR` | 500 | 内部エラー |

---

## event-store サービス (`SYS_EVSTORE_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_EVSTORE_STREAM_NOT_FOUND` | 404 | ストリーム未発見 |
| `SYS_EVSTORE_EVENT_NOT_FOUND` | 404 | イベント未発見 |
| `SYS_EVSTORE_SNAPSHOT_NOT_FOUND` | 404 | スナップショット未発見 |
| `SYS_EVSTORE_STREAM_ALREADY_EXISTS` | 409 | ストリーム重複 |
| `SYS_EVSTORE_VERSION_CONFLICT` | 409 | バージョン競合（楽観的ロック） |
| `SYS_EVSTORE_VALIDATION_FAILED` | 400 | バリデーションエラー（domain/error.rs） |
| `SYS_EVSTORE_VALIDATION_ERROR` | 400 | バリデーションエラー（adapter/handler/error.rs） |
| `SYS_EVSTORE_INTERNAL_ERROR` | 500 | 内部エラー |

---

## graphql-gateway サービス (`SYS_GQLGW_*`)

GraphQL レスポンスのエラーは `errors[].extensions.code` フィールドに設定される。

| コード | HTTP / GraphQL | 説明 |
|--------|----------------|------|
| `SYS_GQLGW_NOT_FOUND` | 404 | スキーマ未発見 |
| `SYS_GQLGW_QUERY_PARSE_FAILED` | 400 | クエリ解析失敗 |
| `SYS_GQLGW_UPSTREAM_ERROR` | 503 | 上流サービスへのリクエスト失敗 |
| `SYS_GQLGW_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_GQLGW_INTERNAL_ERROR` | 500 | 内部エラー |
| `UNAUTHENTICATED` | GraphQL拡張 | gRPC Unauthenticated に対応（GraphQL extensions.code） |
| `FORBIDDEN` | GraphQL拡張 | gRPC PermissionDenied に対応（GraphQL extensions.code） |
| `VALIDATION_ERROR` | GraphQL拡張 | gRPC InvalidArgument/FailedPrecondition/OutOfRange に対応 |
| `BACKEND_ERROR` | GraphQL拡張 | gRPC その他エラーに対応 |

---

## master-maintenance サービス (`SYS_MM_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_MM_TABLE_NOT_FOUND` | 404 | テーブル定義未発見 |
| `SYS_MM_RECORD_NOT_FOUND` | 404 | レコード未発見 |
| `SYS_MM_RULE_NOT_FOUND` | 404 | 整合性ルール未発見 |
| `SYS_MM_DISPLAY_CONFIG_NOT_FOUND` | 404 | 表示設定未発見 |
| `SYS_MM_IMPORT_JOB_NOT_FOUND` | 404 | インポートジョブ未発見 |
| `SYS_MM_RELATIONSHIP_NOT_FOUND` | 404 | テーブルリレーションシップ未発見 |
| `SYS_MM_COLUMN_NOT_FOUND` | 404 | カラム定義未発見 |
| `SYS_MM_OPERATION_NOT_ALLOWED` | 403 | テーブルへの操作権限なし（Create/Update/Delete 禁止） |
| `SYS_MM_DUPLICATE_TABLE` | 409 | テーブル名重複 |
| `SYS_MM_DUPLICATE_COLUMN` | 409 | カラム名重複 |
| `SYS_MM_VERSION_CONFLICT` | 409 | バージョン競合（楽観的ロック） |
| `SYS_MM_INVALID_RULE` | 400 | 整合性ルール無効 |
| `SYS_MM_IMPORT_FAILED` | 400 | インポート処理失敗 |
| `SYS_MM_VALIDATION_ERROR` | 400 | バリデーションエラー（ValidationFailed / RecordValidation 両方） |
| `SYS_MM_INTERNAL_ERROR` | 500 | 内部エラー（SqlBuildError / Internal 両方。詳細はサーバーログのみ） |

---

## navigation サービス (`SYS_NAV_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_NAV_NOT_FOUND` | 404 | ナビゲーション項目未発見 |
| `SYS_NAV_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_NAV_INTERNAL_ERROR` | 500 | 内部エラー |

---

## notification サービス (`SYS_NOTIFY_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_NOTIFY_NOT_FOUND` | 404 | 通知未発見 |
| `SYS_NOTIFY_CHANNEL_NOT_FOUND` | 404 | 通知チャンネル未発見 |
| `SYS_NOTIFY_TEMPLATE_NOT_FOUND` | 404 | 通知テンプレート未発見 |
| `SYS_NOTIFY_ALREADY_SENT` | 409 | 通知送信済み |
| `SYS_NOTIFY_CHANNEL_DISABLED` | 400 | 通知チャンネル無効化 |
| `SYS_NOTIFY_SEND_FAILED` | 500 | 通知送信失敗 |
| `SYS_NOTIFY_VALIDATION_ERROR` | 400 | バリデーションエラー |
| `SYS_NOTIFY_INTERNAL_ERROR` | 500 | 内部エラー |

---

## policy サービス (`SYS_POLICY_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_POLICY_NOT_FOUND` | 404 | ポリシー未発見 |
| `SYS_POLICY_ALREADY_EXISTS` | 409 | ポリシー重複 |
| `SYS_POLICY_EVALUATION_FAILED` | 500 | ポリシー評価失敗 |
| `SYS_POLICY_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_POLICY_INTERNAL_ERROR` | 500 | 内部エラー |

---

## rule-engine サービス (`SYS_RULE_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_RULE_NOT_FOUND` | 404 | ルール未発見 |
| `SYS_RULE_ALREADY_EXISTS` | 409 | ルール重複 |
| `SYS_RULE_EVALUATION_FAILED` | 500 | ルール評価失敗 |
| `SYS_RULE_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_RULE_INTERNAL_ERROR` | 500 | 内部エラー |

---

## saga サービス (`SYS_SAGA_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_SAGA_NOT_FOUND` | 404 | サガ未発見 |
| `SYS_SAGA_INVALID_STATUS_TRANSITION` | 400 | 無効なサガ状態遷移（domain/error.rs） |
| `SYS_SAGA_VALIDATION_ERROR` | 400 | バリデーションエラー（adapter/handler/error.rs） |
| `SYS_SAGA_VALIDATION_FAILED` | 400 | バリデーションエラー（domain/error.rs） |
| `SYS_SAGA_CONFLICT` | 409 | 競合（adapter/handler/error.rs） |
| `SYS_SAGA_COMPENSATION_FAILED` | 500 | 補償処理失敗 |
| `SYS_SAGA_INTERNAL_ERROR` | 500 | 内部エラー |

---

## scheduler サービス (`SYS_SCHED_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_SCHED_NOT_FOUND` | 404 | ジョブ未発見 |
| `SYS_SCHED_ALREADY_EXISTS` | 409 | ジョブ重複 |
| `SYS_SCHED_INVALID_SCHEDULE` | 400 | 無効なスケジュール式 |
| `SYS_SCHED_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_SCHED_INTERNAL_ERROR` | 500 | 内部エラー |

---

## search サービス (`SYS_SEARCH_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_SEARCH_NOT_FOUND` | 404 | 検索インデックス未発見 |
| `SYS_SEARCH_INVALID_QUERY` | 400 | クエリ構文無効 |
| `SYS_SEARCH_INDEXING_FAILED` | 500 | インデックス作成失敗 |
| `SYS_SEARCH_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_SEARCH_INTERNAL_ERROR` | 500 | 内部エラー |

---

## service-catalog サービス (`SYS_SVCCAT_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_SVCCAT_NOT_FOUND` | 404 | サービス未発見 |
| `SYS_SVCCAT_ALREADY_EXISTS` | 409 | サービス重複 |
| `SYS_SVCCAT_VERSION_CONFLICT` | 409 | バージョン競合 |
| `SYS_SVCCAT_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_SVCCAT_INTERNAL_ERROR` | 500 | 内部エラー |

---

## workflow サービス (`SYS_WORKFLOW_*`)

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_WORKFLOW_NOT_FOUND` | 404 | ワークフロー未発見 |
| `SYS_WORKFLOW_ALREADY_EXISTS` | 409 | ワークフロー重複 |
| `SYS_WORKFLOW_INVALID_STATUS_TRANSITION` | 400 | 無効なワークフロー状態遷移 |
| `SYS_WORKFLOW_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_WORKFLOW_INTERNAL_ERROR` | 500 | 内部エラー |

---

## ai-agent サービス (`SYS_AIAGENT_*`) — experimental

> このサービスは experimental ステータスです。エラーコードは今後変更される可能性があります。

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_AIAGENT_NOT_FOUND` | 404 | エージェント未発見 |
| `SYS_AIAGENT_ALREADY_EXISTS` | 409 | エージェント重複 |
| `SYS_AIAGENT_EXECUTION_FAILED` | 500 | エージェント実行失敗 |
| `SYS_AIAGENT_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_AIAGENT_INTERNAL_ERROR` | 500 | 内部エラー |

---

## ai-gateway サービス (`SYS_AIGW_*`) — experimental

> このサービスは experimental ステータスです。エラーコードは今後変更される可能性があります。

| コード | HTTP | 説明 |
|--------|------|------|
| `SYS_AIGW_NOT_FOUND` | 404 | モデル未発見 |
| `SYS_AIGW_MODEL_REQUEST_FAILED` | 503 | モデルへのリクエスト失敗 |
| `SYS_AIGW_RATE_LIMIT_EXCEEDED` | 429 | レートリミット超過 |
| `SYS_AIGW_VALIDATION_FAILED` | 400 | バリデーションエラー |
| `SYS_AIGW_INTERNAL_ERROR` | 500 | 内部エラー |

---

## エラーレスポンスフォーマット

ADR-0005 に準拠した JSON エラーレスポンス形式:

```json
{
  "code": "SYS_AUTH_INVALID_TOKEN",
  "message": "The provided JWT token is invalid or has expired"
}
```

- `code`: 上記カタログのエラーコード（英大文字スネークケース）
- `message`: 英語のエラーメッセージ（クライアント開発者向け）

## 参照

- [ADR-0005: エラーレスポンス設計](architecture/adr/0005-error-response-design.md)
- [認証認可設計](architecture/overview/認証認可設計.md)
