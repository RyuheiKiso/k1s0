# ADR-0066: config サービスの機密設定値 AES-256-GCM 暗号化

## ステータス

承認済み

## コンテキスト

外部技術監査（2026-04-01）の STATIC-HIGH-002 指摘: `config_entries.value_json` が平文で保存されており、DB への不正アクセス時に認証情報・DB 接続情報等が漏洩するリスクがある。

具体的な問題:
- `system.auth.*` / `system.database.*` 等の機密 namespace の設定値が JSONB として平文保存
- `notification` サービスには暗号化実装済み（`k1s0-encryption` ライブラリ）だが `config` サービスは未対応
- `config_entries` テーブルに `encrypted_value` / `is_encrypted` カラムなし

## 決定

### 暗号化アルゴリズム

`k1s0-encryption::aes_encrypt` / `aes_decrypt`（AES-256-GCM）を採用する。既存のライブラリ（`regions/system/library/rust/encryption/`）を再利用することで実装コストを最小化する。

### 実装方針

| コンポーネント | 説明 |
|--------------|------|
| マイグレーション `011` | `config_entries` に `encrypted_value TEXT` と `is_encrypted BOOLEAN NOT NULL DEFAULT false` を追加 |
| `ConfigPostgresRepository.encryption_key` | `Option<[u8; 32]>` — None の場合は暗号化無効 |
| `ConfigPostgresRepository.sensitive_namespaces` | 暗号化対象の namespace プレフィックスリスト（設定ファイルで変更可能） |
| `encrypt_value()` | 機密 namespace かつ鍵あり → `value_json = '{}'`、`encrypted_value = ciphertext`、`is_encrypted = true` |
| `row_to_config_entry()` | `is_encrypted = true` → `aes_decrypt(key, encrypted_value)` を復号して返す |

### 暗号化鍵の取得

優先順位: 環境変数 `CONFIG_ENCRYPTION_KEY`（base64 エンコード 32 バイト）> `config_server.encryption.key_base64`

Kubernetes 環境では Vault SecretProviderClass 経由で環境変数に注入する。

### 設定スキーマ

```yaml
config_server:
  encryption:
    enabled: false                   # 本番環境では true
    sensitive_namespaces:
      - "system.auth"
      - "system.database"
    key_base64: ""                   # 実際の鍵は CONFIG_ENCRYPTION_KEY 環境変数
```

### レスポンス

暗号化は透過的に行われる。API クライアントは平文の JSON 値を受け取り、暗号化/復号は Repository 層で完結する。

## 理由

- **既存ライブラリの再利用**: `notification` サービスで実績のある `k1s0-encryption` ライブラリを使用し、暗号化実装の一貫性を保つ
- **段階的移行**: `is_encrypted` フラグにより既存の平文エントリと暗号化エントリが共存可能。`enabled: false` のままデプロイ後、鍵を設定して有効化できる
- **設定可能な sensitive_namespaces**: 暗号化対象を運用者が制御できる。将来的な namespace 追加に対応
- **デュアルリード設計**: `is_encrypted = false` の既存エントリは `value_json` から読み取り、新規/更新エントリは自動的に暗号化される

## 影響

**ポジティブな影響**:
- DB への不正アクセス時の機密設定値漏洩リスクを低減
- `notification` サービスと暗号化方式が統一される
- 事後監査: `is_encrypted` カラムで暗号化状況を確認可能

**ネガティブな影響・トレードオフ**:
- 本番環境に `CONFIG_ENCRYPTION_KEY` 環境変数の管理が必要
- `enabled: false` から `true` への移行時、既存エントリは次回更新時に暗号化される（一括移行ツールは不要だが漸進的）
- キャッシュ層（`CachedConfigRepository`）は復号済み値をキャッシュするため、鍵変更時はキャッシュ無効化が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| PostgreSQL pgcrypto | DB 側で暗号化 | アプリ鍵管理が複雑、Vault 連携が難しい |
| Vault Dynamic Secrets | 設定値を Vault に移行 | 大規模な変更、config-server の役割が変わる |
| 全カラム暗号化 | value_json を常に暗号化 | 非機密値も暗号化する必要はなくオーバーヘッドが高い |

## 参考

- `regions/system/library/rust/encryption/` — AES-256-GCM 暗号化ライブラリ
- `regions/system/database/config-db/migrations/011_encrypt_config_values.up.sql`
- `regions/system/server/rust/config/src/adapter/repository/config_postgres.rs`
- 外部技術監査報告書 2026-04-01: STATIC-HIGH-002

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-01 | 初版作成（外部監査 STATIC-HIGH-002 対応） | @kiso ryuhei |
