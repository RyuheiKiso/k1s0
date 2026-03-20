# system-rule-engine-server 設計

> **認可モデル注記**: 実装では `resource/action`（例: `rules/read`, `rules/write`, `rules/admin`）で判定し、ロール `sys_admin` / `sys_operator` / `sys_auditor` は middleware でそれぞれ `admin` / `write` / `read` にマッピングされます。


業務ルール（税率計算、与信判定、価格算出等）を外部化して管理・評価するルールエンジンサーバー。JSON/YAML ベースのルール定義をホットリロードで反映し、コードデプロイなしでルール変更を可能にする。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | rules/read |
| sys_operator 以上 | rules/write |
| sys_admin のみ | rules/admin |


system tier の業務ルールエンジンサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ルール定義管理 | JSON/YAML 形式の業務ルール（条件式・アクション・優先度）の CRUD |
| ルールセット管理 | 複数ルールをグループ化したルールセットの管理・バージョン管理 |
| ルール評価 | 入力データに対するルール評価（条件マッチ → アクション実行 → 結果返却） |
| ホットリロード | config-server の動的更新通知を受信し、サービス再起動なしでルール定義を反映 |
| 評価監査ログ | 全評価結果を audit-client 経由で記録（「なぜこの判定になったか」のトレーサビリティ） |
| A/B テスト連携 | featureflag-server と連携し、ルールセットの段階的適用（新旧ルール並行評価）を実現 |
| ルール変更通知 | ルール変更時に Kafka `k1s0.system.rule_engine.rule_changed.v1` で通知 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| ルール評価エンジン | 自前実装（条件式 AST → インタプリタ評価） |
| キャッシュ | moka v0.12（評価結果・ルール定義キャッシュ） |

### 配置パス

