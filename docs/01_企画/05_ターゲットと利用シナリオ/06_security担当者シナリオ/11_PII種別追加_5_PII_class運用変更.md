---
id: plan.security.scenario_pii_class_add_change
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes: []
---

# PII 種別追加 / 5 PII class 運用変更

## 一文方針

新規 PII 種別の追加または既存 PII class の運用変更が必要になった時に、`pii_classification.lock.yaml` への新 entry 追加 → 6 軸 cross-axis bind（鍵管理 / schema breaking / RLS FORCE / preservation_class / client 暗号化 / 監査）の機械検証 → data 担当者との専用クラスタ設定同期の順で実施し、CI の PII coverage check が green であることを確認してクローズする。

## Trigger（発火条件）

tier2 担当者または法務 / DPO から新規 PII 種別の登録依頼が来た時、または業界規制（ISO 9001 / HACCP / ISO 14001）の改定で PII class の定義変更が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（年 1-3 件）。典型きっかけ: 「新規業界 pack（サービス業）の検討で `v1_pii_location`（工場内 GPS 軌跡）が既存 5 class に存在しない。製造業 pack の現場担当者の動線データが該当するため、security 担当者が新 PII 種別として追加判断する」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 協調: data 担当者（PII 専用クラスタ設定変更、シナリオ依存 data-07）
- 協調: tier2 担当者（schema への `field_pii` annotation 追加、Apicurio Registry 更新）
- 確認: formal 担当者（PII 変更が proof_class bind に影響する場合）

## 前提

- `pii_classification.lock.yaml` が build artifact として存在し、tier2 schema の `field_pii` annotation と双方向 lock が CI で管理済み
- 6 軸 cross-axis bind チェック（security 強制機構の層 B: Conftest による PII coverage check）が CI に組み込み済み
- data 担当者が data-07（PII 専用クラスタ運用）のシナリオを熟知していること

## 流れ

1. 新規 PII 種別の定義を法務 / DPO と協議し、対応する PII class（5 class のうちどれに該当するか、または新 class が必要か）を決定する
2. **新 class が必要な場合**: `pii_classification.lock.yaml` の generator script に新 class entry を追加し、class bundle の 6 軸 bind（`v1_authn_authz` / `v1_crypto` / `v1_isolation` / `v1_admission_block` / `v1_audit_detect` の各 dimension）を宣言する。dimension override 禁止のため、class bundle 単位で一括宣言する
3. **既存 class への分類の場合**: 対象 PII column を tier2 schema の `field_pii` annotation で `pii_class=v1_pii_<class>` と明示する。Apicurio Registry に互換性確認（`v1_forward_compatible` 以上）後 publish する
4. `threat_model.lock.yaml` の `v1_pii` asset column を新 PII 種別が影響する surface で確認し、mitigation pointer の漏れがないかチェックする（シナリオ 01 の手順）
5. CI の PII coverage check（`tools/docs_lint` 相当）をローカルで実行し、6 軸 bind が全 entry で green であることを確認する
6. data 担当者へ連絡し、data-07（PII 専用クラスタ運用）のシナリオを先行 PR として依頼する（security と data の同時 PR は禁止：依存方向 lock）
7. data-07 が merge されてから `pii_classification.lock.yaml` の PR を merge する（順序制約）
8. security 担当者 2 名の dual reviewer sign-off を取得する

## 関連適合仕様 / 関連 OSS

- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: Apicurio Registry / CloudNativePG（RLS FORCE）/ OpenBao Transit（DEK wrap）/ Kyverno

## 期待結果 / 観測指標

- `pii_classification.lock.yaml` に新 PII 種別が 6 軸 bind を持つ entry として追加済み
- CI の PII coverage check が green（6 軸 bind 欠落 = CI fail）
- Apicurio Registry の schema compatibility check が green
- data-07 が先行 merge 済みで PII 専用クラスタ設定が反映済み
- dual reviewer sign-off 記録済み

## 失敗時の挙動 / escalation

- **6 軸 bind の一部が欠落（CI fail）**: 欠落している軸（例: preservation_class が `v1_pii_basic` のデフォルトより低い）を特定し、class bundle で一括修正する。dimension override での個別修正は禁止
- **data-07 が先行せず同時 PR を送ってしまった**: data 担当者に order violation を通知し、security PR を draft に戻す。data-07 が merge されてから re-open する。CI の依存方向チェックが warning を出すため、そこで順序違反を検知できる
- **法務 / DPO との合意に時間がかかり cadence 内に完了できない**: 新 PII 種別の本番運用開始を凍結（schema に annotation なし、新 PII を含む機能 PR は merge block）。合意が得られてから追加する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: PII 保護方針](../../../03_概要設計/07_security設計方針/06_PII保護方針.md) — 5 PII class taxonomy と 6 軸 cross-axis bind の正典
- [data 担当者シナリオ: PII 専用クラスタ運用](../05_data担当者シナリオ/07_PII専用クラスタ運用.md) — 本シナリオの依存 data-07（先行 merge 必須）
- [security 強制機構](../../../04_詳細設計/02_強制機構/06_security強制機構.md) — PII coverage check の物理 enforcement 詳細
