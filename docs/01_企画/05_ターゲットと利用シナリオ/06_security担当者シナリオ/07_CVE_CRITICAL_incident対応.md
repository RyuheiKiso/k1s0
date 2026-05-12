---
id: plan.security.scenario_cve_critical_incident
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# CVE CRITICAL incident 対応

## 一文方針

CVE severity が high / critical または known exploit が観測された時に `v1_supply_chain_compromise` incident class の 6 phase playbook を発火し、affected package の Harbor quarantine → rebuild → cosign 再署名 → `cve_triage.lock.yaml` の close_kind 記録 → postmortem PR まで patch SLO（high 14d）内で完結させる。

## Trigger（発火条件）

Trivy / Grype / OSV API feed が CVSS ≥ 7.5 の CVE を検知した時、または NVD / vendor advisory で当該 CVE の known exploit が公開された時（CVSS 閾値未満でも known exploit あれば強制 high 扱い）。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（年間 2-5 件程度）。典型きっかけ: 「openssl-sys の CVSS 9.1 CVE が NVD に公開され、tier1 Rust binary が direct depend。Trivy が CI で検知し Mattermost `#security-incident` に自動 page が来た」

## 主役 / 関与者

- 主役: security 担当者（シニア級、IR commander に自動 assign）
- 関与: tier1 担当者（Rust / Go dep の patch 適用、rebuild、cargo-deny 確認）
- 関与: infra 担当者（Harbor quarantine 操作、Kyverno admission policy の即時 enforcement 確認）
- 関与: ops 担当者（SLO error_budget 消費確認、escalation engine 起動）

## 前提

- 7 incident class × 6 phase playbook が build artifact（`scenario_catalog.lock.yaml`）として存在し、`v1_supply_chain_compromise` の playbook pointer が有効
- `escalation_policy.lock.yaml` の IR commander = security 担当者の assign が自動化済み
- Harbor の quarantine 機能が有効かつ `block-on-unsigned-image` Kyverno policy が active
- `cve_triage.lock.yaml` が build artifact として存在

## 流れ

1. **検知（phase 1）**: Trivy / OSV alert が Mattermost `#security-incident` に到達する。security 担当者が IR commander として自動 assign される。scope assessment: affected package 名 / affected binary / affected cluster namespace を特定する
2. **triage（phase 2）**: CVSS score + known exploit status を NVD / vendor advisory で確定する。`cve_triage.lock.yaml` に `status=in_triage` / `patch_due_at` を記録する。affected tenant / asset を ClickHouse audit query で初期推定する
3. **contain（phase 3）**:
   - 該当 image を Harbor で即時 quarantine する（Harbor UI または `harbor image quarantine --image <digest>`）
   - Kyverno `block-on-unsigned-image` policy が当該 image digest を拒否することを確認する
   - running instance が cluster 上に残っている場合は `kubectl drain` で pod を evacuate する
4. **eradicate（phase 4）**:
   - tier1 担当者が affected package を patch 済み version に更新（`cargo update openssl-sys --precise <patched-version>`）
   - rebuild: Tekton pipeline `security-rebuild-$(date +%Y%m%d)` を起動し、hermetic sandbox（`RUN --network=none`）で再 build する
   - 再 build 後、cosign で再署名する（`cosign sign --key openbao://transit/signing-key <new-image-digest>`）
   - Rekor inclusion proof を取得し、`build_provenance 適合仕様` の `v1_runtime_image_release` に該当する全 artifact が signature + attestation を持つことを確認する
5. **recover（phase 5）**: Harbor の quarantine を解除し、Argo CD で staging → production への rollout を承認する。Prometheus healthcheck / SLO 復帰を確認する
6. **postmortem（phase 6）**: `cve_triage.lock.yaml` の `close_kind=fixed_in_code` と `closed_at` を記録する。postmortem PR を起票し、action item（dep update automation の強化等）を GitHub Issue 化する
7. `release_gate.lock.yaml` の cve_overdue_count が 0 に戻ったことを確認してクローズ

## 関連適合仕様 / 関連 OSS

- build_provenance 適合仕様: [../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)
- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- 関連 OSS: Trivy / Grype / OSV API / Harbor / Cosign / Sigstore Rekor / Tekton / Kyverno / Argo CD

## 期待結果 / 観測指標

- patch SLO（high 14d）以内に `close_kind=fixed_in_code` または `close_kind=fixed_in_spec` で close
- 再 build image が cosign 署名 + Rekor inclusion proof を持ち、Kyverno admission が通過
- Harbor の quarantined image が全 cluster から evacuate され running instance がゼロ
- postmortem PR が merge 済み、action item 全件 GitHub Issue 化済み

## 失敗時の挙動 / escalation

- **patch SLO（high 14d）を超過しそう**: 7 日時点で tier1 担当者が patch 未完の場合、L3 escalation（tech lead）を起動し、`dry_run.lock.yaml` が有効な paired OSS への緊急 migration を検討する。SLO 超過は release_gate に記録され ship blocker になる
- **rebuild が hermetic 検証（bit_for_bit_ci_verified）を通過しない**: Tekton + GitHub Actions + Bazel の 3 builder で sha256 が不一致。Nix flake pin を再確認し、non-determinism の origin を特定する。release_gate の `build_determinism_failure` count が上昇する前に修正する
- **known exploit CVE で即時対応が必要（当日 patch が存在しない）**: `close_kind=accepted_as_bug` で一時的に登録し、`close_due_at=today+14d` を設定。`accepted_as_bug` cap（10 件）との残余を Mattermost で security 担当者全員に通知する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — rebuild / cosign / Rekor の structural spec
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — `v1_supply_chain_compromise` class の contain action 詳細
- [security 担当者シナリオ: cosign admission 拒否対応](./08_cosign_supply_chain_admission拒否対応.md) — 本シナリオの eradicate 後に実施する admission 通過確認