配置: `regions/system/server/rust/rule-engine/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

### gRPC ポート

proto ファイルおよびサーバー実装のデフォルト: **50051**（config.yaml で上書き可能）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| policy-server との違い | policy-server は Rego ベースの「アクセス制御ポリシー」（allow/deny）に特化する。rule-engine は「業務判定ロジック」（税率計算、与信スコア、価格ティア判定等）に特化し、任意の構造化データを返却する |
| ルール定義形式 | JSON/YAML で条件式（`when`）とアクション（`then`）を記述。条件式は比較演算子・論理演算子・関数呼び出しをサポート |
| 評価エンジン | 条件式を AST にパースし、インタプリタで評価する自前実装。OPA のような外部プロセスを使用しないため、レイテンシが低い |
| ルールセット | 複数ルールをグループ化し、優先度順に評価。first-match（最初にマッチしたルールで停止）と all-match（全マッチルール適用）の 2 モードをサポート |
| バージョン管理 | ルールセット単位でバージョンを管理。ロールバック可能。featureflag 連携で新旧バージョンの A/B テスト |
| ホットリロード | config-server の Kafka 変更通知を購読し、ルール定義の再読み込みをトリガー。moka キャッシュを即座に無効化 |
| 監査ログ | 全評価結果を audit-client 経由で非同期送信。入力・適用ルール・出力・判定理由を記録 |
| キャッシュ | moka で評価結果を TTL 60 秒キャッシュ。ルール変更通知受信時に即座に無効化 |
| DB スキーマ | PostgreSQL の `rule_engine` スキーマ（rules, rule_sets, rule_set_versions, evaluation_logs テーブル） |
| Kafka | ルール変更時に `k1s0.system.rule_engine.rule_changed.v1` トピックへ変更通知を送信 |
| ポート | 8111（REST）/ 50051（gRPC） |

---

## ルール定義形式

### ルール（Rule）

```json
{
  "name": "consumption_tax_rate",
  "description": "消費税率の判定ルール",
  "priority": 10,
  "when": {
    "all": [
      { "field": "item.category", "operator": "eq", "value": "food" },
      { "field": "item.is_takeout", "operator": "eq", "value": true }
    ]
  },
  "then": {
    "tax_rate": 0.08,
    "tax_label": "軽減税率"
  }
}
```

### 条件式（when）

| 演算子 | 説明 | 例 |
| --- | --- | --- |
| `eq` | 等値比較 | `{ "field": "status", "operator": "eq", "value": "active" }` |
| `ne` | 非等値比較 | `{ "field": "status", "operator": "ne", "value": "deleted" }` |
| `gt` / `gte` | より大きい / 以上 | `{ "field": "amount", "operator": "gte", "value": 10000 }` |
| `lt` / `lte` | より小さい / 以下 | `{ "field": "score", "operator": "lt", "value": 50 }` |
| `in` | 含まれる | `{ "field": "region", "operator": "in", "value": ["JP", "US"] }` |
| `not_in` | 含まれない | `{ "field": "status", "operator": "not_in", "value": ["deleted", "suspended"] }` |
| `contains` | 文字列含有 | `{ "field": "name", "operator": "contains", "value": "特別" }` |
| `regex` | 正規表現マッチ | `{ "field": "code", "operator": "regex", "value": "^TAX-\\d{4}$" }` |

### 論理結合

| 結合子 | 説明 |
| --- | --- |
| `all` | 全条件が true（AND） |
| `any` | いずれかが true（OR） |
| `none` | 全条件が false（NOT AND） |

### ルールセット（RuleSet）

```json
{
  "name": "tax_calculation",
  "description": "税率計算ルールセット",
  "domain": "accounting",
  "evaluation_mode": "first_match",
  "default_result": { "tax_rate": 0.10, "tax_label": "標準税率" },
  "rules": ["rule-uuid-1", "rule-uuid-2", "rule-uuid-3"]
}
```

| フィールド | 説明 |
| --- | --- |
| `domain` | 業務領域（例: `accounting`, `fa`）。ルールセットの名前空間として機能 |
| `evaluation_mode` | `first_match`（最初のマッチで停止）/ `all_match`（全マッチを配列で返却） |
| `default_result` | どのルールにもマッチしなかった場合のデフォルト結果 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_RULE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/rules` | ルール一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/rules/{id}` | ルール詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/rules` | ルール作成 | `sys_operator` 以上 |
| PUT | `/api/v1/rules/{id}` | ルール更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/rules/{id}` | ルール削除 | `sys_admin` のみ |
| GET | `/api/v1/rule-sets` | ルールセット一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/rule-sets/{id}` | ルールセット詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/rule-sets` | ルールセット作成 | `sys_operator` 以上 |
| PUT | `/api/v1/rule-sets/{id}` | ルールセット更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/rule-sets/{id}` | ルールセット削除 | `sys_admin` のみ |
| POST | `/api/v1/rule-sets/{id}/publish` | ルールセットバージョン公開 | `sys_admin` のみ |
| POST | `/api/v1/rule-sets/{id}/rollback` | 前バージョンへロールバック | `sys_admin` のみ |
| POST | `/api/v1/evaluate` | ルール評価（ルールセット名 + 入力データ） | `sys_operator` 以上 |
| POST | `/api/v1/evaluate/dry-run` | ドライラン評価（結果返却のみ、監査ログ記録なし） | `sys_operator` 以上 |
| GET | `/api/v1/evaluation-logs` | 評価ログ一覧取得 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/rules

登録済みルールの一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |
| `rule_set_id` | string | No | - | ルールセット ID でフィルタ |
| `domain` | string | No | - | ドメインでフィルタ |

**レスポンス例（200 OK）**

