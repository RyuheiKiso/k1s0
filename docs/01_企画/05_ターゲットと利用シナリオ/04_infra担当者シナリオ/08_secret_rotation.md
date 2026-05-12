---
id: plan.infra.scenario_secret_rotation
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# secret rotation

## 一文方針

KEK shamir / cosign signing key / OpenBao dynamic secret / TLS cert の定期ローテーションを分類ごとの手順で安全に実施し、漏洩疑い時は即時 revoke → 全 workload 強制 restart → security 担当者 escalation の順で対応する。

## Trigger（発火条件）

- KEK shamir 鍵の定期ローテーションタイミングが到来した時
- cosign 署名鍵のローテーションが必要になった時
- OpenBao シークレット TTL 切れが検出された時
- 漏洩疑いが発生した時

## 想定頻度 / 典型きっかけ

想定頻度: KEK shamir = 年次 / TLS cert = cert-manager が自動化。典型きっかけ: 「KEK shamir の年次 ceremony を Backstage runbook に従って実施する。M=3 / N=5 の custodian が集合」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: プラットフォーム運営者（KEK shamir ceremony の M 名同席必須）、security 担当者（漏洩疑い時）、dual reviewer

## 前提

- [KEK shamir M-of-N ceremony](../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md) runbook が Backstage に登録済みであること
- `secret_rotation.lock.yaml` がローテーション管理の SOT（Source of Truth）であること
- cert-manager が TLS cert の自動ローテーションを管理していること

## 流れ

1. ローテーション対象を分類: KEK shamir / cosign signing key / OpenBao dynamic secret / TLS cert
2. [KEK shamir M-of-N ceremony](../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md) を Backstage runbook で実行（プラットフォーム運営者 M 名が同席必須）
3. cosign signing key ローテーション: 旧 key で署名済み image は検証可能に保ちつつ新 key で再署名
4. OpenBao dynamic secret: TTL 更新 or 強制 revoke → 影響 workload の環境変数を再 inject（Pod restart）
5. TLS cert: cert-manager で自動ローテーション確認 / 手動対応が必要な場合は IaC で更新
6. 漏洩疑い時: 即時 revoke → 全 workload 強制 restart → postmortem 必須 + security 担当者への escalation
7. ローテーション完了を `secret_rotation.lock.yaml` に追記 + dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- 鍵管理適合仕様: [../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- KEK_shamir_distribution: [../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md](../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)
- 関連 OSS: OpenBao / cosign / cert-manager / Backstage / Kyverno

## 期待結果 / 観測指標

- 新しい key / secret がすべての対象 workload で正常に機能していること
- 旧 key で署名済み image の検証が引き続き可能であること（cosign）
- `secret_rotation.lock.yaml` にローテーション完了記録が追記されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- M-of-N 定足数未達 → ceremony 中止 / 次回 ceremony を再スケジュール
- cosign 鍵漏洩疑い → security 担当者に Mattermost `#security-incident` で即時通報（SLA: 30 分以内）。Backstage runbook `emergency-key-rotation` を起動。production cluster への未署名 image 展開は Kyverno で物理拒否（defense-in-depth 層 E）
- cosign 再署名失敗 → supply chain 担当者 escalation + 影響 image の admission 停止
- OpenBao dynamic secret revoke 後の Pod restart 失敗 → 手動復旧 + incident 登録
- 漏洩疑い対応遅延 → security 担当者即時 escalate + 影響範囲の workload を強制停止

**escalate 先**: security 担当者 / Mattermost `#security-incident`
**SLA**: cosign 鍵漏洩疑い時は 30 分以内通報
**runbook**: Backstage runbook `emergency-key-rotation` を参照

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [04_Kyverno_admission_policy追加.md](04_Kyverno_admission_policy追加.md) — Kyverno admission policy 追加シナリオ
- [05_GitOps配信_progressive_delivery.md](05_GitOps配信_progressive_delivery.md) — GitOps 配信 progressive delivery シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
