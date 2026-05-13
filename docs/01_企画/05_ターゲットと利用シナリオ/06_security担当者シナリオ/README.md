---
id: plan.security.scenario_index
axis: security
phase: plan
kind: index
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# security 担当者シナリオ index

## 一文方針

security 担当者（シニア級）が日常的に踏む 14 シナリオを 1 ファイル 1 シナリオで列挙する。[5⁴ = 625 cell threat catalog](../../../03_概要設計/07_security設計方針/01_脅威モデル方針.md) の coverage 維持・[8 secret class](../../../03_概要設計/07_security設計方針/03_秘密管理方針.md) の lifecycle・[7 drill class](../../../03_概要設計/07_security設計方針/08_セキュリティ訓練方針.md) の cadence・[7 incident class × 6 phase playbook](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) の全走を、全 19 軸の defense-in-depth 層 D / E / F（13 cross-cutting 適合仕様を含む）の cross-axis bind として提供する。

## 現状業務での痛み

- 脅威モデルを Excel / 文書で管理すると、新機能追加時の cell 更新漏れが気付かれないまま蓄積する
- secret rotation の cadence を人手管理すると、rotation 未実施 secret が本番に残存する
- インシデント対応手順が人の記憶に依存し、深夜の on-call で手順誤りが起きる
- 監査証跡の改竄可否を保証できないと、外部監査・法的手続きで証拠能力が失われる
- build artifact の出所保証が署名なしだと、supply chain attack を検出できない

## k1s0 でこう変わる

- 625 cell threat catalog を `threat_model.lock.yaml` で管理し、cell 更新漏れを CI が物理検出する
- secret rotation cadence 超過を Kyverno が本番 deploy を物理停止し、rotation 未実施 secret を構造で排除する
- 7 incident class × 6 phase playbook を lock yaml で実体化し、深夜対応でも手順を物理的に追跡できる
- audit hash chain + WORM Object Lock + RFC 3161 + Sigstore Rekor の四層 immutability で改竄を物理不可能にする
- SLSA L3+ / cosign / SBOM で build provenance を 5 class 物理 enforce し、supply chain attack を防御する

## 担当者プロフィール

security 担当者はシニア級エンジニアを前提とし、脅威モデリング / KEK shamir custodian / SPIFFE-SVID / SLSA L3+ / Cosign / OpenBao の運用経験を持つ。詳細な要件は層別エンジニア要件を参照。

- 参照: [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)

- 級: シニア
- 想定人数: 2-3 名（プラットフォーム運営者 5-8 名と協調）
- 必須スキル: 5⁴ cell threat model / KEK shamir / OAuth 2.1 + DPoP / OWASP / penetration testing / SPIFFE / audit hash chain / WebAuthn / OpenBao
- 責務: 625 cell coverage 維持 / 8 secret class lifecycle / 7 drill class cadence / 7 incident class × 6 phase playbook / 5 PII class cross-axis bind / 5 build_provenance_class × 5 phase enforcement

> **注記**: 「全 19 軸の defense-in-depth 層 D/E（13 cross-cutting 適合仕様を含む）」の「13」は `docs/04_詳細設計/03_クロスカッティング適合仕様/` 配下の適合仕様**ファイル数**です。tier3 の「13 層強制機構」（tier3 固有の enforcement 13 層）とは別概念です。

## security 軸の主要分類

### 5⁴ = 625 cell threat catalog（5 actor × 5 capability × 5 surface × 5 asset）

| 軸 | class |
|---|---|
| actor（5） | `v1_external_unauth` / `v1_external_auth` / `v1_insider_application` / `v1_insider_operator` / `v1_supply_chain` |
| capability（5） | `v1_read` / `v1_write` / `v1_delete` / `v1_dos` / `v1_lateral` |
| surface（5） | `v1_north_south` / `v1_east_west` / `v1_control_plane` / `v1_persistence` / `v1_supply_chain_surface` |
| asset（5） | `v1_pii` / `v1_business_data` / `v1_audit_log` / `v1_credential` / `v1_availability` |

全 cell は `threat_model.lock.yaml` で管理。unreachable は `explicit_unreachable=true` marking 必須。`explicit_unreachable` rate の上昇は月次レビューで抑制。

詳細: [脅威モデル適合仕様](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)

### 5 mitigation_class

