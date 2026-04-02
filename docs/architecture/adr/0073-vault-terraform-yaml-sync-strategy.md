# ADR-0073: Vault ロール定義の権威情報を Terraform に統一し YAML との同期を CI で検証する

## ステータス

承認済み

## コンテキスト

Vault の Kubernetes Auth ロール定義が2つの場所で管理されている状態が続いている。

1. **Terraform**（`infra/terraform/modules/vault/auth.tf`）  
   実際に Vault へ適用される権威情報。`vault_kubernetes_auth_backend_role` リソースとして全サービス分を定義している。

2. **Vault YAML**（`infra/vault/auth/*.yaml`）  
   ConfigMap 形式のリファレンス設定。ADR-0045（Vault per-service roles）の将来計画を含む用途で保持している。  
   現状、`k1s0-system-auth.yaml` の `role_name: auth-rust` のみが存在する。

この二重管理により以下の問題が発生しうる：

- Terraform と YAML のロール名・SA 名・ポリシー名が乖離しても CI で検出されない
- Phase 5 でサービス個別ロールを追加する際に、YAML 側だけ更新して Terraform 側が漏れるリスクがある
- C-05 監査（`bound_service_account_names` の不一致）のような問題が再発する可能性がある

INFRA-002 監査指摘として、この乖離を CI で自動検出する仕組みが必要との指摘を受けた。

## 決定

1. **Terraform を Vault ロール定義の権威情報とする**  
   `infra/terraform/modules/vault/auth.tf` が Vault に適用される唯一の正式定義であることを明示する。

2. **CI で Vault YAML と Terraform の整合性を検証する**  
   `_validate.yaml` に `validate-vault-tf-sync` ジョブを追加し、  
   `infra/vault/auth/*.yaml` に記述されたロール名が Terraform に存在しない場合を警告として検出する。  
   YAML は将来計画を含む可能性があるため `exit 1`（ブロッカー）ではなく `::warning::` に留める。

3. **Vault YAML の位置付けを明確化する**  
   `infra/vault/auth/*.yaml` は「Terraform 移行前のリファレンス・計画ドキュメント」であり、  
   Vault への実際の適用は Terraform のみが行う旨をコメントで明記する。

## 理由

- **Terraform が最小変更で権威情報となれる**：既に全サービスの個別ロールが Terraform に定義済みのため、新たな移行作業なしに権威情報の一元化が達成できる。
- **CI 検証は warning 止まりが適切**：YAML は ADR-0045 の Phase 5 計画も含むため、将来のロールが YAML に先行記述される可能性がある。ブロッカーにすると計画記述ができなくなる。
- **完全な機械的比較は過剰投資**：ロール名のみの検証で乖離の大半を検出できる。SA 名・ポリシー名の完全比較は YAML の JSON パース等が必要であり、リスク低減効果に対してコストが高い。

## 影響

**ポジティブな影響**:

- Vault YAML に定義されたロールが Terraform に存在しない場合、PR で警告が出力されるため乖離を早期に発見できる
- C-05 類似の問題（SA 名の不一致による Vault 認証失敗）の再発リスクが低下する
- 権威情報の所在が明確になり、新規サービス追加時の手順が統一される

**ネガティブな影響・トレードオフ**:

- CI ジョブが1つ追加されるため PR の実行時間がわずかに増加する（30秒未満）
- YAML 側が将来計画を含む場合に warning が出続けるが、これは意図的な設計である
- SA 名・ポリシー名レベルの完全検証は行わないため、それらの乖離は手動レビューに依存する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: YAML を廃止して Terraform のみで管理 | ConfigMap を削除し Terraform を唯一の管理対象とする | YAML は将来計画（Phase 5）のリファレンスとして活用価値がある。急な削除は ADR-0045 計画の可視性を損なう |
| 案 B: YAML を Terraform の入力として自動生成 | Python/Jinja 等で YAML → Terraform HCL を自動生成 | 生成ツールの保守コストが発生し、モノリポの複雑度を増す |
| 案 C: 完全な機械的比較（ロール名 + SA 名 + ポリシー名） | JSON パースを含む詳細比較スクリプトを CI に追加 | YAML の JSON フォーマットが変更されると CI が壊れるリスクがある。ロール名比較でリスクの大半をカバーできる |
| 案 D: exit 1 で PR をブロック | 乖離を検出したら CI 失敗とする | YAML は将来計画を含むため、新しいロールを計画段階で YAML に記述すると CI が壊れる。warning が適切 |

## 参考

- [ADR-0045: Vault per-service roles](0045-vault-per-service-roles.md)
- `infra/terraform/modules/vault/auth.tf` — Vault ロール定義の権威情報
- `infra/vault/auth/k1s0-system-auth.yaml` — リファレンス ConfigMap
- `.github/workflows/_validate.yaml` — `validate-vault-tf-sync` ジョブ

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（INFRA-002 監査対応） | @kiso |
