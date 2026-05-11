---
id: detail.tier1.oss_lifecycle_conformance
axis: tier1
phase: detail
kind: conformance_spec
status: draft
depends_on:
  - arch.tier1.tier1_index
  - arch.tier1.libraries
  - arch.tier1.feature_categories
  - req.overview.oss_catalog
  - detail.tier1.migration_pair_conformance
  - detail.tier1.tier1_enforcement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
lock_artifacts:
  - oss_inventory.lock.yaml
---

# tier1 OSS ライフサイクル適合仕様（v1）

## 一文定義
- OSS ライフサイクル適合仕様は、6 lifecycle_class × 4 dimension（adoption_status / health_check_cadence / migration_trigger_set / response_action）の cross-product として宣言される OSS catalog の各 instance に対し、8 signal（license_change / eol_announced / cve_backlog_threshold / maintainer_turnover / fork_event / conformance_drift / major_up / spec_drift）の発火条件を機械可読に固定し、Kyverno admission policy + Cosign 署名検証で物理 enforce する meta-axis である。

## 位置づけ
- 02_採用 OSS 一覧 で確定された全 OSS、04_提供機能カテゴリ L1+ / L2\* / 予約カテゴリ、02_Library L1+ 移行コミットメント / L2\* 同族保証、02_移行 Pair 適合仕様 primary pair、02_採用 OSS 一覧 L2\* 同族（Apicurio↔Karapace、flagd↔GFF、Parca↔Pyroscope）、Companion runtime（Keycloak / Backstage / Argo CD / Apicurio）が散在的に宣言する OSS 採用区分 / health check cadence / 移行発動 trigger / response policy を、機械可読な単一の真として固定
- 02_移行 Pair 適合仕様 が「移行 pair の機構（dry_run / phase / assertion）」を定義するのに対し、本仕様は「いつ / どの signal で移行を発動するか」の発火条件と全 OSS の inventory を単一所有

## 設計原則
- **lifecycle_class は bundle**: class 1 値が他 4 dimension（`adoption_status` / `health_check_cadence` / `migration_trigger_set` / `response_action`）を一意に導出
- **dimension override 禁止**
- **dead spec を CI で殺す**: 本仕様 / 02_採用 OSS 一覧 / 02_Library / 02_移行 Pair / 他軸 spec から参照されない OSS は dead inventory として CI fail
- **移行 trigger は構造化された signal の集合**: 8 signal を class が purely additive に enumerate、各 signal は 09 audit signal の sub-class `oss.lifecycle.event` として物理 emit
- **inventory.lock.yaml は build artifact**: 02_採用 OSS 一覧（人間可読）と本仕様の `oss_inventory.lock.yaml`（機械可読）の双方向整合を build script で担保
- **defense-in-depth は 5 層**

## v1 lifecycle_class セット（6 class）

| class | adoption_status | health_check_cadence | primary migration_trigger | response_action |
|---|---|---|---|---|
| `v1_l1plus_primary` | primary_l1plus | quarterly | license_change / eol_announced / cve_backlog_threshold / maintainer_turnover / fork | pair_switch_to_target（08 dry_run の lock 切替） |
| `v1_l1plus_pair_target` | pair_l1plus | annual | dry_run_cadence_elapsed（365 日） | conformance_dry_run_run |
| `v1_l2star_member` | l2star_member | quarterly | conformance_drift / license_change / eol_announced | family_substitute_or_demote |
| `v1_l3_runtime` | l3_companion_runtime | semi_annual | major_up / eol_announced / cve_backlog_threshold | runtime_upgrade_with_compat_test |
| `v1_reserved_category` | reserved | annual | adoption_demand / blocker_resolution | promote_to_primary_or_remove |
| `v1_inhouse_authoritative` | inhouse_authoritative | quarterly | spec_drift / cve_backlog_threshold / conformance_drift / major_up | inhouse_spec_track_or_runtime_upgrade |

### 各 class の不変条件
- `v1_l1plus_primary`: 「単一 OSS 深耕」哲学の現役採用 OSS。7 signal いずれかの発生で移行 trigger を発動し、08 の primary pair を新 primary に切替。quarterly health check（CVE backlog / maintainer commit cadence / Foundation status / license の 4 指標）。典型: CloudNativePG / Strimzi / Temporal / ZEN / Buf / Apicurio / sqlx / OpenBao / Keycloak / Argo CD / Backstage / Prometheus / OpenTelemetry Collector / ClickHouse / Cosign / Trivy / Kyverno / cert-manager / SPIRE / Litmus / Weaver
- `v1_l1plus_pair_target`: L1+ 移行 pair の控え。08 `dry_run.lock.yaml` で年次 dry_run を完了し、`last_green_at` が 365 日以内であること。典型: StackGres / RedPanda / Cadence / 自製 DSL backend
- `v1_l2star_member`: 同族二重採用カテゴリ（L2\*）の各メンバ。Apicurio + Karapace、flagd + GFF、Parca + Pyroscope。Vector Search はかつて L2\* 候補だったが、L1+ 単一深耕（pgvector）+ 移行コミットメント枠に再分類。族外 standalone vector engine への移行は `project_spawn`（業務コード再設計プロジェクト発足）として扱う
- `v1_l3_runtime`: Companion runtime（業務的に併走する OSS）。Headlamp / Kubeshark / Perses / Apache Superset / Jaeger v2 / Backstage / PgBouncer / kube-state-metrics / kube-vip / KEDA / MetalLB / OpenTofu / Tilt / Helm / Kustomize / Tekton / cert-manager / ESO / Harbor。半期 health check（major_up / eol_announced / CVE backlog の 3 trigger）
- `v1_reserved_category`: 「予約カテゴリ」（Distributed SQL、Document Store、Time-Series 専用 DB 等）。年次 review で adoption_demand or blocker_resolution を trigger とし、promote_to_primary or remove の判断
- `v1_inhouse_authoritative`: 採用 OSS 候補の機能ギャップを埋めるために企画自製（in-house Apache 2.0 配布、`maintained_by: in_house`）される library / runtime。「自製禁止規律はあるが、L1+ 単一深耕の health_check_cadence を community fork にギャンブルできない場合に限り認める」例外区画。`spec_drift`（実装が依拠する公開 spec の改訂）を primary trigger に持つ。典型: `k1s0.Connect.NetCore` / `k1s0_apicurio_additive_controller` / `k1s0_hlc_lib` / 自製 DSL backend

