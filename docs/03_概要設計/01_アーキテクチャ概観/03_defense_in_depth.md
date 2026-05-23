---
id: arch.overview.defense_in_depth
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.overview.architecture_index
  - arch.overview.nineteen_axis_theory
covered_by:
  defense_in_depth_layers: [A, B, C, D, E, F]
  proof_classes:
    - v1_program_correctness_proof
---

# defense-in-depth 6 層

## 一文方針
- 全 19 軸が共通して採用する defense-in-depth は 6 層（A: compile / B: lint / C: integration test / D: runtime / E: 物理 / F: 数学的）。任意の単層が破れても他層が必ず止める構造で、文章運用ではなく CI / lint / Kyverno / HSM / 外部公証 + formal proof の多重防御で物理 enforce する。

## 6 層の定義

### 層 A: compile（codegen / 型 check）
- protoc + Buf による単一 codegen pipeline
- 型レベル不変条件（phantom type / sealed trait / branded type / typestate via nominal types）
- method annotation 必須化（buf lint）
- build artifact（*.lock.yaml）は手書き禁止、generated と git の差分検出

### 層 B: lint（規約 check）
- 言語別 lint（cargo-deny / Roslyn analyzer / depguard / ESLint）
- 公開 API snapshot drift 検出
- L3 / L2\* カテゴリの公開シグネチャに OSS 型出現で fail
- 依存方向検査（業界横断 → 業界 pack 禁止 / tier3 → tier1 直接 import 禁止）

### 層 C: integration test / contract test
- Pact provider / consumer contract test
- Testcontainers（tier1 Server + Companion Mock + tier2 atomic 三表書込 + Outbox + Keycloak + OpenBao）
- Playwright E2E（Browser SPA + Tauri）
- Litmus chaos drill
- cross-tenant integration test
- conformance scenario corpus（全 adapter × 全 class × 全 scenario）

### 層 D: runtime（OSS / Library の物理 enforcement）
- PostgreSQL Row Level Security FORCE
- pgaudit
- WebCrypto / OS keychain / DPAPI
- 24h Idempotency-Key TTL
- retry budget sliding window
- circuit breaker
- Capability Negotiation
- Envoy `jwt_authn` filter + Keycloak token introspection + DPoP
- atomic 三表書込

### 層 E: 物理 enforcement（OS / Kubernetes / OSS の物理機能）
- Cosign signed package のみ install 可能（Harbor admission policy で unsigned reject）
- Kyverno admission policy（25+ policy）
- HSM PKCS#11 destroy command による KEK shamir M-of-N share zeroize
- 外部公証 attestation（RFC 3161 trusted timestamp + Sigstore transparency log）
- WebCrypto API non-extractable CryptoKey
- OS keychain user 認証
- Browser engine の CSP（`default 'self'`）
- Envoy Gateway 業務 listener が HTTP/2 + HTTP/3 のみ accept、ALPN h2 必須

### 層 F: 数学的 enforcement（formal、optional）
- TLA+ + Apalache による temporal_safety_proof / temporal_liveness_proof
- Stainless / Dafny による program_correctness_proof
- Lean 4 + mathlib による KEK shamir threshold algebra の数学的 proof
- Kani / CBMC による Rust / C 実装の runtime_modelcheck_proof
- 5 proof_class の 95 cell（proof_matrix 基底）coverage

## 軸別 defense-in-depth 層の組合せ
- tier1 / tier2 / tier3 / client: 層 A〜E（formal は対象軸により層 F も）
- security: 層 A〜E（threat_model / build_provenance）
- formal: 層 A〜F（meta-meta）
- ops / test / data / infra: 層 A〜E

## 多重防御の意図
- 任意の単層が破れても他層が必ず止める
- 例: cosign signature 検証が bypass されても、Kyverno admission policy + 内部レジストリ唯一化で物理拒否
- 文章運用に依存しない（人間が忘れても CI / lint / runtime / 物理機構が止める）

## 採用しない defense-in-depth
- 層数を 1〜2 に減らす（必ず 5 層以上、formal 対象は + 層 F）
- 文章運用のみ（必ず CI / lint / build artifact / Kyverno）
- 「dev mode で runtime check を緩める」経路（production / development 区別なし）

## 関連参照
- [アーキテクチャ概観 README](README.md)
- [19 軸論](02_19軸論.md)
- [5 proof_class 論](04_5proof_class論.md)
- [tier1 強制機構](../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- [tier2 強制機構](../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- [tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