| class | 物理機構例 |
|---|---|
| `v1_authn_authz` | Envoy `jwt_authn` / Keycloak token introspection / DPoP / WebAuthn |
| `v1_crypto` | HSM PKCS#11 / WebCrypto non-extractable / KEK shamir M-of-N |
| `v1_isolation` | RLS FORCE / NetworkPolicy / Kyverno / PII 専用クラスタ |
| `v1_admission_block` | cosign verify AND-gate / Harbor admission / Kyverno 25+ policy |
| `v1_audit_detect` | audit hash chain / WORM Object Lock / RFC 3161 timestamp / Sigstore Rekor |

### 8 secret class lifecycle

| class | lifetime | rotation cadence | store |
|---|---|---|---|
| `v1_authn_token` | ≤ 1 h（refresh ≤ 24 h） | lifetime 期限（自動） | OpenBao + External Secrets |
| `v1_api_key` | ≤ 90 日 | 90 日強制 | OpenBao kv2 + External Secrets |
| `v1_db_password` | ≤ 24 h | 24 h（自動 lease 更新） | OpenBao Database secret engine |
| `v1_kek` | （tier1/11 決定） | 90 日標準 | OpenBao Transit + HSM-backed |
| `v1_dek` | （tier1/11 決定） | — | data store 内（KEK wrap 済） |
| `v1_tls_private_key` | ≤ 90 日 | cert-manager 自動更新 | K8s Secret + OpenBao root CA |
| `v1_signing_key` | 永続（rotation 365 日） | 365 日強制 | OpenBao Transit / HSM-backed |
| `v1_ssh_key` | ≤ 1 h | lifetime 期限（自動） | OpenBao SSH secret engine |

rotation cadence 超過で Kyverno `block-on-rotation-overdue` が本番 deploy を物理停止。

### 7 drill class cadence

| drill class | cadence | success criteria |
|---|---|---|
| `v1_red_team_table_top` | 月次（rotation） | playbook step 全踏破 + postmortem PR |
| `v1_chaos_failure_drill` | 月次（class 別） | SLO budget 消費 ≤ target |
| `v1_restore_drill` | preservation_class 別（14-180 日） | RPO/RTO target 内 |
| `v1_secret_compromise_drill` | 四半期 | revoke → re-issue ≤ 30 分 |
| `v1_supply_chain_drill` | 四半期 | cosign key rotation + quarantine + cleanup ≤ 60 分 |
| `v1_red_team_live` | 半年 | escalation detection rate ≥ target |
| `v1_external_pen_test` | 年次 | 全発見が CVE 風 catalog 化 + patch SLO 内修正 |

cadence 違反は Kyverno `block-on-drill-overdue` で admission block。

### 7 incident class × 6 phase

| class | 内容 |
|---|---|
| `v1_secret_exposure` | secret の git / image / log への漏洩 |
| `v1_data_exfiltration` | PII / business_data の不正 export 疑義 |
| `v1_data_tampering` | DB / topic / audit 改竄 |
| `v1_availability_loss` | SLO 違反級 outage |
| `v1_supply_chain_compromise` | CI / image / dep の compromise 疑義 |
| `v1_privilege_escalation` | cluster-admin / DB superuser 不正取得疑義 |
| `v1_audit_chain_break` | audit_event hash chain divergence |

6 phase: 検知 → triage → contain → eradicate → recover → postmortem（**postmortem PR merge が次 release の物理 prerequisite**）。

### 5 PII class

| class | 代表例 |
|---|---|
| `v1_pii_basic` | 氏名 / 住所 / 電話 / 端末 ID |
| `v1_pii_special` | 人種 / 信条 / 病歴 / 犯罪歴 |
| `v1_pii_finance` | 口座 / クレカ番号 |
| `v1_pii_credential` | password hash / OTP seed / KEK ID |
| `v1_pii_location` | GPS 軌跡 / 現在位置（工場内位置含む） |

PII column は `pii_classification.lock.yaml` で管理。6 軸 cross-axis bind（鍵管理 / schema breaking / RLS FORCE / preservation_class / client 暗号化 / 監査）が欠落すると CI fail。

### 5 build_provenance_class × 5 phase

| class | 用途 |
|---|---|
| `v1_runtime_image_release` | tier1/2 server image / production deploy 主経路 |
| `v1_library_sdk_release` | SDK 9 言語 lockstep release |
| `v1_cli_or_oci_artifact_release` | CLI tool / OCI artifact 単体 |
| `v1_thin_business_api_artifact` | codegen artifact / Buf generated proto |
| `v1_dev_or_canary_artifact` | dev / canary build |

