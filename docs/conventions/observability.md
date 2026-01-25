# 観測性（Observability）規約

本ドキュメントは、k1s0 における観測性（ログ/トレース/メトリクス）の出力規約を定義する。

## 1. 基本方針

- 全サービスで **統一された出力形式** を使用する
- **OTel（OpenTelemetry）** を標準とする
- ログ/トレース/メトリクスは **相関可能** にする（trace_id で紐付け）

## 2. ログ

### 2.1 形式

- **JSON**（1 行 1 イベント）
- 人間向けの整形出力は開発時のみ許可

### 2.2 必須フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `timestamp` | string (ISO 8601) | イベント発生時刻 |
| `level` | string | ログレベル（DEBUG/INFO/WARN/ERROR） |
| `service_name` | string | サービス名 |
| `env` | string | 環境（dev/stg/prod） |
| `trace_id` | string | トレース ID（分散トレースとの相関用） |
| `span_id` | string | スパン ID |
| `message` | string | ログメッセージ |

### 2.3 推奨フィールド

| フィールド | 説明 |
|-----------|------|
| `request_id` | リクエスト ID（採用する場合） |
| `grpc.method` | gRPC メソッド名 |
| `http.route` | HTTP ルート |
| `error.kind` | エラー分類 |
| `error.code` | エラーコード |

### 2.4 ログ出力例

```json
{
  "timestamp": "2026-01-25T10:30:00.123Z",
  "level": "INFO",
  "service_name": "user-service",
  "env": "dev",
  "trace_id": "abc123def456",
  "span_id": "span789",
  "message": "User created successfully",
  "user_id": 12345
}
```

## 3. トレース

### 3.1 伝播方式

- **W3C Trace Context** を使用
- HTTP ヘッダ: `traceparent`, `tracestate`
- gRPC メタデータ: 同上

### 3.2 必須属性

| 属性 | 説明 |
|------|------|
| `service.name` | サービス名 |
| `deployment.environment` | 環境（dev/stg/prod） |
| `rpc.system` / `rpc.method` | gRPC の場合 |
| `http.method` / `url.path` | HTTP の場合 |

### 3.3 スパン作成の原則

以下の境界で必ずスパンを切る：

- HTTP/gRPC リクエストの受信
- 外部 DB への呼び出し
- 外部 Redis/Cache への呼び出し
- 他サービスへの gRPC/HTTP 呼び出し

## 4. メトリクス

### 4.1 エクスポート

- OTel Metrics（Collector 経由）

### 4.2 命名規則

```
k1s0.{service}.{component}.{metric}
```

例：
- `k1s0.user_service.http.request_count`
- `k1s0.user_service.db.query_duration_ms`

### 4.3 必須メトリクス

| メトリクス | 説明 | ラベル |
|-----------|------|--------|
| `request_count` | リクエスト数 | protocol, route/method, status_code |
| `request_failures` | 失敗リクエスト数 | protocol, route/method, status_code, error_code |
| `request_duration_ms` | レイテンシ（histogram） | protocol, route/method |
| `dependency_failures` | 外部依存の失敗数 | dependency, error_kind |
| `dependency_duration_ms` | 外部依存のレイテンシ | dependency |
| `config_fetch_failures` | 設定取得の失敗数 | source (yaml/db) |

## 5. エラー出力

### 5.1 エラーログの必須フィールド

| フィールド | 説明 |
|-----------|------|
| `error.kind` | 分類（validation/authz/conflict/dependency/internal） |
| `error.code` | エラーコード（`{service}.{category}.{reason}`） |
| `error.message` | 要約（機密情報を含めない） |
| `http.status_code` / `grpc.status_code` | ステータスコード |

### 5.2 エラーログ出力例

```json
{
  "timestamp": "2026-01-25T10:30:00.123Z",
  "level": "ERROR",
  "service_name": "user-service",
  "env": "dev",
  "trace_id": "abc123def456",
  "span_id": "span789",
  "message": "Failed to create user",
  "error.kind": "conflict",
  "error.code": "user.already_exists",
  "error.message": "User with this email already exists",
  "http.status_code": 409
}
```

## 6. ヘルスチェック

### 6.1 HTTP を提供する場合

```
GET /healthz
```

- 成功: `200 OK`
- 失敗: `503 Service Unavailable`

### 6.2 gRPC のみの場合

```
grpc.health.v1.Health/Check
```

- 成功: `SERVING`
- 失敗: `NOT_SERVING`

## 7. 疎通チェック（dev-check）の合格条件

`scripts/dev-check.ps1` は以下を検証する：

1. **プロセス/ヘルス**
   - HTTP: `/healthz` が 200
   - gRPC: `Health/Check` が SERVING

2. **依存疎通**
   - DB: `SELECT 1` 等が成功
   - Redis: ping が成功（利用する場合）
   - config-service: 設定取得が成功（利用する場合）

3. **観測**
   - 起動後に少なくとも 1 件のトレース/メトリクスが Collector に到達

## 関連ドキュメント

- [エラー規約](error-handling.md)
- [構想.md](../../work/構想.md): 全体方針（12. 観測性）
