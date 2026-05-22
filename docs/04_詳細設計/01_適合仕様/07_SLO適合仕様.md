---
id: detail.tier1.slo_conformance
axis: tier1
phase: detail
kind: conformance_spec
status: published
version: 1.0.0
depends_on:
  - arch.tier1.tier1_index
  - detail.tier1.observability_conformance
  - detail.data.preservation_conformance
  - detail.ops.ops_loop_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_program_correctness_proof
lock_artifacts:
  - instruments.lock.yaml
trace:
  fr_ids:
  - FR-tier1-007

---

# tier1 SLO 適合仕様（v1）

## 一文定義
- SLO 適合仕様は、6 slo_class × 5 dimension（sli_kind / target_level / window_class / multi_window_burn / alert_severity）の cross-product として宣言される SLO catalog の各 instance に対し、Prometheus recording rule + alertmanager rule + Perses dashboard + Litmus chaos scenario を build artifact から生成し、error budget 枯渇時に Kyverno admission policy で deploy 物理 freeze する meta-axis である。

## 位置づけ
- 02_採用 OSS 一覧 Prometheus / Perses / OpenTelemetry Collector / Litmus、04_提供機能カテゴリ Observability / Reliability、03_観測適合仕様 5 signal SoR、tier2 信頼性運用準備（RTO/RPO/SLO 文章記述）が散在的に宣言する SLI 種別 / SLO target / 計測 window / multi-window burn-rate / alert policy / error budget を、機械可読な単一の真として固定
- 03_観測適合仕様 が「signal の入口」を定義するのに対し、本仕様は「signal を SLO に変換し、error budget を物理 enforcement に流す出口」を定義

## 設計原則
- **slo_class は bundle**: class 1 値が他 5 dimension（`sli_kind` / `target_level` / `window_class` / `multi_window_burn` / `alert_severity`）を一意に導出
- **dimension override 禁止**
- **dead spec を CI で殺す**
- **error budget は物理 enforcement する**: error budget が一定閾値以下になると、当該 service の Deployment / Argo Rollout CR を Kyverno admission policy で物理拒否
- **SLI 計測式は build artifact**: Prometheus recording rule / alertmanager rule / Perses dashboard / Litmus chaos scenario は全て classes.yaml + scenarios.yaml + instruments.lock.yaml から build script で生成
- **defense-in-depth は 5 層**

## v1 slo_class セット（6 class）

| class | sli_kind | target | window | multi_window_burn | alert_severity |
|---|---|---|---|---|---|
| `v1_request_availability` | availability | 99.9% | 30d | fast_2h_14.4x + slow_24h_3x | page_immediate |
| `v1_request_latency_p99` | latency | p99 ≤ class.target | 30d | fast_1h_14x + slow_6h_3x | page_immediate |
| `v1_request_latency_p99_cross_region` | latency | p99 ≤ class.target | 30d | fast_1h_14x + slow_6h_3x | page_immediate |
| `v1_event_freshness` | freshness | p95 ≤ class.target | 7d | fast_30m_14x + slow_4h_3x | page_business_hours |
| `v1_workflow_completion` | completion_rate | 99.5% | 30d | fast_4h_14x + slow_3d_3x | ticket_only |
| `v1_data_durability` | durability | 99.999999999% (11n) | 365d | any_loss=immediate | page_immediate |

### 各 class の不変条件と典型用途
- `v1_request_availability`: HTTP / gRPC API 経路の可用性。SLI = `good_events / total_events`、good = 5xx 以外かつ deadline 内 response。月間 error budget 約 43.2 分
- `v1_request_latency_p99`: request 経路の p99 latency。SLI = threshold 内 request の比率。class instantiate 時に `latency_threshold_ms` を指定（例: `v1_alert` RPC = 200ms、07 の `class.lag` と整合）。preservation_class が cross-region 系では本 class を採用しない
- `v1_request_latency_p99_cross_region`: cross-region write を伴う request 経路の p99 latency。典型値は 1500ms（Tokyo↔Osaka 〜30ms RTT × remote_apply 往復、Tokyo↔US 〜180ms RTT を想定）。`v1_cross_region_replicated` / `v1_global_replicated` 配下の write 経路で必須採用
- `v1_event_freshness`: server-driven event 配信の freshness（end-to-end lag）。SLI = `(consumer_observed_at - producer_published_at) ≤ target` 比率
- `v1_workflow_completion`: Temporal Workflow / Saga の完了率。SLI = `completed / started`、timeout / cancellation を fail 扱い
- `v1_data_durability`: tenant data の永続性。SLO target = 11 nines（確率的目標、burn rate 計算用）、freeze policy = `freeze_on_any_loss=true`（決定論的物理機構）。loss event 検出は 32 outbox relay reconciliation / audit hash chain mismatch / DB checksum 等の signal が発生した瞬間、burn rate 計算と独立に物理 freeze に流す

