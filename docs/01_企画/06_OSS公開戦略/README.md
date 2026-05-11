---
id: plan.oss_publication_strategy
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
  - plan.legal_check
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# OSS 公開戦略

## 一文方針
- 本企画自製の `v1_inhouse_authoritative` 区画 OSS（`k1s0.Connect.NetCore` / `k1s0_apicurio_additive_controller` / `k1s0_hlc_lib` / `protoc-gen-k1s0-go-fsm` / 自製 DSL backend / k1s0 Companion Mock / Tauri Companion sidecar 等）は Apache 2.0 で GitHub に公開、Cosign signed + SBOM + SLSA L3+ attestation で配布、コミュニティ contribution は CLA + 4 reviewer dual sign-off で受領する。

## 公開対象 OSS
本企画 `v1_inhouse_authoritative` 区画:
- **`k1s0.Connect.NetCore`** (Apache 2.0): .NET 8 LTS 向け Connect-RPC 実装。Connect Conformance Suite 全 case green 維持
- **`k1s0_apicurio_additive_controller`** (Apache 2.0): Apicurio Operator が CR で表現できない rule 形を補完する additive controller
- **`k1s0_hlc_lib`** (Apache 2.0): HLC（Hybrid Logical Clock）言語別 wrapper（Rust / Go / .NET）
- **`protoc-gen-k1s0-go-fsm`** (Apache 2.0): Go の typestate via nominal types codegen plugin
- **自製 DSL backend** (Apache 2.0): ZEN Engine 互換 Rust 実装
- **`k1s0 Companion Mock`** (Apache 2.0): SDK 開発者の local mock backend
- **Tauri Companion sidecar** (Apache 2.0): WebUSB / Web Bluetooth / Web Serial の Firefox / Safari bridge
- **`k1s0.Companion.NetFx.OTelExt`** (Apache 2.0): signalfx/splunk-otel-dotnet fork + k1s0 processor

## 配布規律
- license: Apache 2.0（DCO + CLA 必須）
- artifact: GitHub Release + Harbor mirror + 各言語 official registry（NuGet / npm / cargo / Maven Central / RubyGems）
- 全 artifact: Cosign signed + SBOM (Syft) + SLSA L3+ attestation
- in-toto attestation で chain of custody

## コミュニティ contribution 受領規律
- DCO（Developer Certificate of Origin）+ CLA（Contributor License Agreement）必須
- PR review: 4 reviewer pool（core team）+ dual sign-off
- LLM 単独 sign-off 禁止（必ず人間 reviewer 必須）
- contribution license agreement で Apache 2.0 互換を担保

## 公開タイミング
- 1.0.0 ship と同時に GitHub public repo 公開
- 公開前に internal security review + threat modeling
- 公開後の vulnerability 報告は GitHub Security Advisory + responsible disclosure

## 公開しない（internal only）
- tier2 業界 pack 実装（製造業 pack の業務 schema / Workflow / 決定表）
- tenant 別 override 実装
- 内部 build pipeline（Bazel BUILD ファイル等）
- 内部 Kyverno admission policy（一部）

## upstream 寄稿戦略
- L1+ primary OSS への upstream 寄稿:
  - Apicurio Operator: 標準 CR で表現できない rule 形の支援機能
  - OpenTelemetry Weaver: 業界横断 SemConv 拡張
  - protobuf-go FSM: Go typestate plugin
- 寄稿は本企画の `v1_inhouse_authoritative` のうち upstream 採用が見込まれるもののみ

## v2 候補（公開検討中）
- `k1s0-perf-h3wt`（HTTP/3 + WebTransport load tool、k6 + xk6-webtransport の代替）
- 自製 escalation engine（Argo Workflows + Mattermost ベース）
- 自製 ABAC policy DSL

## 採用しない公開戦略
- BUSL / SSPL / AGPL 系 license での公開
- contribution license agreement なしの PR 受領
- LLM 単独 sign-off
- 公開前 security review なしの公開
- vulnerability disclosure を商用 program で運営

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [法務確認](../04_法務確認/README.md)
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
