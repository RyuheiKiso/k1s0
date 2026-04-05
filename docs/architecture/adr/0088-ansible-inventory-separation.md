# ADR-0088: Ansible インベントリの環境分離方針

## ステータス

承認済み

## コンテキスト

L-008 外部技術監査で、Ansible インベントリが dev/staging/prod を統一的に扱っており、
環境分離のベストプラクティスに反するという指摘があった。

現在の構成は `infra/ansible/inventory/hosts.yaml` の単一ファイルに全環境（dev/staging/prod）の
ホスト定義が同居している。Ansible のベストプラクティスでは環境ごとに別ディレクトリ
（例: `inventory/dev/hosts.yaml`, `inventory/staging/hosts.yaml`, `inventory/prod/hosts.yaml`）
に分離することが推奨されている。

一方、現在のファイルは YAML の `children:` 階層により dev/staging/prod が明確にセクション分離され、
環境ごとの `vars:` も独立して定義されている。

## 決定

現時点では単一ファイル構成を維持し、ディレクトリ分離への移行は将来課題とする。

## 理由

1. **現行の分離は機能的に有効**: `hosts.yaml` 内で dev/staging/prod が YAML 階層として明確に分離されており、
   `env:` 変数により実行時の環境識別が可能。Ansible の `--limit dev` / `--limit staging` / `--limit prod`
   オプションで誤った環境への適用を防止できる。

2. **移行コストと安定性トレードオフ**: ディレクトリ分離への移行は既存の playbooks・roles・CI/CD パイプラインの
   参照パス変更を伴い、適切な回帰テストなしに実施すると本番インフラへの影響リスクがある。

3. **ADR-0086 との整合**: ADR-0086 で Ansible Vault または動的インベントリへの移行が既にロードマップに
   含まれている。ディレクトリ分離よりも Vault 暗号化（M-019 対応）を優先する。

## 将来のロードマップ

動的インベントリ（Terraform output 連携）への移行時に合わせて、以下の構成に再編成する予定:

```
infra/ansible/inventory/
├── dev/
│   ├── hosts.yaml
│   └── group_vars/
│       └── all.yaml
├── staging/
│   ├── hosts.yaml
│   └── group_vars/
│       └── all.yaml
└── prod/
    ├── hosts.yaml
    └── group_vars/
        └── all.yaml
```

移行時は以下を同時実施する:
- IP アドレスの Ansible Vault 暗号化（ADR-0086 対応）
- Terraform output からの動的 IP 取得スクリプト追加
- CI/CD パイプラインの `--inventory` パス更新

## 影響

**ポジティブな影響**:

- 現状の安定稼働を維持しつつ、監査指摘に対する設計意思決定を明文化できる
- 将来の移行ロードマップを明確化し、段階的な改善が可能になる

**ネガティブな影響・トレードオフ**:

- 現時点ではベストプラクティスの完全準拠ではない
- 誤って `--limit` を省略した場合に全環境が対象になるリスクが残る（運用ルールで補完すること）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: 即時ディレクトリ分離 | 環境ごとにディレクトリを分割する | 移行コストが高く、本番インフラへの影響リスクがある |
| 案 B: 現状維持（ADR なし） | 変更なし | 監査指摘に対する設計意思決定が不明確のまま残る |
| 案 C: 動的インベントリへの即時移行 | Terraform output 連携で動的生成 | ADR-0086 の Vault 移行と同時進行させるべき大規模変更 |

## 参考

- [ADR-0086: Ansible インベントリの Vault 暗号化](0086-ansible-inventory-vault.md)
- [Ansible Best Practices: Directory Layout](https://docs.ansible.com/ansible/latest/tips_tricks/sample_setup.html)
- `infra/ansible/inventory/hosts.yaml` — 現行インベントリファイル

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（L-008 外部技術監査対応） | @kiso ryuhei |
