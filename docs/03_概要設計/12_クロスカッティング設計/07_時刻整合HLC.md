---
id: arch.cross_cutting.clock_integrity_hlc
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.cross_cutting.cross_cutting_index
  - detail.infra.clock_integrity_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_program_correctness_proof
---

# 時刻整合 HLC

## 一文方針
- 時刻整合は PTP（Precision Time Protocol）+ chrony + GPS-OCXO で物理層、HLC（Hybrid Logical Clock）で application 層を担保する。SDK 公開 surface から `now()` / `Date.now()` / `time.Now()` / `DateTime.UtcNow` / `SystemTime::now()` を撤去、`monotonic_now()` / `hlc_now()` のみ exposing する。

## 5 clock_integrity_class（[時刻整合適合仕様](../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)）
- `v1_intra_rack_ptp`: 単一 rack 内、PTP grandmaster + GPS-disciplined OCXO で sub-microsecond
- `v1_cross_dc_ntp_smeared`: 複数 DC、chrony NTP smeared
- `v1_application_hlc_only`: HLC（Hybrid Logical Clock）+ monotonic_now、native client（Rust / Go / .NET 8）
- `v1_browser_client_skew_tolerant`: server-anchor HLC token を opaque pass-through、Browser SPA / .NET FW Companion / Thin API
- `v1_air_gapped_legacy`: air-gapped 環境、独立 GPS source

## HLC（Hybrid Logical Clock）
- physical_ms + logical_counter + node_id の 3 tuple
- happens-before 関係を ±skew 注入下でも保つ
- wall-clock subtraction を禁止、HLC tuple の compare で deadline / TTL 判定

## 自製 `k1s0_hlc_lib`
- 言語別 wrapper（Rust / Go / .NET 8）
- v1_inhouse_authoritative（Apache 2.0、`maintained_by: in_house`）
- spec_drift signal source: HLC algorithm 改訂

## SDK 公開 surface の wall-clock API 撤去
- `now()` / `Date.now()` / `time.Now()` / `DateTime.UtcNow` / `SystemTime::now()` を SDK 公開 API として禁止
- `monotonic_now()` / `hlc_now()` のみ exposing
- Roslyn analyzer / ESLint custom rule / Rust `#[deny]` lint / Go vet で wall-clock API 直接 import を error level で reject

## SDK distribution_class と clock_integrity_class の写像
- `v1_full_native_with_companion` → `v1_application_hlc_only`
- `v1_legacy_dotnet_framework` → `v1_browser_client_skew_tolerant`
- `v1_browser_spa_typescript` → `v1_browser_client_skew_tolerant`
- `v1_thick_native_via_tauri` → `v1_application_hlc_only`
- `v1_thin_business_api_only` → `v1_browser_client_skew_tolerant`

## deadline propagation
- RPC deadline = `monotonic_now() + timeout_ms` で算出
- wire 上は HLC tuple として propagate（wall-clock subtraction 禁止）
- 呼出元の monotonic + sender-issued HLC + receiver-side HLC compare

## resume_token の TTL
- server-anchor HLC tuple を必須包含
- client wall-clock 由来の TTL 計算を禁止
- Browser SPA / .NET Framework Companion / Tauri webview client は server-issued HLC token を opaque pass-through
- native client は monotonic + HLC で TTL 判定

## SESSION_ORDERED / CAUSAL conformance class
- `property_axiom`「±60s skew 注入下で HLC happens-before respected」を必須 assertion
- `clock_causality_proof.lock.yaml` と Bidi conformance test fixture が双方向 lock

## 物理層 enforcement
- PTP grandmaster + GPS-disciplined OCXO で sub-microsecond clock sync
- chrony NTP（cross-DC smeared）
- linuxptp（ptp4l / phc2sys / ts2phc）
- Cilium Tetragon（eBPF observability で clock skew 監視）

## 5 層 defense-in-depth
- 層 A: SDK 公開 surface から wall-clock API 撤去（compile fail）
- 層 B: lint（言語別 wall-clock API import deny）
- 層 C: chaos test（skew injection で property test）
- 層 D: runtime（HLC happens-before respected）
- 層 E: 物理（PTP + GPS + chrony）

## 採用しない設計
- SDK 公開 API での wall-clock API 露出
- wall-clock subtraction を deadline / TTL 判定に使う
- 単一 GPS source
- step（leap second）production
- HLC tuple なしの resume_token / deadline propagation

## 関連参照
- [時刻整合適合仕様](../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)
- [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)
- [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md)
- [Tauri Companion sidecar](../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md)
- [client SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)
