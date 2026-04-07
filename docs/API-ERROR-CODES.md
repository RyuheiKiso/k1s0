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