5 phase: build → attest → verify → publish → monitor。全 class が 5 enforcement orchestrator（tekton_chains_attestor / bazel_nix_hermetic_runner / reproducibility_consensus_verifier / cosign_rekor_witness_chain / kyverno_admission_verifier）に bind 済みであることを CI で物理確認。

詳細: [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)

## 重要原則

- **「現実には起こりにくいから登録しない」判断を採らない**: 625 cell は cross-product の完全列挙。unreachable は `explicit_unreachable=true` marking で管理し、登録は省略しない
- **mitigation は物理機構の class であって文章では成立しない**: 5 mitigation_class のいずれかが cosign signed AND-gate + audit chain で build artifact に物理 bind されていることが条件
- **severity 別 close_due_at**: counter-example / CVE は high 14d / medium 30d / low 90d、`accepted_as_bug` cap ≤ 10 件、cap 越えは破壊的変更扱い
- **audit は immutability 三重化**: WORM Object Lock + audit hash chain（pgaudit / ingest gap heartbeat）+ RFC 3161 trusted timestamp + Sigstore Rekor transparency log の四層で改竄を物理拒否

## シナリオ一覧

| # | シナリオ名 | trigger 概要 | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|---|---|---|---|---|
| 01 | [threat_model 625 cell 月次レビュー](01_threat_model_625cell月次レビュー.md) | 月次 cycle 到来または unreachable marking drift 検知時 | 月次 | 脅威モデル適合仕様 | [周期] |
| 02 | [SBOM / CVE / supply chain 月次トリアージ](02_SBOM_CVE_supply_chain月次トリアージ.md) | 月次 cycle 到来または CVE feed 流入時 | 月次 | 脅威モデル適合仕様 / build_provenance 適合仕様 | [周期] |
| 03 | [red_team_table_top 実施](03_red_team_table_top.md) | 月次 rotation で担当 actor class が決定した時 | 月次 | security 強制機構 | [周期] |
| 04 | [red_team_live drill 実施](04_red_team_live_drill.md) | 半年 cycle 到来時 / infra chaos drill green 確認後 | 半年 | security 強制機構 / セキュリティ訓練 | [周期] |
| 05 | [chaos_failure_drill / atomic 三表書込 P1-P4 検証](05_chaos_failure_drill.md) | 月次 cycle 到来時 | 月次 | 脅威モデル適合仕様 / データ保全適合仕様 | [周期] |
| 06 | [secret_compromise_drill / KEK shamir ceremony](06_secret_compromise_drill_KEK_shamir_ceremony.md) | 四半期 ceremony 到来時 / compromise 疑い緊急発火時 | 四半期 + 緊急 | 鍵管理適合仕様 / security 強制機構 | [周期]+[緊急] |
| 07 | [CVE CRITICAL incident 対応](07_CVE_CRITICAL_incident対応.md) | CVE high/critical 発火または exploit 観測時 | イベント駆動 | build_provenance 適合仕様 / 脅威モデル適合仕様 | [緊急] |
| 08 | [cosign / supply chain admission 拒否対応](08_cosign_supply_chain_admission拒否対応.md) | Harbor admission 拒否または unsigned image 検知時 | イベント駆動 | build_provenance 適合仕様 / security 強制機構 | [緊急] |
| 09 | [audit hash chain 改竄検知 → 外部公証](09_audit_hash_chain改竄検知_外部公証.md) | hash chain verification fail または ingest gap heartbeat miss 時 | イベント駆動 | 脅威モデル適合仕様 / security 強制機構 | [緊急] |
| 10 | [break-glass emergency_step_up 事後レビュー](10_break_glass_emergency_step_up_post_review.md) | `v1_emergency_step_up` 実行記録の audit 到着時 | イベント駆動 | 認証適合仕様 / security 強制機構 | [計画]+[緊急] |
| 11 | [PII 種別追加 / 5 PII class 運用変更](11_PII種別追加_5_PII_class運用変更.md) | 新規 PII 種別追加または専用クラスタ設定変更が必要になった時 | 不定期 | 脅威モデル適合仕様 / データ保全適合仕様 | [計画] |
| 12 | [WebAuthn step_up / federation 政策変更](12_WebAuthn_step_up_federation政策変更.md) | IdP federation 追加または step_up 要求境界変更が必要になった時 | 不定期 | 認証適合仕様 / security 強制機構 | [計画] |
| 13 | [build_provenance_class 追加 / 5 phase enforcement 変更](13_build_provenance_class追加_5_phase_enforcement変更.md) | build_provenance_class 追加または SLSA L3+ 経路変更が必要になった時 | 不定期 | build_provenance 適合仕様 | [計画] |
| 14 | [data breach / privacy incident response](14_data_breach_privacy_incident_response.md) | PII 漏洩疑いまたは breach notification 要件が発火した時 | 非計画的（年間 0-1 件） | 脅威モデル適合仕様 / データ保全適合仕様 | [緊急] |

