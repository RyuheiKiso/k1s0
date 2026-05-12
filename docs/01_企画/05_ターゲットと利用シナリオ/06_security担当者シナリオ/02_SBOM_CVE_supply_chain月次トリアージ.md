---
id: plan.security.scenario_sbom_cve_monthly_triage
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# SBOM / CVE / supply chain 月次トリアージ

## 一文方針

Syft 生成 SBOM の差分確認・CVE feed（OSV / NVD）との照合・5 検知経路からの alert 消化を月次で実施し、severity 別 patch SLO（high 14d / medium 30d / low 90d）に従い修正 PR を起票または `accepted_risk` として `cve_triage.lock.yaml` に記録し、1.0.0 ship blocker の `release_gate.lock.yaml` が green を維持することを物理確認する。

## Trigger（発火条件）

月次 cycle の到来（毎月第 2 営業日）、または Renovate / dependabot / Trivy の CI alert が severity high 以上の CVE を検知した時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次（緊急は随時）。典型きっかけ: 「Renovate が `openssl-sys 0.9.92` に CVE-2024-XXXX（CVSS 9.1）を検知した。tier1 Rust binary がこの crate に直接依存している」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: tier1 担当者（Rust crate / Go module の依存グラフ変更 + cargo-deny / govulncheck 確認）
- 関与: infra 担当者（base image / K8s cluster component の CVE 対応）
- 関与: tier2 / tier3 担当者（各言語 SDK の dep 更新）

## 前提

- 全 build artifact に Syft で生成した SPDX + CycloneDX 形式 SBOM が添付済み
- `oss_inventory.lock.yaml` が存在し、採用 OSS 全件を管理済み
- `cve_triage.lock.yaml` が build artifact として CI で整合チェック済み
- Renovate が自動 PR を weekly で生成するよう設定済み
- cargo-deny / Roslyn analyzer / depguard / ESLint の B 層 lint が CI に組み込み済み

## 流れ

1. `oss_inventory.lock.yaml` の全 entry に対し Trivy + Grype double-bound scan を実行し、前月差分の CVE 一覧を生成する（`tools/lock_yaml_generator/cve_diff`）
2. 5 検知経路（gitleaks pre-commit / Trivy image scan / Grype SBOM scan / cargo-deny / OSV API feed）からの alert を統合し、CVSS score + exploitability（known exploit あり / なし）で severity を確定する
3. 各 CVE を threat_model の cell にマッピングし、対応 mitigation_class を `cve_triage.lock.yaml` に記録する。known exploit がある CVE は強制的に severity = high 扱い
4. patch 可能な CVE: Renovate PR が既存なら review + merge。存在しなければ手動 PR 起票。patch SLO（high 14d / medium 30d / low 90d）を `cve_triage.lock.yaml` の `patch_due_at` に記録する
5. patch 不可能 / upstream 未修正 CVE: `accepted_risk` として `unreachable_reason` + `review_due_at`（30d）を記入する。`accepted_risk` count は release_gate の AND-gate 入力（cap 超えは ship blocker）
6. supply chain 疑義（cosign verify が失敗した image / Rekor inclusion proof が欠落した artifact）は即時シナリオ 08 に escalate する。月次トリアージ内で解決を試みない
7. `cve_triage.lock.yaml` を generator script で更新し、CI の xref check が green であることを確認する
8. security 担当者 2 名の sign-off を取得し PR を merge する

## 関連適合仕様 / 関連 OSS

- build_provenance 適合仕様: [../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)
- OSS ライフサイクル適合仕様: [../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- 関連 OSS: Syft / Trivy / Grype / cargo-deny / Renovate / OSV API / Sigstore Cosign / Rekor

## 期待結果 / 観測指標

- 全 CVE が `cve_triage.lock.yaml` に `patch_due_at` または `accepted_risk` として記録済み
- 前月の `patch_due_at` を超過した CVE がゼロ（超過は `release_gate.lock.yaml` の ship blocker）
- `accepted_risk` count が cap（10 件）以下
- CI の CVE check が green

## 失敗時の挙動 / escalation

- **patch_due_at 超過（SLO 違反）**: 該当 OSS の `dry_run.lock.yaml`（移行 Pair）が有効な場合は paired OSS への緊急 migration を tier1 担当者に依頼。Mattermost `#security` に即時報告（SLA: 4 時間以内）、L2 escalation（tech lead）。postmortem は 3 営業日以内
- **known exploit CVE が供給される**: 即時シナリオ 07（CRITICAL incident 対応）を発火。月次トリアージを中断し、07 の 6 phase playbook を優先する
- **supply chain 疑義（cosign verify fail）**: 即時シナリオ 08 を発火。当該 image を Harbor で quarantine し、cluster 上の running instance を drain する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — SBOM / cosign / Rekor の structural spec
- [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) — 8 signal による移行 trigger 判定
- [tier1 担当者シナリオ: SBOM CVE 月次トリアージ](../01_tier1担当者シナリオ/10_SBOM_CVE_月次トリアージ.md) — tier1 視点の CVE 対応（security との分担）