```json
{
  "rules": [
    {
      "id": "rule-001",
      "name": "consumption_tax_rate",
      "description": "消費税率の判定ルール",
      "priority": 10,
      "when": {
        "all": [
          { "field": "item.category", "operator": "eq", "value": "food" },
          { "field": "item.is_takeout", "operator": "eq", "value": true }
        ]
      },
      "then": { "tax_rate": 0.08, "tax_label": "軽減税率" },
      "enabled": true,
      "version": 2,
      "created_at": "2026-03-01T10:00:00.000+00:00",
      "updated_at": "2026-03-05T14:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 35,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### POST /api/v1/rules

新しいルールを作成する。作成時にルール定義の構文バリデーションを行う。

**リクエスト例**

```json
{
  "name": "consumption_tax_rate",
  "description": "消費税率の判定ルール",
  "priority": 10,
  "when": {
    "all": [
      { "field": "item.category", "operator": "eq", "value": "food" },
      { "field": "item.is_takeout", "operator": "eq", "value": true }
    ]
  },
  "then": { "tax_rate": 0.08, "tax_label": "軽減税率" }
}
```

**レスポンス例（201 Created）**

```json
{
  "id": "rule-001",
  "name": "consumption_tax_rate",
  "description": "消費税率の判定ルール",
  "priority": 10,
  "when": {
    "all": [
      { "field": "item.category", "operator": "eq", "value": "food" },
      { "field": "item.is_takeout", "operator": "eq", "value": true }
    ]
  },
  "then": { "tax_rate": 0.08, "tax_label": "軽減税率" },
  "enabled": true,
  "version": 1,
  "created_at": "2026-03-01T10:00:00.000+00:00",
  "updated_at": "2026-03-01T10:00:00.000+00:00"
}
```

**レスポンス例（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_RULE_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      { "field": "when.all[0].operator", "message": "unknown operator: 'between'" },
      { "field": "priority", "message": "priority must be between 1 and 1000" }
    ]
  }
}
```

#### POST /api/v1/rule-sets

ルールセットを作成する。

**リクエスト例**

```json
{
  "name": "tax_calculation",
  "description": "税率計算ルールセット",
  "domain": "accounting",
  "evaluation_mode": "first_match",
  "default_result": { "tax_rate": 0.10, "tax_label": "標準税率" },
  "rule_ids": ["rule-001", "rule-002", "rule-003"]
}
```

**レスポンス例（201 Created）**

```json
{
  "id": "rs-001",
  "name": "tax_calculation",
  "description": "税率計算ルールセット",
  "domain": "accounting",
  "evaluation_mode": "first_match",
  "default_result": { "tax_rate": 0.10, "tax_label": "標準税率" },
  "rule_ids": ["rule-001", "rule-002", "rule-003"],
  "current_version": 1,
  "enabled": true,
  "created_at": "2026-03-01T10:00:00.000+00:00",
  "updated_at": "2026-03-01T10:00:00.000+00:00"
}
```

#### POST /api/v1/rule-sets/{id}/publish

ルールセットの現在の状態を新しいバージョンとして公開する。公開後、ルール変更通知を Kafka に送信する。

**レスポンス例（200 OK）**

```json
{
  "id": "rs-001",
  "name": "tax_calculation",
  "published_version": 3,
  "previous_version": 2,
  "published_at": "2026-03-05T15:00:00.000+00:00"
}
```

#### POST /api/v1/rule-sets/{id}/rollback

前のバージョンにロールバックする。

**レスポンス例（200 OK）**

```json
{
  "id": "rs-001",
  "name": "tax_calculation",
  "rolled_back_to_version": 2,
  "previous_version": 3,
  "rolled_back_at": "2026-03-05T16:00:00.000+00:00"
}
```

**レスポンス例（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_RULE_NO_PREVIOUS_VERSION",
    "message": "no previous version to rollback: current version is 1",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/evaluate

ルールセット名（またはID）と入力データを指定してルール評価を実行する。評価結果は監査ログに記録される。

**リクエスト例**

```json
{
  "rule_set": "accounting.tax_calculation",
  "input": {
    "item": {
      "category": "food",
      "is_takeout": true,
      "price": 1000
    }
  },
  "context": {
    "user_id": "user-123",
    "transaction_id": "tx-456"
  }
}
```

> `rule_set` は `{domain}.{name}` 形式で指定する。`context` は監査ログに記録される補足情報。

**レスポンス例（200 OK -- first_match）**

```json
{
  "evaluation_id": "eval_01JABCDEFG1234567890",
  "rule_set": "accounting.tax_calculation",
  "rule_set_version": 3,
  "matched_rule": {
    "id": "rule-001",
    "name": "consumption_tax_rate",
    "priority": 10
  },
  "result": {
    "tax_rate": 0.08,
    "tax_label": "軽減税率"
  },
  "cached": false,
  "evaluated_at": "2026-03-05T15:30:00.000+00:00"
}
```