## 新規参画者向けオンボーディング

シニア級エンジニアとして着任した際の推奨学習順序:

- **Day 1-3**: README 全体読了 → 01（625 cell レビュー）→ 02（SBOM/CVE トリアージ）を通読（最頻周期業務）
- **Day 4-7**: 03（red_team_table_top）の次回演習に観察役として参加し、playbook の踏み方を身体で覚える
- **Week 2**: 09（audit hash chain）と 10（break-glass post-review）を staging 環境で手順通りに演習する
- **Week 3-4**: 04（live drill）→ 05（chaos）→ 06（secret compromise drill）の演習ロールオーバーに参加
- **Month 2 以降**: 07（CVE CRITICAL）→ 08（cosign 拒否）の incident simulation を staging で実施。11-13（計画変更系）は発生時に先輩担当者とペア実施
- **Month 3 以降**: 14（data breach）の playbook シミュレーションを法務 / DPO 担当者も含めた cross-functional で実施。06 の KEK shamir ceremony に custodian 補佐として参加

## シナリオ間の依存関係

- **05（chaos）→ 06（secret compromise）の順序確認**: chaos drill が green でなければ secret_compromise_drill の success criteria が不確かになる。`drill_progress.lock.yaml` の chaos green を確認してから compromise drill に進む。
- **06（KEK shamir ceremony）→ data-05（DEK rotation）の連携**: KEK の再構築または rotation が完了した後に data 担当者が DEK rotation を実施する。KEK rotation と DEK rotation の並行実施は禁止。
- **07（CRITICAL CVE）→ 08（cosign admission 拒否）の連動**: CVE 起因で image を rebuild する場合、rebuild → 再署名 → cosign verify → Harbor admission 通過まで一連で扱う。08 の手順を 07 の eradicate phase に組み込む。
- **09（hash chain 改竄）→ 14（data breach）の発火**: hash chain divergence が PII 漏洩経路の audit_event 欠落である場合、`v1_data_exfiltration` または `v1_data_tampering` として 14 を発火する。
- **10（break-glass post-review）と ops 軸の incident response の二者協調**: break-glass 実行は ops 担当者が主導して IR playbook を走らせる。事後 audit 確認（OpenBao audit log + audit hash chain の一貫性検証）は security 担当者が主導。二者の sign-off が揃うまで postmortem は close 不可。
- **11（PII 種別追加）→ data-07（PII 専用クラスタ運用）の順序**: 新規 PII 種別追加が `pii_classification.lock.yaml` で決定した後に、data 担当者がクラスタ設定変更を実施する。security と data の同時 PR は禁止（依存方向 lock）。
- **13（build_provenance_class 変更）→ tier1-09（supply chain 障害対応）/ tier1-10（SBOM CVE 月次トリアージ）の連動**: build_provenance_class を追加または変更した場合、tier1 担当者が OSS lifecycle signal / SBOM 差分 check の対象 class を更新する。security と tier1 の共同 PR として単一 review cycle で完結させる。

## 関連参照

- [security 設計方針](../../../03_概要設計/07_security設計方針/README.md) — 5 enum 軸・8 サブ方針の設計原則
- [脅威モデル適合仕様](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md) — 625 cell の正典定義と mitigation pointer binding
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — 5 build_provenance_class × 5 phase の structural spec
- [security 強制機構](../../../04_詳細設計/02_強制機構/06_security強制機構.md) — 多層 enforcement の物理機構一覧
- [audit_ingest_gap_monitor](../../../04_詳細設計/03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) — hash chain + ingest gap heartbeat の二重防御
- [ターゲットと利用シナリオ index](../README.md) — 全担当者シナリオの横断構成と各軸シナリオ index へのリンク
