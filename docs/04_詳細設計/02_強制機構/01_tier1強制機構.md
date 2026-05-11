---
id: detail.tier1.tier1_enforcement
axis: tier1
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.tier1.tier1_index
  - arch.tier1.libraries
  - arch.tier1.feature_categories
  - detail.tier1.bidi_conformance
  - detail.tier1.migration_pair_conformance
  - detail.tier1.observability_conformance
  - detail.tier1.auth_conformance
  - detail.tier1.key_management_conformance
  - detail.tier1.schema_evolution_conformance
  - detail.tier1.slo_conformance
  - detail.tier1.oss_lifecycle_conformance
  - detail.tier1.tenant_capacity_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_property_axiom_proof
---

# tier1 強制機構

## 一文方針
- tier1 facade 規律（OSS 直接 import 禁止、`.proto` 単一の真、9 適合仕様の bundle 維持、L1+ 移行コミットメント、Companion 二役割）は CI / lint / 公開 API snapshot / 内部 registry / Kyverno admission policy / Cosign 署名検証 の 6 層多重防御で物理 enforce、手書き drift / dimension override / dead spec はすべて CI fail、最終 safety net は Harbor admission policy + Cosign 署名検証 + HSM zeroize + 外部公証 attestation で物理 reject。

## 5 層 defense-in-depth（tier1 全体）

### 層 A: compile（codegen / 型 check）
- protoc + Buf による単一 codegen pipeline（External / Internal Proto 両層）
- 全 RPC method に必須 method annotation（`tier1.bidi.conformance_class` / `tier1.auth.auth_class` / `tier1.slo.slo_class` / `tier1.capacity.quota_class` / `tier1.observability.signal_class` / `field_pii` / `redaction_class`）
- KeyHandle / AuthContext は必須引数化、生 key bytes / 生 access_token 不可視
- build artifact（`capabilities.lock.yaml` / `dry_run.lock.yaml` / `instruments.lock.yaml` / `enforcement_points.lock.yaml` / `backends.lock.yaml` / `idp_capabilities.lock.yaml` / `oss_inventory.lock.yaml` / `registries.lock.yaml`）は手書き禁止

### 層 B: lint（規約 check）
- Buf custom lint（proto → 各適合仕様の `classes.yaml` 照合）
- 言語別 lint:
  - Rust: `cargo-deny` の `[bans]`（OSS クライアント crate を deny）+ `cargo public-api`（公開 API snapshot）+ clippy + 自製 `cargo-k1s0-lint`
  - C# (.NET 8+): BannedApiAnalyzers + Central Package Management + `Microsoft.CodeAnalysis.PublicApiAnalyzers` + Roslyn analyzer 自製 `K1s0Analyzer`
  - Go: golangci-lint の `depguard` + `go-apidiff` + 自製 `go/analysis`
  - TypeScript: `eslint-plugin-import` の `no-restricted-imports` + `eslint-plugin-boundaries` + `api-extractor`
- L3 / L2\* カテゴリは公開シグネチャに OSS 型が現れたら CI fail。L1+ カテゴリは「[提供機能カテゴリ](../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md)」の露出概念一覧と allowlist 一致を検査

### 層 C: integration test / contract test
- Testcontainers による全 8 adapter × 5 conformance_class × 9 scenario の Bidi conformance test（Bidi 適合仕様 整合 2）
- 4 pair × 5 phase の dry_run（移行 Pair 適合仕様 layer C）
- L2\* 同族の二バックエンド conformance（Apicurio + Karapace、flagd + GFF、Parca + Pyroscope）
- Pact provider / consumer contract test（全 Library 公開 API × 全 capability_class）
- Litmus chaos test で SLO burn-rate alert / quota noisy neighbor isolation の発火検証
- KEK shamir threshold ceremony の dry_run（property p1〜p5）

### 層 D: runtime（tier1 ランタイムの物理 enforcement）
- Capability Negotiation で `chosen_transport` 決定（Bidi 適合仕様）
- Envoy `jwt_authn` filter + Keycloak token introspection + DPoP / mTLS（認証適合仕様）
- HSM PKCS#11 + SPIRE + OpenBao audit log（鍵管理適合仕様）
- Apicurio Registry runtime での schema 違反 reject（スキーマ進化適合仕様）
- Prometheus + alertmanager + Argo Rollouts の error budget burn-rate page（SLO 適合仕様）
- 4 階層 enforcement（envoy_local_rate_limit / library_token_bucket / oss_native_quota / storage_layer_quota / broker_layer）（テナント容量適合仕様）
- OTel Collector による PII 物理削除 + `trace_id` 統一注入（観測適合仕様）

### 層 E: 物理 enforcement（OS / Kubernetes / OSS の物理機能）
- 内部パッケージリポジトリ一元化（cargo / NuGet / Athens / Verdaccio）— OSS クライアントの物理取得不能
- Harbor admission policy で unsigned image / package を reject
- Kyverno admission policy（infra 側、08_infra/14）:
  - `require-image-from-harbor`
  - `block-on-cve-backlog-threshold`
  - `require-cosign-signature`
  - `block-host-network-host-pid`
  - `require-tls-1.3-or-higher`
  - `block-direct-apicurio-write`（apicurio_gitops_sot）
  - `block-single-region-kek-reconstruction`（KEK_shamir_distribution）
- HSM PKCS#11 destroy command による KEK shamir M-of-N share zeroize
- 外部公証 attestation（RFC 3161 trusted timestamp + Sigstore transparency log）による audit hash chain root
- Browser engine の CSP（Content Security Policy）が default `'self'`
- Envoy Gateway 業務 listener が HTTP/2 + HTTP/3 のみ accept、ALPN h2 必須（HTTP/2 enforcement）