**レスポンス例（200 OK -- all_match）**

```json
{
  "evaluation_id": "eval_01JABCDEFG1234567891",
  "rule_set": "fa.asset_classification",
  "rule_set_version": 2,
  "matched_rules": [
    {
      "id": "rule-010",
      "name": "tangible_asset",
      "priority": 10,
      "result": { "asset_type": "tangible", "depreciation_method": "straight_line" }
    },
    {
      "id": "rule-011",
      "name": "high_value_asset",
      "priority": 20,
      "result": { "requires_approval": true, "approval_level": "director" }
    }
  ],
  "cached": false,
  "evaluated_at": "2026-03-05T15:30:00.000+00:00"
}
```

**レスポンス例（200 OK -- マッチなし、デフォルト結果）**

```json
{
  "evaluation_id": "eval_01JABCDEFG1234567892",
  "rule_set": "accounting.tax_calculation",
  "rule_set_version": 3,
  "matched_rule": null,
  "result": { "tax_rate": 0.10, "tax_label": "標準税率" },
  "default_applied": true,
  "cached": false,
  "evaluated_at": "2026-03-05T15:30:00.000+00:00"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RULE_SET_NOT_FOUND",
    "message": "rule set not found: accounting.tax_calculation",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/evaluate/dry-run

評価を実行するが、監査ログへの記録をスキップする。ルール作成時のテストやデバッグ用途。リクエスト・レスポンス形式は `/api/v1/evaluate` と同一。

#### GET /api/v1/evaluation-logs

評価ログの一覧を取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |
| `rule_set` | string | No | - | ルールセット名でフィルタ |
| `domain` | string | No | - | ドメインでフィルタ |
| `from` | string | No | - | 開始日時（ISO 8601） |
| `to` | string | No | - | 終了日時（ISO 8601） |

**レスポンス例（200 OK）**

```json
{
  "logs": [
    {
      "evaluation_id": "eval_01JABCDEFG1234567890",
      "rule_set": "accounting.tax_calculation",
      "rule_set_version": 3,
      "matched_rule_id": "rule-001",
      "input_hash": "sha256:abc123...",
      "result": { "tax_rate": 0.08, "tax_label": "軽減税率" },
      "context": { "user_id": "user-123", "transaction_id": "tx-456" },
      "evaluated_at": "2026-03-05T15:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 1250,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_RULE_NOT_FOUND` | 404 | 指定されたルールが見つからない |
| `SYS_RULE_SET_NOT_FOUND` | 404 | 指定されたルールセットが見つからない |
| `SYS_RULE_ALREADY_EXISTS` | 409 | 同一名のルールが既に存在する |
| `SYS_RULE_VALIDATION_ERROR` | 400 | リクエストまたはルール定義のバリデーションエラー |
| `SYS_RULE_INVALID_CONDITION` | 400 | 条件式の構文エラー（不明な演算子、型不一致等） |
| `SYS_RULE_NO_PREVIOUS_VERSION` | 409 | ロールバック先の前バージョンが存在しない |
| `SYS_RULE_EVALUATION_ERROR` | 500 | ルール評価中の内部エラー |
| `SYS_RULE_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

gRPC ポート: **50051**

```protobuf
syntax = "proto3";
package k1s0.system.rule_engine.v1;

import "k1s0/system/common/v1/types.proto";

service RuleEngineService {
  rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
  rpc GetRule(GetRuleRequest) returns (GetRuleResponse);
  rpc CreateRule(CreateRuleRequest) returns (CreateRuleResponse);
  rpc UpdateRule(UpdateRuleRequest) returns (UpdateRuleResponse);
  rpc DeleteRule(DeleteRuleRequest) returns (DeleteRuleResponse);
  rpc ListRuleSets(ListRuleSetsRequest) returns (ListRuleSetsResponse);
  rpc GetRuleSet(GetRuleSetRequest) returns (GetRuleSetResponse);
  rpc CreateRuleSet(CreateRuleSetRequest) returns (CreateRuleSetResponse);
  rpc UpdateRuleSet(UpdateRuleSetRequest) returns (UpdateRuleSetResponse);
  rpc DeleteRuleSet(DeleteRuleSetRequest) returns (DeleteRuleSetResponse);
  rpc PublishRuleSet(PublishRuleSetRequest) returns (PublishRuleSetResponse);
  rpc RollbackRuleSet(RollbackRuleSetRequest) returns (RollbackRuleSetResponse);
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  rpc EvaluateDryRun(EvaluateRequest) returns (EvaluateResponse);
}

// --- Rule messages ---

message Rule {
  string id = 1;
  string name = 2;
  string description = 3;
  int32 priority = 4;
  bytes when_json = 5;
  bytes then_json = 6;
  bool enabled = 7;
  uint32 version = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}

message ListRulesRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  optional string rule_set_id = 2;
  optional string domain = 3;
}

message ListRulesResponse {
  repeated Rule rules = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message GetRuleRequest {
  string id = 1;
}

message GetRuleResponse {
  Rule rule = 1;
}

message CreateRuleRequest {
  string name = 1;
  string description = 2;
  int32 priority = 3;
  bytes when_json = 4;
  bytes then_json = 5;
}

message CreateRuleResponse {
  Rule rule = 1;
}

message UpdateRuleRequest {
  string id = 1;
  optional string description = 2;
  optional int32 priority = 3;
  optional bytes when_json = 4;
  optional bytes then_json = 5;
  optional bool enabled = 6;
}

message UpdateRuleResponse {
  Rule rule = 1;
}

message DeleteRuleRequest {
  string id = 1;
}

message DeleteRuleResponse {
  bool success = 1;
  string message = 2;
}

// --- RuleSet messages ---

message RuleSet {
  string id = 1;
  string name = 2;
  string description = 3;
  string domain = 4;
  string evaluation_mode = 5;
  bytes default_result_json = 6;
  repeated string rule_ids = 7;
  uint32 current_version = 8;
  bool enabled = 9;
  k1s0.system.common.v1.Timestamp created_at = 10;
  k1s0.system.common.v1.Timestamp updated_at = 11;
}

message ListRuleSetsRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  optional string domain = 2;
}

message ListRuleSetsResponse {
  repeated RuleSet rule_sets = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message GetRuleSetRequest {
  string id = 1;
}

message GetRuleSetResponse {
  RuleSet rule_set = 1;
}

message CreateRuleSetRequest {
  string name = 1;
  string description = 2;
  string domain = 3;
  string evaluation_mode = 4;
  bytes default_result_json = 5;
  repeated string rule_ids = 6;
}

message CreateRuleSetResponse {
  RuleSet rule_set = 1;
}

message UpdateRuleSetRequest {
  string id = 1;
  optional string description = 2;
  optional string evaluation_mode = 3;
  optional bytes default_result_json = 4;
  repeated string rule_ids = 5;
  optional bool enabled = 6;
}

message UpdateRuleSetResponse {
  RuleSet rule_set = 1;
}

message DeleteRuleSetRequest {
  string id = 1;
}

message DeleteRuleSetResponse {
  bool success = 1;
  string message = 2;
}

message PublishRuleSetRequest {
  string id = 1;
}

message PublishRuleSetResponse {
  string id = 1;
  uint32 published_version = 2;
  uint32 previous_version = 3;
  k1s0.system.common.v1.Timestamp published_at = 4;
}

message RollbackRuleSetRequest {
  string id = 1;
}

message RollbackRuleSetResponse {
  string id = 1;
  uint32 rolled_back_to_version = 2;
  uint32 previous_version = 3;
  k1s0.system.common.v1.Timestamp rolled_back_at = 4;
}

// --- Evaluate messages ---

message EvaluateRequest {
  string rule_set = 1;
  bytes input_json = 2;
  bytes context_json = 3;
}

message MatchedRule {
  string id = 1;
  string name = 2;
  int32 priority = 3;
  bytes result_json = 4;
}

message EvaluateResponse {
  string evaluation_id = 1;
  string rule_set = 2;
  uint32 rule_set_version = 3;
  repeated MatchedRule matched_rules = 4;
  bytes result_json = 5;
  bool default_applied = 6;
  bool cached = 7;
  k1s0.system.common.v1.Timestamp evaluated_at = 8;
}
```

---

## Kafka メッセージング設計

### ルール変更通知

ルールまたはルールセットの作成・更新・削除・公開・ロールバック時に以下のメッセージを Kafka トピック `k1s0.system.rule_engine.rule_changed.v1` に送信する。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.rule_engine.rule_changed.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | ルールセット ID（例: `rs-001`） |

**メッセージ例**

```json
{
  "event_type": "RULE_SET_PUBLISHED",
  "rule_set_id": "rs-001",
  "rule_set_name": "tax_calculation",
  "domain": "accounting",
  "action": "PUBLISHED",
  "version": 3,
  "previous_version": 2,
  "timestamp": "2026-03-05T15:00:00.000+00:00",
  "actor_user_id": "admin-001"
}
```

> `action` は `CREATED` / `UPDATED` / `DELETED` / `PUBLISHED` / `ROLLED_BACK` を使用する。

### config-server 変更通知の購読

config-server がルールエンジン関連の設定変更を通知した場合（Kafka `k1s0.system.config.changed.v1`）、ルール定義の再読み込みをトリガーする。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.config.changed.v1` |
| コンシューマーグループ | `rule-engine.config-watcher` |
| フィルタ | `namespace = "rule-engine"` のメッセージのみ処理 |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Rule`, `RuleSet`, `RuleSetVersion`, `EvaluationResult`, `Condition`, `ConditionNode` | エンティティ・条件式 AST 定義 |
| domain/repository | `RuleRepository`, `RuleSetRepository`, `RuleSetVersionRepository`, `EvaluationLogRepository` | リポジトリトレイト |
| domain/service | `RuleEvaluationService`, `ConditionParser`, `ConditionEvaluator` | 条件式パース・評価・ルールマッチング |
| usecase | `CreateRuleUseCase`, `UpdateRuleUseCase`, `DeleteRuleUseCase`, `CreateRuleSetUseCase`, `UpdateRuleSetUseCase`, `DeleteRuleSetUseCase`, `PublishRuleSetUseCase`, `RollbackRuleSetUseCase`, `EvaluateUseCase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `RulePostgresRepository`, `RuleSetPostgresRepository`, `RuleSetVersionPostgresRepository`, `EvaluationLogPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/cache | `RuleEvalCacheService` | moka キャッシュ実装（ルール定義・評価結果キャッシュ） |
| infrastructure/messaging | `RuleChangeKafkaProducer`, `ConfigChangeKafkaConsumer` | Kafka プロデューサー/コンシューマー |
| infrastructure/audit | `AuditLogSender` | audit-client 経由の監査ログ送信 |

### ドメインモデル

#### Rule

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ルールの一意識別子 |
| `name` | String | ルール名（例: `consumption_tax_rate`） |
| `description` | String | ルールの説明 |
| `priority` | i32 | 優先度（1-1000、小さいほど高優先） |
| `when_condition` | ConditionNode | 条件式の AST |
| `then_result` | serde_json::Value | マッチ時の結果データ |
| `enabled` | bool | ルールの有効/無効 |
| `version` | u32 | バージョン（更新ごとにインクリメント） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### RuleSet

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ルールセットの一意識別子 |
| `name` | String | ルールセット名（例: `tax_calculation`） |
| `description` | String | ルールセットの説明 |
| `domain` | String | 業務領域（例: `accounting`, `fa`） |
| `evaluation_mode` | EvaluationMode | `FirstMatch` / `AllMatch` |
| `default_result` | serde_json::Value | デフォルト結果 |
| `rule_ids` | Vec\<UUID\> | 所属ルール ID 一覧（優先度順） |
| `current_version` | u32 | 現在の公開バージョン |
| `enabled` | bool | ルールセットの有効/無効 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### RuleSetVersion

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | バージョンの一意識別子 |
| `rule_set_id` | UUID | 所属ルールセット ID |
| `version` | u32 | バージョン番号 |
| `rule_ids_snapshot` | Vec\<UUID\> | 公開時点のルール ID スナップショット |
| `default_result_snapshot` | serde_json::Value | 公開時点のデフォルト結果 |
| `published_at` | DateTime\<Utc\> | 公開日時 |
| `published_by` | String | 公開者ユーザー ID |

#### ConditionNode（AST）

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `combinator` | Option\<Combinator\> | `All` / `Any` / `None`（論理結合の場合） |
| `children` | Option\<Vec\<ConditionNode\>\> | 子条件ノード（論理結合の場合） |
| `field` | Option\<String\> | 比較対象フィールドパス（リーフ条件の場合） |
| `operator` | Option\<Operator\> | 比較演算子（リーフ条件の場合） |
| `value` | Option\<serde_json::Value\> | 比較値（リーフ条件の場合） |

### キャッシュ戦略

| 項目 | 値 |
| --- | --- |
| キャッシュライブラリ | moka v0.12 |
| ルール定義キャッシュキー | `rule_set:{domain}.{name}:v{version}` |
| 評価結果キャッシュキー | `eval:{domain}.{name}:v{version}:{input_hash}` （入力 JSON の SHA-256 ハッシュ） |
| TTL | 60 秒 |
| 最大エントリ数 | 100,000 |
| 無効化トリガー | ルールセット公開・ロールバック時に該当ルールセットのエントリを即座に無効化 + Kafka 通知受信時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (rule_handler.rs)            │   │
                    │  │  healthz / readyz / metrics               │   │
                    │  │  list_rules / get_rule / create_rule      │   │
                    │  │  list_rule_sets / create_rule_set         │   │
                    │  │  publish_rule_set / rollback_rule_set     │   │
                    │  │  evaluate / evaluate_dry_run              │   │
                    │  │  list_evaluation_logs                     │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (tonic_service.rs)           │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateRuleUseCase / UpdateRuleUseCase /        │
                    │  DeleteRuleUseCase / CreateRuleSetUseCase /     │
                    │  PublishRuleSetUseCase / RollbackRuleSetUseCase │
                    │  EvaluateUseCase                                │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  Rule,          │              │ RuleRepository             │   │
    │  RuleSet,       │              │ RuleSetRepository          │   │
    │  RuleSetVersion,│              │ RuleSetVersionRepository   │   │
    │  ConditionNode  │              │ EvaluationLogRepository    │   │
    └────────────────┘              │ (trait)                    │   │
              │                      └──────────┬─────────────────┘   │
              │  ┌────────────────┐             │                     │
              └──▶ domain/service │             │                     │
                 │ RuleEvaluation │             │                     │
                 │ Service        │             │                     │
                 │ ConditionParser│             │                     │
                 │ ConditionEval  │             │                     │
                 └────────────────┘             │                     │
                    ┌───────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌──────▼─────────────────┐  │
                    │  │ Kafka        │  │ RulePostgres           │  │
                    │  │ Producer +   │  │ Repository             │  │
                    │  │ Consumer     │  ├────────────────────────┤  │
                    │  └──────────────┘  │ RuleSetPostgres        │  │
                    │  ┌──────────────┐  │ Repository             │  │
                    │  │ moka Cache   │  ├────────────────────────┤  │
                    │  │ Service      │  │ EvaluationLogPostgres  │  │
                    │  └──────────────┘  │ Repository             │  │
                    │  ┌──────────────┐  └────────────────────────┘  │
                    │  │ AuditLog     │  ┌────────────────────────┐  │
                    │  │ Sender       │  │ Database               │  │
                    │  └──────────────┘  │ Config                 │  │
                    │                    └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## DB スキーマ

PostgreSQL の `rule_engine` スキーマに以下のテーブルを配置する。

```sql
CREATE SCHEMA IF NOT EXISTS rule_engine;

CREATE TABLE rule_engine.rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT NOT NULL UNIQUE,
    description  TEXT NOT NULL DEFAULT '',
    priority     INTEGER NOT NULL DEFAULT 100,
    when_json    JSONB NOT NULL,
    then_json    JSONB NOT NULL,
    enabled      BOOLEAN NOT NULL DEFAULT true,
    version      INTEGER NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE rule_engine.rule_sets (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name             TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    domain           TEXT NOT NULL,
    evaluation_mode  TEXT NOT NULL DEFAULT 'first_match',
    default_result   JSONB NOT NULL DEFAULT '{}',
    rule_ids         UUID[] NOT NULL DEFAULT '{}',
    current_version  INTEGER NOT NULL DEFAULT 0,
    enabled          BOOLEAN NOT NULL DEFAULT true,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (domain, name)
);

CREATE TABLE rule_engine.rule_set_versions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_id             UUID NOT NULL REFERENCES rule_engine.rule_sets(id) ON DELETE CASCADE,
    version                 INTEGER NOT NULL,
    rule_ids_snapshot       UUID[] NOT NULL,
    default_result_snapshot JSONB NOT NULL,
    published_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_by            TEXT NOT NULL,
    UNIQUE (rule_set_id, version)
);

CREATE TABLE rule_engine.evaluation_logs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_name    TEXT NOT NULL,
    rule_set_version INTEGER NOT NULL,
    matched_rule_id  UUID,
    input_hash       TEXT NOT NULL,
    result           JSONB NOT NULL,
    context          JSONB NOT NULL DEFAULT '{}',
    evaluated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rules_enabled ON rule_engine.rules(enabled);
CREATE INDEX idx_rule_sets_domain ON rule_engine.rule_sets(domain);
CREATE INDEX idx_rule_set_versions_rule_set_id ON rule_engine.rule_set_versions(rule_set_id);
CREATE INDEX idx_evaluation_logs_rule_set ON rule_engine.evaluation_logs(rule_set_name, evaluated_at DESC);
CREATE INDEX idx_evaluation_logs_evaluated_at ON rule_engine.evaluation_logs(evaluated_at DESC);
```

---

## 設定ファイル例

### config.yaml（本番）
> ※ dev環境では省略可能なセクションがあります。


```yaml
app:
  name: "rule-engine"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8111
  grpc_port: 50051

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.rule_engine.rule_changed.v1"

cache:
  max_entries: 100000
  ttl_seconds: 60

audit:
  enabled: true
  endpoint: "http://audit-server.k1s0-system.svc.cluster.local:8080"
  batch_size: 100
  flush_interval_ms: 5000

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
  issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
  audience: "k1s0-api"
  jwks_cache_ttl_secs: 300
```

### Helm values

```yaml
# values-rule-engine.yaml（infra/helm/services/system/rule-engine/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/rule-engine
  tag: ""

replicaCount: 2

container:
  port: 8111
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/rule-engine/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/rule-engine/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-rule-engine-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-rule-engine-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-policy-server.md](../policy/server.md) -- アクセス制御ポリシー評価サーバー（本サーバーとの役割分担）
- [system-featureflag-server.md](../featureflag/server.md) -- フィーチャーフラグ（A/B テスト連携先）
- [system-config-server.md](../config/server.md) -- 動的設定管理（ホットリロードのトリガー元）
- [system-library-audit-client.md](../../libraries/observability/audit-client.md) -- 監査ログクライアント
- [REST-API設計.md](../../architecture/api/REST-API設計.md) -- D-007 統一エラーレスポンス

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。

### サービス固有メトリクス

| メトリクス名 | 型 | 説明 |
| --- | --- | --- |
| `rule_engine_evaluations_total` | counter | ルール評価実行回数（ラベル: `rule_set`, `domain`, `result_type`） |
| `rule_engine_evaluation_duration_seconds` | histogram | ルール評価実行時間 |
| `rule_engine_cache_hits_total` | counter | 評価結果キャッシュヒット数 |
| `rule_engine_cache_misses_total` | counter | 評価結果キャッシュミス数 |
| `rule_engine_rules_active` | gauge | 有効なルール数 |
| `rule_engine_rule_sets_active` | gauge | 有効なルールセット数 |
