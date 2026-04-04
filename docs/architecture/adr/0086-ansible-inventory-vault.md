# ADR-0086: Ansible inventory の Vault 化ロードマップ

## ステータス

承認済み（移行中）

## コンテキスト

外部技術監査（M-019）において、`infra/ansible/inventory/hosts.yaml` に内部 IP アドレスが
平文で記載されていることが指摘された。

現在の構成:

```yaml
dev-master-01:
  ansible_host: 10.10.1.11
dev-worker-01:
  ansible_host: 10.10.1.21
...
```

内部 IP は外部から直接到達できないプライベートアドレスだが、以下のリスクがある:
- リポジトリが公開された場合にインフラ構成が漏洩する
- 内部ネットワーク構成の変更追跡がコードに依存する
- ゼロトラスト原則（機密情報は暗号化して管理）に違反している

## 決定

中期的に Ansible Vault または動的インベントリに移行する。
移行は以下のフェーズで段階的に実施する:

### フェーズ 1（短期・現状維持）
- 現在の平文 IP はプライベートアドレスであり外部公開リポジトリでないため、直ちに対処が必要なセキュリティリスクではない
- 本 ADR によって問題を明文化し、移行計画を策定する

### フェーズ 2（中期・2026-Q3 目標）
以下のいずれかに移行する:

**オプション A: Ansible Vault による暗号化**
```bash
ansible-vault encrypt_string '10.10.1.11' --name 'ansible_host'
```
- hosts.yaml の ansible_host 値を Vault で暗号化する
- vault-password-file は CI/CD の Secrets Manager で管理する

**オプション B: 動的インベントリ**
- Terraform output から動的インベントリスクリプトで IP を取得する
- `infra/ansible/inventory/` に `hosts.py` を追加して Terraform state から IP を解決する

## 理由

- Ansible Vault はシンプルで既存ワークフローへの影響が最小限
- 動的インベントリは Terraform との一元管理が可能だが実装コストが高い
- どちらを選択するかは実際のデプロイフローを確認した上で判断する

## 影響

**ポジティブな影響**:

- 内部ネットワーク構成情報の漏洩リスクを低減する
- ゼロトラスト原則への準拠度が向上する

**ネガティブな影響・トレードオフ**:

- 移行作業コストが発生する
- CI/CD パイプラインに Vault パスワード管理の仕組みを追加する必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持 | IP を平文のまま管理する | セキュリティ原則違反のため中期的には移行が必要 |
| CMDB 連携 | IT 資産管理データベースから動的取得 | 導入コストが高く現時点では過剰 |

## 参考

- [Ansible Vault ドキュメント](https://docs.ansible.com/ansible/latest/user_guide/vault.html)
- `infra/ansible/inventory/hosts.yaml` — 対象ファイル

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（M-019 監査対応） | k1s0 team |
