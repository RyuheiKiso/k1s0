---
id: detail.cross_kek.kek_shamir_distribution
axis: cross_kek
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - detail.tier1.key_management_conformance
  - detail.data.preservation_conformance
  - detail.security.threat_model_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_program_correctness_proof
related_axes:
  - tier1
  - data
  - security
  - infra
---

# KEK Shamir Secret Sharing 分散保管 適合仕様（v1）

## 位置づけ
- 05_鍵管理適合仕様 の `v1_data_kek` 行 / backend `hsm_pkcs11_shamir_distributed_v1` / `dr_cross_region_failover` scenario を物理達成するための protocol / parameter / 復元 ceremony / drill scenario を一箇所に固定
- 09_data PII 専用 cluster / 14_データ保全適合仕様 / infra クラスタ構成方針 / 10_security 強制機構 / test 故障注入検証方針 と双方向 lock
- OpenBao OSS が Vault Enterprise の Performance Replica / DR Replica 機能を持たない物理制約に対する、企画固有の cross-region 鍵冗長設計

## 設計原則
- KEK material は単一 region の単一 HSM partition に物理存在しない。攻撃者が単一 region の OpenBao instance + HSM partition + share custodian を完全に compromise しても、KEK 全体を復元できない
- 復元は M-of-N 閾値 ceremony によってのみ成立する。M 未満の region 集合からの復元 API 呼出は admission policy で block + audit_event emit（`block-single-region-kek-reconstruction`）
- share rotation は全 region atomic。region 単独の rotation は threshold 整合性を壊すため禁止
- GDPR crypto-shred との互換性: 全 share の同期 destroy で wrapped DEK 不可逆失効を property test で検証

## M-of-N parameter（既定値）
- N（share total、region 数）: 5
- M（share threshold）: 3
- 設計根拠:
  - 2 region 完全断まで KEK 物理復元継続可能（5 - 3 = 2 region tolerance）
  - 3 region 以上の同時断は意図的に「鍵を取れない」状態とし、不正アクセス防御を優先
  - region 数 5 は 14_データ保全適合仕様 `v1_global_replicated` の cell_partitioning と無関係に独立して選定
- parameter は `instance_attributes_required` として 14_データ保全適合仕様 の `v1_cross_region_replicated` entry に `kek_shamir_threshold_m` / `kek_shamir_total_n` を持つ（M ≥ 3、N ≥ M + 2 の minimum constraint を Conftest で物理 enforce）

## share custodian role
- 各 region の share に対し 2 名の custodian を割当てる（break-glass two-person rule）
- custodian は region の OpenBao admin role を持ち、share 取得 API を呼び出せる唯一の role
- custodian role の bootstrap は Backstage Software Template で生成、初回 share 配布も Software Template で記録（audit_event emit 必須）
- custodian rotation: custodian の人事異動 / 退職時は所属 region の share を再生成（threshold rotation）し、新 custodian に再配布。残 N-1 region は影響なし

## 復元 ceremony runbook（Backstage Software Template として配布）

### trigger 条件
- 1 region 以上の OpenBao + HSM 完全喪失（`v1_kek_share_threshold_fault` drill または real incident）
- KEK rotation 時の旧 KEK 全 share zeroize 前 verify
- 監査要請時の KEK 物理復元 demonstration

### 手順（Software Template が procedural に enforce）
1. ceremony commander 1 名 + observer 2 名以上を assign
2. M region から各 1 名の custodian を online 召集（break-glass token 発行、audit_event emit）
3. 各 custodian が region の OpenBao API から自 region 保管の share を air-gapped ceremony host に転送（TLS 1.3 + 短期 OIDC token + ceremony commander 署名）
4. air-gapped host 上で Shamir reconstruction（vendored Rust crate `shamir-secret-sharing`、08_OSS ライフサイクル適合仕様 の `v1_l1plus_primary` 相当の crate を inventory 登録）で KEK 復元
5. 復元 KEK で wrapped DEK を unwrap し、業務継続 or rotation 操作を実施
6. ceremony 終了後、air-gapped host のメモリを物理 electrical reset、host 自体を physical destruction。全 audit_event を 14_データ保全適合仕様 `v1_cross_region_replicated` の audit ClickHouse SoR に到達確認

### ceremony 時間
- trigger 〜 復元成功までの目標 30 分以内（14_データ保全適合仕様 `v1_cross_region_replicated` `restore_window` 300 秒の cap には収まらないため、KEK 復元完了までは write block を許容する設計。drill で実測値を定期更新）

