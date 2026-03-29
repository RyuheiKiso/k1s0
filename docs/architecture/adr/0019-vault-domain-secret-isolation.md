# ADR-0019: Vault ポリシーのドメイン単位シークレット分離

## ステータス

承認済み

## コンテキスト

従来の Vault ポリシーは business / service / system の tier 単位で設定されており、
同一 tier 内の全ドメインが同じシークレットパス（例: `secret/data/k1s0/business/*`）へアクセス可能だった。

```hcl
# 現行 business.hcl — tier 全体への read/list が許可されている
path "secret/data/k1s0/business/*" {
  capabilities = ["read", "list"]
}
```

この設計では、特定ドメインのサービスが侵害された場合、攻撃者は同 tier 内の
**全ドメイン**のシークレット（API キー、DB 認証情報、外部サービストークン等）を
読み取ることができる。

k1s0 では business tier に複数ドメイン（project-master、task 等）が存在し、
各ドメインは互いに独立したシークレットを持つ。tier 単位の粗粒度アクセス制御は
最小権限の原則（Principle of Least Privilege）に反するリスク設計となっていた。

## 決定

ドメイン単位のシークレットパスと専用ポリシーファイルを新設する。

- **シークレットパス規約**: `secret/data/k1s0/{tier}/{domain}/*`
  - 例: `secret/data/k1s0/business/project-master/*`
  - 例: `secret/data/k1s0/business/task/*`
- **ポリシーファイル規約**: `infra/terraform/modules/vault/policies/{domain}.hcl`
  - 例: `project-master.hcl`、`task.hcl`
- **既存 tier ポリシー**: 後方互換性のため残し、新規サービスはドメインポリシーを使用する
- **移行方針**: 既存サービスは段階的にドメインポリシーへ切り替え、完全移行後に tier ポリシーを廃止する

## 理由

1. **最小権限の原則**: 各ドメインは自ドメインのシークレットにのみアクセスできれば十分であり、
   他ドメインのパスへのアクセス権は不要
2. **侵害影響範囲の局所化**: あるドメインが侵害されても、影響はそのドメインのシークレットに
   限定され、同 tier 内の他ドメインへの横断的アクセスを防止できる
3. **監査性の向上**: Vault の監査ログでどのドメインのシークレットが参照されたかを
   ドメイン単位で追跡できるようになる
4. **段階的移行が可能**: 既存 tier ポリシーを維持しながら並行導入できるため、
   サービス稼働に影響を与えずに移行できる

## 影響

**ポジティブな影響**:

- 侵害時の爆発半径（Blast Radius）をドメイン単位に限定できる
- ドメインごとにアクセス制御を精細に設定できる（read のみ / write も許可 等）
- セキュリティ監査においてドメイン単位のアクセスログ分析が可能になる
- Zero Trust アーキテクチャへの準拠度が向上する

**ネガティブな影響・トレードオフ**:

- ドメイン数に比例してポリシーファイルの管理コストが増加する
- 既存サービスのデプロイメント設定（Vault ロールのポリシー参照）を変更する作業が発生する
- tier ポリシーと domain ポリシーが並行する移行期間中、設定の一貫性管理が複雑になる
- シークレットの既存パス（`secret/data/k1s0/business/*` 以下）を
  ドメイン別パスへ再配置する作業が必要になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| tier 単位ポリシーのまま維持 | 現状の `business/*` 等の粗粒度ポリシーを継続 | 侵害時の影響範囲が同 tier 全ドメインに及ぶため最小権限の原則に反する |
| サービスアカウント単位のポリシー | デプロイ単位（Pod / コンテナ）ごとにポリシーを発行 | ドメイン内に複数サービスがある場合に管理コストが過大になる。ドメイン単位が適切な粒度 |
| Vault Namespace によるテナント分離 | tier ごとに Vault Namespace を分割 | Vault Enterprise 機能が必要。OSS 版では利用できない |

## System Tier 対応状況（L-14 監査対応）

### 現状（2026-03-29 時点）

business / service tier はドメイン単位のポリシーファイルが存在し、段階的移行中。
system tier については個別 HCL ポリシーファイル（`infra/vault/policies/{service}.hcl`）は全 27 サービス分が既に作成済みだが、
`infra/terraform/modules/vault/auth.tf` の Kubernetes auth ロールは依然として単一 `system` ロールに集約されている。

### 作業残項目（Phase 5 予定）

- `auth.tf` において各 system tier サービス用の個別 `vault_kubernetes_auth_backend_role` リソースを作成する
- 各ロールの `token_policies` を専用 HCL ポリシー（`infra/vault/policies/{service}.hcl`）に紐付ける
- 完全移行後に現在の単一 `system` ロールを廃止する
- 詳細は ADR-0045 を参照

## 参考

- [外部監査対応 2026-03-22](../../memory/project_audit_response_2026_03_22.md) — セキュリティ監査指摘事項への対応
- [ADR-0011: RBAC 管理者権限分離](./0011-rbac-admin-privilege-separation.md) — 最小権限原則の適用事例
- [ADR-0045: Vault サービス個別ロール実装計画](./0045-vault-per-service-roles.md) — system tier 個別ロール移行
- 現行 Vault ポリシー: `infra/terraform/modules/vault/policies/business.hcl`
- HashiCorp Vault: [Policies](https://developer.hashicorp.com/vault/docs/concepts/policies)
