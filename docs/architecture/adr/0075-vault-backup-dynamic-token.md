# ADR-0075: Vault バックアップ CronJob の動的トークン発行方式への移行

## ステータス

提案

## コンテキスト

`infra/kubernetes/backup/vault-backup-cronjob.yaml` の Vault バックアップ CronJob は、
Vault Raft スナップショットを取得するために `VAULT_TOKEN` 環境変数を使用している。
現状このトークンは Kubernetes Secret（`vault-backup-credentials`）に静的に保存されており、
以下のリスクが存在する。

1. **トークン漏洩リスク**: Kubernetes Secret は base64 エンコードのみであり、
   etcd 暗号化が設定されていない環境では平文と等価。
   Secret が誤って参照・ログ出力された場合にトークンが漏洩する。
2. **ローテーション運用コスト**: 静的トークンは手動でローテーションする必要があり、
   ローテーション漏れによって長期間有効なトークンが残存するリスクがある。
3. **最小権限原則の未徹底**: 静的トークンは CronJob 実行中以外も有効であり、
   不要な時間帯でも Vault API へのアクセス権を保持する。

INFRA-006 監査指摘として、動的トークン発行方式への移行が推奨されている。

## 決定

Vault バックアップ CronJob において、Vault Agent Sidecar Injector（または Vault Agent
Init Container）を使用した動的トークン発行方式に移行する。

具体的には以下の構成を採用する:
- CronJob Pod に Vault Agent を init container として追加する
- Kubernetes Auth Method を使用して Pod の ServiceAccount トークンで Vault に認証する
- Vault の `operator/raft/snapshot` パスへのポリシーを持つ短命トークン（TTL: 1h）を動的発行する
- 発行されたトークンを Vault Agent が `/vault/secrets/token` ファイルとして Pod に注入する
- CronJob のバックアップスクリプトはファイルからトークンを読み込む

## 理由

| 観点 | 静的トークン（現状） | 動的トークン（移行後） |
|------|--------------------|--------------------|
| トークン有効期間 | 無期限（手動ローテーションまで） | TTL: 1h（ジョブ実行時のみ有効） |
| ローテーション | 手動運用が必要 | Vault が自動管理 |
| 漏洩時の影響範囲 | 即時の無効化操作が必要 | TTL 経過で自動失効 |
| Kubernetes Secret への依存 | あり | なし（Vault Kubernetes Auth に集約） |
| 監査ログ | Vault 監査ログに記録されるが、誰が使ったか不明 | Pod/ServiceAccount 単位で追跡可能 |

動的トークン方式を採用することで、最小権限の原則を徹底し、
運用コストを削減しながらセキュリティレベルを向上させる。

また、すでに本プロジェクトでは Vault Agent Injector を採用しており（ADR-0045）、
同一パターンを踏襲することで一貫性を保つことができる。

## 影響

**ポジティブな影響**:

- トークンの有効期間が CronJob 実行中のみに限定され、漏洩時の影響範囲が縮小する
- Kubernetes Secret への静的トークン保存が不要になり、シークレット管理が Vault に集約される
- Vault 監査ログで ServiceAccount 単位の操作追跡が可能になる
- ローテーション作業が完全自動化される

**ネガティブな影響・トレードオフ**:

- CronJob の Pod spec に Vault Agent Injector のアノテーションを追加する必要がある
- Vault の Kubernetes Auth Method と対応する Vault ロールポリシーを設定する必要がある
- Vault が停止している場合、バックアップ CronJob も実行できなくなる（循環依存）
  - 緩和策: Vault 自体のバックアップ（Raft スナップショット）は Vault の稼働が前提のため許容する
  - 緩和策: Vault の HA 構成（3ノード以上）で可用性を確保する（既存構成）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 静的トークン（現状維持） | Kubernetes Secret にトークンを保存 | 漏洩リスクと手動ローテーション運用コストが残存する |
| External Secrets Operator | ESO で Vault からトークンを同期 | Kubernetes Secret にいったん書き込まれるため静的トークンと本質的に同じリスクがある |
| Vault Agent Sidecar（長期稼働） | Deployment として常時起動する Vault Agent | CronJob は一時的な実行のため Sidecar より Init Container が適切 |
| AppRole Auth Method | ロールID/シークレットIDによる認証 | Kubernetes Auth の方が ServiceAccount ベースで管理が一元化できる。AppRole はシークレットIDの安全な受け渡しが別途必要 |

## 実装手順

移行時は以下の順序で実施する:

1. Vault に Kubernetes Auth Method のロールを追加する
   ```hcl
   # vault-backup-cronjob 専用ポリシー
   path "sys/storage/raft/snapshot" {
     capabilities = ["read"]
   }
   ```
2. Vault ロールに `vault-backup-sa` ServiceAccount を紐付ける
3. CronJob の Pod テンプレートに Vault Agent Injector アノテーションを追加する
   ```yaml
   annotations:
     vault.hashicorp.com/agent-inject: "true"
     vault.hashicorp.com/agent-init-first: "true"
     vault.hashicorp.com/role: "vault-backup"
     vault.hashicorp.com/agent-inject-secret-token: "auth/token/create"
     vault.hashicorp.com/agent-pre-populate-only: "true"
   ```
4. バックアップスクリプトを `VAULT_TOKEN` 環境変数ではなく
   `/vault/secrets/token` ファイル参照に変更する
5. 既存の Kubernetes Secret（`vault-backup-credentials`）を削除する

## 参考

- [ADR-0045: Vault per-service roles](0045-vault-per-service-roles.md)
- [ADR-0031: etcd 暗号化保存](0031-etcd-encryption-at-rest.md)
- [Vault Agent Injector ドキュメント](https://developer.hashicorp.com/vault/docs/platform/k8s/injector)
- [Vault Kubernetes Auth Method](https://developer.hashicorp.com/vault/docs/auth/kubernetes)
- `infra/kubernetes/backup/vault-backup-cronjob.yaml`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（INFRA-006 監査対応） | @kiso |
