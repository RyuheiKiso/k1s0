# ADR-0052: JSONB カラム暗号化戦略

## ステータス

部分実装済み（notification.channels.config 実装完了。vault.secrets.metadata / file_metadata は将来対応）

## コンテキスト

外部監査 C-005 の指摘により、以下の JSONB カラムが平文で保存されていることが判明した。

- `notification.channels.config` — チャネル設定（Webhook URL、API キー等の認証情報を含む可能性）
- `vault.secrets.metadata` — Vault シークレットのメタデータ（`secret_versions.encrypted_data` は BYTEA で暗号化済みだが metadata は未対応）
- `file_storage.file_metadata.metadata`, `tags` — ファイルメタデータ

PCI-DSS / SOC2 要件の観点から、認証情報を含む可能性のある JSONB カラムの平文保存はセキュリティリスクとなる。

## 決定

アプリケーション層暗号化による段階的対応を採用する。

### 優先度

| 優先度 | テーブル・カラム | 理由 |
|--------|-----------------|------|
| 高 | `notification.channels.config` | 認証情報（Webhook URL、API キー）を含むリスクが最も高い |
| 中 | `vault.secrets.metadata` | Vault シークレットに関連するメタデータ |
| 低 | `file_storage.file_metadata.metadata`, `tags` | 一般的に非機密データ |

### 実装方針

- Vault の既存暗号化インフラを活用する
- 暗号化形式は以下のいずれかを採用:
  - BYTEA + nonce BYTEA カラムへの移行
  - `{"ciphertext": "<base64>", "nonce": "<base64>"}` 形式の JSONB として保存

### 移行戦略（2フェーズ）

1. **dual-read 期間**: 暗号化データと平文データの両方を読み取り可能にする。新規書き込みは暗号化形式で行う
2. **平文除去**: 全データの暗号化移行完了後、平文カラムを削除する

**実装状況（2026-03-29 更新）**:
- `notification.channels.config`: **実装完了**（migration 011 + `channel_postgres.rs` AES-256-GCM 暗号化）
- `vault.secrets.metadata`: 将来対応（vault.secret_versions に encrypted_data BYTEA が既存。metadata は補助情報のため相対的優先度低）
- `file_storage.file_metadata.metadata/tags`: 将来対応（機密情報を含む可能性が低いため優先度低）

## 理由

- PCI-DSS / SOC2 要件への対応として、認証情報を含む可能性のある JSONB カラムの暗号化が必要
- 認証情報（Webhook URL、API キー等）の平文保存は、DB への不正アクセス時に被害を拡大させるリスクがある
- Vault の既存暗号化インフラを活用することで、新たな暗号化基盤の導入コストを抑制できる
- 段階的対応により、リスクの高いカラムから優先的に対処できる

## 影響

**ポジティブな影響**:

- 機密 JSONB データの保護が実現される
- PCI-DSS / SOC2 準拠に向けた対応が進む
- DB レベルでの情報漏洩リスクが軽減される

**ネガティブな影響・トレードオフ**:

- アプリケーション層での encrypt / decrypt 処理の追加が必要
- 暗号化・復号によるパフォーマンスオーバーヘッドが発生する
- dual-read 期間中のコード複雑性が一時的に増加する
- 暗号化カラムに対する JSONB クエリ（フィルタリング等）が不可能になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| pgcrypto 列暗号化 | PostgreSQL の pgcrypto 拡張による DB レベルでの暗号化 | Vault 暗号化との二重管理が発生する。sqlx との統合が複雑になる。鍵管理が DB 側に分散する |
| Transparent Data Encryption (TDE) | ディスクレベルでの暗号化 | DB アクセス権限を持つ攻撃者に対しては保護にならない。カラム単位の暗号化が不可能 |
| 全カラム一括暗号化 | 優先度を設けず全カラムを同時に暗号化 | 移行リスクが高く、段階的対応の方が安全 |

## 参考

- [ADR-0022: Vault シークレット管理](0022-vault-secret-management.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成 | @team |
| 2026-03-29 | notification.channels.config の AES-256-GCM 暗号化実装完了（migration 011）。ステータス更新 | @team |
