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

> 朝 8 時半、SBOM CVE scan の CI notification が Mattermost `#security-incident` に届く。security 担当者（シニア級）が CVSS スコアと SBOM diff を手元の dashboard で確認し、月次トリアージの開始を tier1 担当者に通知する。Trivy が `openssl-sys` の CVSS 9.1 CVE を検知しており、patch SLO（high 14d）のカウントダウンが始まっている。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: SBOM の CVE を月次トリアージし severity 別 patch SLO を management して ship blocker を解消する

## 現状業務での痛み

- CVE をスプレッドシート管理しており、パッチ優先度の判断が属人的で担当者によって対応速度が異なる
- patch SLO の追跡が手動で、SLO 超過が本番障害として表れるまで気付かれない
- accepted_risk の上限管理がなく、リスク受容が際限なく蓄積する

## k1s0 でこう変わる

- cve_triage.lock.yaml が patch SLO と accepted_risk を管理し、SLO 超過が CI の ship blocker として自動検知される
- Trivy / Grype の double-bound scan が CVSS スコアを自動確定し、優先度判断の属人化がなくなる
- accepted_risk cap（10 件）が release_gate.lock.yaml で強制管理され、リスク受容の上限が物理的に制限される

## Trigger（発火条件）

月次 cycle の到来（毎月第 2 営業日）、または Renovate / dependabot / Trivy の CI alert が severity high 以上の CVE を検知した時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次（緊急は随時）。典型きっかけ: 「Renovate が `openssl-sys 0.9.92` に CVE-2024-XXXX（CVSS 9.1）を検知した。tier1 Rust binary がこの crate に直接依存している」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: tier1 担当者（Rust crate / Go module の依存グラフ変更 + cargo-deny / govulncheck 確認）
- 関与: infra 担当者（base image / K8s cluster component の CVE 対応）
- 関与: tier2 / tier3 担当者（各言語 SDK の dep 更新）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security）| シニア | 本社 IT 室 / リモート | Mattermost #security-incident / cve_triage.lock.yaml | CVSS 確定・patch SLO 設定・accepted_risk 記録・dual sign-off |
| 関与（tier1）| シニア〜ミドル | 本社 IT 室 / リモート | GitHub Renovate PR list | Rust / Go dep patch 適用・cargo-deny 確認・rebuild |
| 関与（infra）| シニア〜ミドル | 本社 IT 室 / リモート | Harbor scan dashboard | base image / K8s component の CVE 対応・quarantine 操作 |
| 関与（tier2 / tier3）| ミドル | 本社 IT 室 / リモート | Renovate PR list | 各言語 SDK dep 更新 PR の review / merge |

## 個人 KPI / 達成感

- patch SLO 達成率（high 14d / medium 30d / low 90d）を cve_triage.lock.yaml で定量確認でき、セキュリティ対応品質の達成感を得られる
- accepted_risk count の削減を数値で確認でき、リスク管理改善の進捗を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（SBOM diff 確認 1h + CVE 照合 2h + patch SLO 設定 2h + PR review 1h）
- 関与人数: 4〜5 名（security 担当者・tier1・infra・tier2 / tier3 担当者）
- コスト感: 低〜中。Renovate が自動 PR を生成するため patch 作業コストが大幅削減される

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | SBOM diff と CVE alert を統合し CVSS score を確定 | `SBOM diff 確認 / CVE N 件検出 / CVSS 確定` |
| 2h | security 担当者 | patch 可能 CVE の SLO を設定し accepted_risk を cve_triage.lock.yaml に記録 | `patch SLO 設定完了 / accepted_risk N 件記録` |
| 4h | security 担当者 | generator で lock.yaml を更新し CI xref check green を確認して PR 提出 | `CI green / PR #NNN 提出` |
| 1d | dual reviewer (security 2名) | patch SLO と accepted_risk 根拠を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務の依存 OSS が CVE の対象になり得るが、特に影響度が高い 3 業務:

- **SCADA テレメトリ収集**: テレメトリ収集パイプラインの OSS（Kafka 系 / Rust binary）は supply chain attack の入口になりやすく、CVSS high / critical の CVE は制御系データの完全性 / 可用性に直結する。
- **計量装置連続データ**: 計量データ収集系の OSS に CVE がある場合、改竄検知の信頼性が揺らぎ、業務データの integrity 保証が失われる risk がある。
- **FA 生産指示**: 制御系コンポーネントの CVE は設備誤動作につながる可能性があり、patch SLO を最優先で適用する必要がある。

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

## 失敗パターン (anti-pattern)

- CVE をスプレッドシート管理: lock.yaml を使わない管理は CI の ship blocker 自動検知が機能しない
- accepted_risk cap 無視: cap を超えた accepted_risk は release_gate.lock.yaml が ship blocker として阻止する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — SBOM / cosign / Rekor の structural spec
- [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) — 8 signal による移行 trigger 判定
- [tier1 担当者シナリオ: SBOM CVE 月次トリアージ](../01_tier1担当者シナリオ/10_SBOM_CVE_月次トリアージ.md) — tier1 視点の CVE 対応（security との分担）
