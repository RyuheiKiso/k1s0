---
id: req.tech.inhouse_policy
axis: overview
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.tech.tech_index
  - detail.tier1.oss_lifecycle_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# 自製 in-house 方針

## 一文方針
- 採用 OSS 候補の機能ギャップを埋めるための自製は `v1_inhouse_authoritative` lifecycle_class（in-house Apache 2.0 配布、`maintained_by: in_house`）の例外区画でのみ認める。「自製禁止規律はあるが、L1+ 単一深耕の health_check_cadence を community fork にギャンブルできない場合に限り認める」原則。

## v1_inhouse_authoritative の採用条件
- L1+ 単一深耕の health_check_cadence を community fork に依存できない
- 採用 OSS 候補の機能ギャップが明確
- spec_drift signal source（外部公開 spec の改訂 watch）が明示
- inhouse_spec_memo（軸内 .txt メモ）の存在が必須

## 1.0.0 ship の自製 OSS（v1_inhouse_authoritative）
- **`k1s0.Connect.NetCore`**: .NET 8 LTS 向け Connect-RPC 実装（Connect 公式 .NET 実装が存在しないため）
- **`k1s0_apicurio_additive_controller`**: Apicurio Operator が CR で表現できない rule 形を補完
- **`k1s0_hlc_lib`**: HLC（Hybrid Logical Clock）言語別 wrapper
- **`protoc-gen-k1s0-go-fsm`**: Go の typestate via nominal types codegen plugin
- **自製 DSL backend**: ZEN Engine 互換 Rust 実装
- **`k1s0 Companion Mock`**: SDK 開発者の local mock backend
- **Tauri Companion sidecar**: WebUSB / Web Bluetooth / Web Serial の Firefox / Safari bridge
- **`k1s0.Companion.NetFx.OTelExt`**: signalfx/splunk-otel-dotnet fork + k1s0 processor

## 自製 OSS の規律（dimension override 禁止規律を継承）
- license: Apache 2.0
- health_check_cadence: quarterly 固定
- 8 signal のうち maintainer_turnover / fork_event / license_change を持たない（自製のため定義不能）
- 代わりに `spec_drift`（外部 spec 改訂）を primary trigger
- cve_backlog_threshold: 内部 grype scan
- response_action: `inhouse_spec_track_or_runtime_upgrade`

## 公開戦略
- GitHub public repo + Cosign signed + SBOM + SLSA L3+ attestation
- 配布: NuGet / npm / cargo / Maven Central / RubyGems + Harbor mirror
- contribution: CLA + 4 reviewer dual sign-off
- LLM 単独 sign-off 禁止

## 採用しない自製
- `v1_inhouse_authoritative` の宣言なき自製
- inhouse_spec_memo なき自製
- BUSL / SSPL / AGPL 系 license での自製
- contribution license agreement なしの PR 受領

## 受入条件
- 全 v1_inhouse_authoritative 自製 OSS が `oss_inventory.lock.yaml` に登録
- inhouse_spec_memo 全揃い
- Apache 2.0 + Cosign signed + SBOM + SLSA L3+

## 関連参照
- [技術選定 index](README.md)
- [OSS 採用一覧](01_OSS採用一覧.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [.NET 8 LTS Connect-RPC 自製実装](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
- [protoc-gen-go FSM](../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- [OSS 公開戦略](../../01_企画/06_OSS公開戦略/README.md)