### 層 F（formal、optional）
- TLA+ + Apalache による Bidi conformance / 認証 / 鍵管理の temporal_safety_proof
- Stainless / Dafny による Library API の program_correctness_proof
- Lean 4 + mathlib による KEK shamir threshold algebra の数学的 proof
- Kani / CBMC による Rust / C 実装の runtime_modelcheck_proof

## CI 不変条件（tier1 全体、merge 不可）
- 整合 1: 全 RPC method に Bidi / Auth / SLO / Quota / Observability の class annotation
- 整合 2: 全 PII field に `field_pii` / `redaction_class` annotation
- 整合 3: External / Internal Proto の対応マップが build artifact、手書き diff は CI fail
- 整合 4: `capabilities.lock.yaml` の adapter supports と proto conformance_class が双方向 lock
- 整合 5: 4 pair × 5 phase = 20 cell の `dry_run.lock.yaml` `last_green_at` が 365 日以内
- 整合 6: Library 公開 API snapshot drift なし（言語別 public-api-snapshot ツール）
- 整合 7: `oss_inventory.lock.yaml` と 02_採用 OSS 一覧 の双方向整合
- 整合 8: KeyHandle / AuthContext 以外の生 key bytes / 生 access_token 露出ゼロ
- 整合 9: `enforcement_points.lock.yaml` の全 enforcement point が実 deploy で動作
- 整合 10: `instruments.lock.yaml` の全 SLO instrument が Prometheus + Perses + Litmus で動作
- 整合 11: aggregate_qualified_name の 4 軸（proto / event / DB / SemConv）同期 property green
- 整合 12: `idp_capabilities.lock.yaml` の全 IdP adapter が認証 conformance scenario で green
- 整合 13: `backends.lock.yaml` の全 key backend が rotation / destruction scenario で green
- 整合 14: cross-region 系 preservation_class の write 経路は `v1_request_latency_p99_cross_region` を必須採用、双方向 lock
- 整合 15: dead spec 検出（参照されない class / scenario / adapter / signal / SDK package / OSS は CI fail）

## L1+ 移行コミットメント特別 enforcement
- 4 primary pair の dry_run cadence（365 日）超過は Library リリース pipeline を物理 block
- 1.0.0 ship 前は initial dry_run（4 pair すべて green）が ship blocker

## L2\* 同族保証特別 enforcement
- 各 L2\* カテゴリで 2 実装の Testcontainers conformance test green を merge 条件
- 片側でしか通らないテストが発生 → 当該 API は L2\* として適格でないとみなし API を族内ポータブル範囲まで縮める or L1+ へ昇格

## Companion 特別 enforcement
- `k1s0.Companion.NetFx.OTelExt` の 4 stack 全てで JWT claim → Activity attribute 注入を property test で検証
- `k1s0.Connect.NetCore` は Connect Conformance Suite 全 case green が ship blocker
- WinHttpHandler の prerequisite check（Windows 10 1607+ / Server 2016+）が起動時 fail-fast

## supply chain attestation
- 全 tier1 deliverable（Server 系 image / Library package / Companion / lock artifact）に Cosign signature + SBOM (Syft) + 脆弱性スキャン (Grype) + SLSA provenance attestation
- audit hash chain root は外部公証（RFC 3161 + Sigstore transparency log）attestation 必須

## break-glass
- tier1 facade 規律の緊急回避（OSS 直接 import の一時許容 / dimension override の一時許容 等）は禁止
- ただし KEK shamir ceremony の break-glass two-person rule（commander + observer 2 名以上）は admission allow

## 採用しない強制機構
- OPA Gatekeeper（Kyverno L1+ 単一深耕）
- Snyk / WhiteSource（OSS 系 Trivy / Grype のみ）
- 「dev mode」を提供して runtime check を緩める経路
- HashiCorp Vault（OpenBao MPL 2.0 のみ）
- 文章のみの規律（必ず CI / lint / build artifact / Kyverno に物理転写）

## 至高路線における立ち位置
- tier1 facade 規律は文章に依存しない。CI / lint / 公開 API snapshot / 内部 registry / Kyverno admission policy / Cosign 署名検証 / HSM zeroize / 外部公証 attestation の多重防御で物理 enforce
- 1.0.0 release 時に 15 不変条件 + 5 層 defense-in-depth（+ 層 F formal optional）が全 green であることを物理 ship blocker

## 関連参照
- [tier1 設計方針 index](../../03_概要設計/02_tier1設計方針/README.md)
- [Library](../../03_概要設計/02_tier1設計方針/02_Library.md)
- [Bidi 適合仕様](../01_適合仕様/01_Bidi適合仕様.md)
- [移行 Pair 適合仕様](../01_適合仕様/02_移行Pair適合仕様.md)
- [観測適合仕様](../01_適合仕様/03_観測適合仕様.md)
- [認証適合仕様](../01_適合仕様/04_認証適合仕様.md)
- [鍵管理適合仕様](../01_適合仕様/05_鍵管理適合仕様.md)
- [スキーマ進化適合仕様](../01_適合仕様/06_スキーマ進化適合仕様.md)
- [SLO 適合仕様](../01_適合仕様/07_SLO適合仕様.md)
- [OSS ライフサイクル適合仕様](../01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [テナント容量適合仕様](../01_適合仕様/09_テナント容量適合仕様.md)
- [HTTP/2 enforcement](../03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [KEK Shamir 分散](../03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)
- [apicurio GitOps SoT](../03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md)