## dimension の根拠
- `sli_kind` は SLI 算出式と numerator/denominator semantics を決める（混合すると burn-rate 計算が分解不能）
- `target_level` は class により semantics が異なる（availability / completion_rate は %、latency / freshness は閾値 + ratio、durability は 11 nines + freeze_on_any_loss）。class instantiate 時に instance 別に拡張属性で具体値を宣言
- `window_class` は SRE Workbook 流の budget 計算 window
- `multi_window_burn` は false positive 抑制と早期検出の両立（fast = 高 burn-rate 短時間、slow = 中 burn-rate 長時間、AND で page）
- `alert_severity` は人間の応答 latency を決める

## derived dimension（per-tenant SLO）を独立次元にしない理由
- per-tenant SLO は本来 dimension たり得るが、6 class それぞれで「`tenant_id` ラベルで分割した同型 SLI」として表現可能。class instantiate 時の instance 属性として表現

## 単一の真（3 ファイル構成）

### `classes.yaml`（軸 slo_class enum）
- class bundle 定義

### `scenarios.yaml`（軸 catalog）
- scenario × slo_class → Litmus chaos scenario id + alert assertion id
- 主要 scenario: `latency_burn_rate_fires_on_inject` / `availability_drop_burst_detection` / `freshness_lag_increase_alert` / `durability_loss_event_freezes_deploy` / `workflow_completion_drop_ticket` / `cross_region_rtt_inject_no_false_positive`

### `instruments.lock.yaml`（build artifact）
- 各 instrument backend（Prometheus / Perses / Litmus / Alertmanager）の static 自己宣言を tier1 build script が集約して生成。手書き禁止
- recording rule / alert rule / dashboard / chaos scenario は全て本 lock から生成

## 5 層 defense-in-depth
- 層 A: compile 時（SLO は yaml 単一の真、coding 不要）
- 層 B: lint（alert / dashboard / chaos が yaml 由来かを CI 検査）
- 層 C: chaos test（Litmus）で burn-rate alert が fire することを検証
- 層 D: runtime（Prometheus + alertmanager + ops-edge cluster page 機構）
- 層 E: error budget の物理 freeze（budget 枯渇で Kyverno が deploy CR を reject）

## CI 不変条件（merge 不可）
- 整合 1: `classes.yaml` の全 class が `scenarios.yaml` の Litmus chaos scenario でカバーされる
- 整合 2: `scenarios.yaml` の全 alert assertion id が alertmanager rule に対応
- 整合 3: `instruments.lock.yaml` は手書き禁止、build artifact と git の差分検出
- 整合 4: cross-region 系 preservation_class（`v1_cross_region_replicated` / `v1_global_replicated`）の write 経路は `v1_request_latency_p99_cross_region` を必須採用、双方向 lock
- 整合 5: 全 SLO instance の error budget が Kyverno admission policy に bind され、枯渇で deploy 物理 freeze
- 整合 6: dead spec 検出

## 1.0.0 ship blocker
- 6 slo_class × 全 instance の完全 conformance green
- Litmus chaos scenario で burn-rate alert fire 検証 green
- Kyverno admission policy が error budget 枯渇時に deploy 物理 freeze する property test green

## SLO 防御 5 層（深く）
- v1_request_availability: 24 SLI / 96 alert
- 詳細は [SLO protection layers](../03_クロスカッティング適合仕様/05_SLO_protection_layers.md) を参照

## 採用しない設計
- 文章のみの SLO 記述（recording rule / alert rule / dashboard / chaos scenario が build artifact）
- slo_class dimension override
- 単窓 alert（MWMBR 必須）
- per-tenant SLO の独立 dimension 化

## 関連参照
- [tier1 設計方針 index](../../03_概要設計/02_tier1設計方針/README.md)
- [観測適合仕様](03_観測適合仕様.md)
- [テナント容量適合仕様](09_テナント容量適合仕様.md)
- [SLO protection layers](../03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
- [運用ループ適合仕様](17_運用ループ適合仕様.md)
