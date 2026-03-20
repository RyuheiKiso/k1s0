# ドメインマスタ管理 (Domain Master Management)

## 概要

master-maintenance サーバーに `domain_scope` を導入し、業務領域（accounting, fa 等）ごとに固有のマスタデータをスキーマ分離して管理する機能。
既存のシステムスコープテーブル（domain_scope=NULL）は後方互換で動作し続ける。

## DB変更

### table_definitions テーブル
- `domain_scope VARCHAR(100) DEFAULT NULL` 追加
- NULL = システム共通（既存動作維持）
- 値あり = ドメイン固有（例: "accounting", "fa"）
- UNIQUE制約: `(name, COALESCE(domain_scope, '__system__'))`

### change_logs テーブル
- `domain_scope VARCHAR(100) DEFAULT NULL` 追加

### インデックス
- `idx_table_definitions_domain_scope` ON table_definitions(domain_scope)
- `idx_change_logs_domain_scope` ON change_logs(domain_scope)

## REST API 拡張

全既存エンドポイントに `?domain_scope=xxx` クエリパラメータを追加。未指定はシステムスコープ。

### 新規エンドポイント
- `GET /api/v1/domains` — 登録済みドメイン一覧を取得

## gRPC API 拡張

各リクエストメッセージに `optional string domain_scope` フィールドを追加。
空文字列 = 未指定として扱う。

### 新規 RPC
- `ListDomains` — 登録済みドメイン一覧を取得

## RBAC

### 既存ロール（後方互換）
- `sys_admin`: 全ドメインアクセス可
- `sys_operator`: システムスコープのread/write
- `sys_auditor`: システムスコープのread

### ドメインロール（新規）
- `{domain}_admin`: 該当ドメインの全操作
- `{domain}_operator`: 該当ドメインのread/write
- `{domain}_auditor`: 該当ドメインのread

## Kafka イベント

### 既存トピック
- ペイロードに `domain_scope` フィールド追加

### ドメイン別トピック（新規）
- パターン: `k1s0.business.{domain}.mastermaintenance.data_changed.v1`
- domain_scope が設定されている場合、ドメイン別トピックにも発行

## 後方互換性

- domain_scope は全て Optional。未指定 = NULL = システムスコープ
- REST API はクエリパラメータ方式（パス変更なし）
- gRPC は proto3 optional string（空文字列 = 未指定）
- 既存テスト: domain_scope=None で呼び出すだけで変更不要