## drill scenario（19_検証規律適合仕様 と双方向 lock）
- `drill_class`: `v1_zone_gameday`（local drill、month 1 回）+ `v1_region_gameday`（quarter 1 回）
- `blast_radius`: controlled（drill region は production の非業務時間帯に隔離した copy）
- `target_layer`: data01 + security15 + infra01
- `drill_cadence_days`: 30（`v1_kek_share_threshold_fault` fault_class の cadence と同調）
- 検証 property（property test として実装）:
  - p1: M region 健全時、閾値復元が成立し業務継続（wrapped DEK の unwrap 成功）
  - p2: M-1 以下の region 健全時、復元不能であることが正確に admission block される（誤って成功しない）
  - p3: share rotation 全 region atomic で完了（rotation 中に閾値復元が一貫性を持つ）
  - p4: crypto-shred 互換: 全 share destroy 後、wrapped DEK が物理的に unwrap 不能であることを 7 日間連続 verify
  - p5: 単一 region のみで復元 API を呼ぶと `block-single-region-kek-reconstruction` admission が block + audit_event emit

## vendor diversity
- 各 region の HSM partition は別 vendor を許容する（`vendor_diversity: allowed_per_region`）
- 1 vendor の firmware 脆弱性が全 region 同時失効を起こさない冗長設計
- 採用候補 vendor:
  - region A: NetHSM（OSS、Nitrokey）
  - region B: Thales Luna Network HSM（vendor、PKCS#11 v3）
  - region C: YubiHSM 2（vendor、PKCS#11 v3）
  - region D: Entrust nShield（vendor、PKCS#11 v3）
  - region E: SoftHSM2（dev / drill 専用、production 不可）
- production 必須要件: FIPS 140-3 Level 3 認証（SoftHSM2 は production に使わない、drill 用のみ）

## Argo Workflows / GitOps integration
- share 配布 / rotation / ceremony は Argo Workflows CR で declarative に表現
- Argo CD は各 region の OpenBao policy / share 配布 manifest を read-only 同期。manifest 改変は git PR レビュー必須（break-glass two-person rule の audit chain と整合）
- ceremony commander の break-glass token 発行も Argo Events 経由で audit_event 化

## 5 層 defense-in-depth
- 層 A: compile 時。share 配布 manifest が manifest 形式 check で blocking。N / M parameter の constraint（M ≥ 3、N ≥ M + 2）が Conftest で物理 enforce
- 層 B: lint。本ファイルが他軸（11、14_データ保全、PII cluster、infra クラスタ構成、security 強制機構、test 故障注入）からの参照経路を持つことを grep で確認
- 層 C: 02_移行 Pair 適合仕様 dry_run scenarios と接続。share 配布の dry_run cadence を年次（11 `v1_data_kek` の rotation cadence と同期）で実行
- 層 D: runtime monitoring。OpenBao audit log + HSM audit event の連続性を 09 audit signal sub-class `kek.share.access` として SoR に到達確認
- 層 E: admission。`block-single-region-kek-reconstruction` policy が最終 safety net

## ship blocker 解除条件（1.0.0）
- SB-1: share 生成・配布 ceremony script（Backstage Software Template）が full automated path で実行可能（手動 step ゼロ、ceremony commander 署名は除く）
- SB-2: `v1_kek_share_threshold_fault` drill が property p1〜p5 全て green（chaos drill cadence 30 日最初の 1 cycle 以上を実走）
- SB-3: audit chain で share location / access の cryptographic 完全性を 30 日連続 verify、divergence ゼロ
- SB-4: crypto-shred 互換 property test green（全 share destroy で wrapped DEK 不可読化）
- SB-5: 本仕様が 11、14_データ保全、PII cluster、infra クラスタ構成、security 強制機構、test 故障注入から参照されること（層 B grep）

## 残リスク
- ceremony 中のヒューマン誤操作: break-glass two-person rule + Backstage Software Template の procedural enforcement で吸収。誤った復元試行は層 E admission で block
- share 1 個の窃取 + 残 region の M-1 share の同時 compromise: M region の同時 compromise が成立した時点で「攻撃者は十分以上に compromise している」前提となるため、本設計は M region 同時 compromise を threat model 範囲外。これに該当する場合は incident response + 全 KEK 即時 rotation + tenant 通知の経路に escalate
- vendor lock: `vendor_diversity` を inventory で強制するが、特定 vendor のサプライチェーン中断時は drill cadence で早期検出（per-region replacement drill を年次実施）

## 関連参照
- [鍵管理適合仕様](../01_適合仕様/05_鍵管理適合仕様.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [PII 専用クラスタ](08_PII_dedicated_cluster.md)
- [脅威モデル適合仕様](../01_適合仕様/15_脅威モデル適合仕様.md)
- [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md)
