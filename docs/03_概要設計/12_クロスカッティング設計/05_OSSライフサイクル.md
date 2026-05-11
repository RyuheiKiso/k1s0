---
id: arch.cross_cutting.oss_lifecycle
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.cross_cutting.cross_cutting_index
  - detail.tier1.oss_lifecycle_conformance
  - detail.tier1.migration_pair_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# OSS ライフサイクル

## 一文方針
- 6 lifecycle_class × 4 dimension（adoption_status / health_check_cadence / migration_trigger_set / response_action）の bundle で全採用 OSS の lifecycle を機械可読に固定。8 signal（license_change / eol_announced / cve_backlog_threshold / maintainer_turnover / fork_event / conformance_drift / major_up / spec_drift）で移行 trigger を発火、L1+ 4 primary pair の dry-run green を年次で維持。

## 6 lifecycle_class（[OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)）
- `v1_l1plus_primary`: 単一 OSS 深耕の現役採用
- `v1_l1plus_pair_target`: L1+ 移行 pair の控え（年次 dry_run）
- `v1_l2star_member`: 同族二重採用カテゴリのメンバ
- `v1_l3_runtime`: Companion runtime
- `v1_reserved_category`: 予約カテゴリ（Distributed SQL 等）
- `v1_inhouse_authoritative`: 自製 in-house（Apache 2.0、`maintained_by: in_house`）

## 8 signal
- `license_change`: ライセンス変更
- `eol_announced`: ベンダー / コミュニティが EOL を公式宣言
- `cve_backlog_threshold`: critical / high CVE が SLA 超過件数
- `maintainer_turnover`: コア maintainer の交代
- `fork_event`: コミュニティが分裂（hard fork）
- `conformance_drift`: L2\* 同族 / Conformance Suite で member 間 drift
- `major_up`: 本命 OSS の semver major version up
- `spec_drift`: 公開 spec の改訂（`v1_inhouse_authoritative` 限定）

各 signal は 09 audit signal の sub-class `oss.lifecycle.event` として物理 emit。

## 4 primary pair（[移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)）
- `relational_pg_pair`: CloudNativePG → StackGres
- `messaging_kafka_pair`: Apache Kafka → RedPanda
- `workflow_pair`: Temporal → Cadence
- `rule_engine_pair`: ZEN Engine → 自製 DSL backend

## L1+ 移行コミットメント
- day-1 で複数 backend を並走させない
- OSS ライフサイクルイベント発生時に tier1 が toolchain で支援
- toolchain は Library 本体と同じリポジトリで継続的に維持
- 移行先候補（カテゴリごとに 1 つ以上）を文書化、Testcontainers での dry-run 移行テストを年次で実施
- dry-run の green を Library リリース条件に含める

## 5 phase 移行（[移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)）
- `schema_diff`
- `state_replicate`
- `dual_write_ramp`（0% / 5% / 25% / 50% / 75% / 100%）
- `cutover`（短時間 write freeze）
- `rollback`（24h 以内）

## v1_inhouse_authoritative 区画
- 採用 OSS 候補の機能ギャップを埋めるために企画自製
- 「自製禁止規律はあるが、L1+ 単一深耕の health_check_cadence を community fork にギャンブルできない場合に限り認める」例外区画
- 典型: `k1s0.Connect.NetCore` / `k1s0_apicurio_additive_controller` / `k1s0_hlc_lib` / 自製 DSL backend / `protoc-gen-k1s0-go-fsm`
- `requires_inhouse_spec_memo: true` の memo file 必須

## 5 層 defense-in-depth
- 層 A: 採用 OSS は inventory 登録必須、image build 時に inventory tag 必須
- 層 B: lint（採用 OSS リスト変更が PR 上で必須承認）
- 層 C: 08 dry_run scenarios と接続して L1+ pair の conformance を CI 検証
- 層 D: runtime monitoring（signal を 09 audit に emit）
- 層 E: license 違反 / EoL OSS の物理拒否（Kyverno admission policy + Cosign 署名検証）

## 採用しない設計
- lifecycle_class dimension override
- 文章のみの OSS 選定
- AGPL / SSPL / BSL / Confluent Community License / Elastic License v2
- self-fork without lifecycle_class 宣言
- toolchain なしの MAJOR up

## 関連参照
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)
- [tier1 Library](../02_tier1設計方針/02_Library.md)
- [.NET 8 LTS Connect-RPC 自製実装](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