## 8 signal の定義
- `license_change`: ライセンス変更（Apache 2.0 → BUSL 等）
- `eol_announced`: ベンダーまたはコミュニティが EOL を公式宣言
- `cve_backlog_threshold`: critical / high CVE が SLA 超過件数
- `maintainer_turnover`: コア maintainer の交代が一定期間内に閾値超過
- `fork_event`: コミュニティが分裂（hard fork）
- `conformance_drift`: L2\* 同族 / Conformance Suite で member 間 drift 検出
- `major_up`: 本命 OSS の semver major version up
- `spec_drift`: 実装が依拠する公開 spec の改訂（`v1_inhouse_authoritative` 限定）

各 signal は 09 audit signal の sub-class `oss.lifecycle.event` として物理 emit。

## dimension の根拠
- `adoption_status` は OSS の役割を決める（primary / pair_target / l2star_member / l3_runtime / reserved / inhouse_authoritative）
- `health_check_cadence` は signal 観測の頻度（quarterly / annual / semi_annual）
- `migration_trigger_set` は class が purely additive に enumerate する signal 集合
- `response_action` は trigger 発火時の行動

## 単一の真（3 ファイル構成）

### `classes.yaml`（軸 lifecycle_class enum）
- class bundle 定義

### `scenarios.yaml`（軸 catalog）
- scenario × lifecycle_class → signal injection + assertion id
- 主要 scenario: `license_change_detected` / `cve_backlog_overflow_freezes_deploy` / `dry_run_cadence_elapsed_blocks_release` / `l2star_conformance_drift_demotes_member` / `major_up_with_compat_test` / `spec_drift_detected`

### `oss_inventory.lock.yaml`（build artifact）
- 全採用 OSS の static 自己宣言（class / version / SBOM hash / Cosign signature / health_check_last_at / signal_source_endpoint）を tier1 build script が集約して生成。手書き禁止
- 02_採用 OSS 一覧 と本 lock の双方向整合を CI で担保

## 5 層 defense-in-depth
- 層 A: compile 時（採用 OSS は inventory 登録必須、image build 時に inventory tag 必須）
- 層 B: lint（採用 OSS リスト変更が PR 上で必須承認）
- 層 C: 02_移行 Pair 適合仕様 dry_run scenarios と接続して L1+ pair の conformance を CI 検証
- 層 D: runtime monitoring（signal を 09 audit に emit）
- 層 E: license 違反 / EoL OSS の物理拒否（Kyverno admission policy + Cosign 署名検証）

## CI 不変条件（merge 不可）
- 整合 1: `classes.yaml` の全 class が `scenarios.yaml` の matrix で覆われる
- 整合 2: `oss_inventory.lock.yaml` の全 OSS が `classes.yaml` のいずれかの class に属する（dead inventory 検出）
- 整合 3: `oss_inventory.lock.yaml` は手書き禁止、build artifact と git の差分検出
- 整合 4: 02_採用 OSS 一覧 と `oss_inventory.lock.yaml` の双方向整合
- 整合 5: 全 OSS の Cosign signature + SBOM が Harbor に存在
- 整合 6: `v1_l1plus_primary` の各 OSS について 02_移行 Pair 適合仕様 で primary pair が宣言され、`dry_run.lock.yaml` の `last_green_at` が 365 日以内
- 整合 7: `v1_inhouse_authoritative` の各 entry について `requires_inhouse_spec_memo: true` の memo file が存在
- 整合 8: dead spec 検出

## 1.0.0 ship blocker
- 6 lifecycle_class × 全 OSS instance の完全 conformance green
- `oss_inventory.lock.yaml` が 02_採用 OSS 一覧 と byte-equal
- 全 `v1_l1plus_primary` OSS の primary pair dry_run green
- `v1_inhouse_authoritative` 全 entry の `inhouse_spec_memo` 存在

## 採用しない設計
- lifecycle_class dimension override
- 文章のみの OSS 選定（必ず inventory 登録 + build artifact 経由）
- AGPL / SSPL / BSL / Confluent Community License / Elastic License v2
- single_region HSM 配置（KEK shamir M-of-N 必須）
- self-fork without lifecycle_class 宣言

## 関連参照
- [tier1 設計方針 index](../../03_概要設計/02_tier1設計方針/README.md)
- [Library](../../03_概要設計/02_tier1設計方針/02_Library.md)
- [提供機能カテゴリ](../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md)
- [移行 Pair 適合仕様](02_移行Pair適合仕様.md)
- [tier1 強制機構](../02_強制機構/01_tier1強制機構.md)
- [.NET 8 LTS Connect-RPC 自製実装](../03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
- [apicurio GitOps SoT](../03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md)
