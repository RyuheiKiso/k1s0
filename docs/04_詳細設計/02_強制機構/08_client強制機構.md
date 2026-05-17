---
id: detail.client.client_enforcement
axis: client
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.client.client_index
  - arch.client.codegen_policy
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# client 強制機構

## 一文方針
- client SDK の規律は CI / lint / contract test / property test / supply chain attestation で物理 enforce、手書き SDK code path / public API drift / SemVer skew はすべて CI fail、最終 safety net は Cosign signed package + Kyverno admission policy（infra 側）で物理 reject。

## 5 層 defense-in-depth

### 層 A: compile（codegen / 型 check）
- protoc + Buf による単一 codegen pipeline（[コード生成方針](../../03_概要設計/09_client設計方針/08_コード生成方針.md)）
- 全言語 SDK の generated code が compile できること
- method annotation（`retry_policy` / `slo_class` / `quota_class` / `observability_class` / `capability_class`）の必須化（buf lint）
- `sdk_inventory.lock.yaml` / `capability_matrix.lock.yaml` / `sdk_conformance.lock.yaml` は build artifact、手書き禁止

### 層 B: lint（規約 check）
- 全 SDK 公開 API に annotation 必須（`retry_policy_class` / `capability_class` / `pii`）
- SDK 公開 API として「生 HTTP」「LocalStorage 直書込」「access_token getter」「自製 crypto」が露出していないこと
- 言語別 lint:
  - .NET 8 / .NET Framework: Roslyn analyzer（自製 `K1s0Analyzer` pack）+ `.editorconfig` + StyleCop
  - Java 21: SpotBugs + ErrorProne + 自製 `k1s0-spotbugs-plugin` + Checkstyle
  - Node.js 20 / Browser TS: ESLint + `@typescript-eslint` + 自製 `eslint-plugin-k1s0` + `eslint-plugin-boundaries`
  - Rust: clippy + 自製 `cargo-k1s0-lint` subcommand
  - Python: ruff + `mypy --strict` + 自製 `k1s0-mypy-plugin`
  - Ruby: rubocop + 自製 `rubocop-k1s0`
  - Go: golangci-lint + 自製 `k1s0-golangci-plugin`
- Buf lint: proto style + breaking change check
- npm audit / cargo audit / pip-audit / bundler-audit / dependabot security advisories: critical / high ゼロ必須

### 層 C: integration test / contract test
- Pact provider / consumer contract test（全 9 言語 × 全 capability_class）
- Testcontainers で tier1 Server + Companion Mock + tier2 atomic 三表書込 + Outbox + Keycloak + OpenBao を起動、SDK が round-trip
- Playwright で Browser SPA + Tauri SDK の E2E
- 39 適合仕様 8 製造業 pack stress test scenario の SDK 投影テスト

### 層 D: runtime（SDK ランタイムの物理 enforcement）
- WebCrypto / OS keychain / DPAPI による at-rest encryption
- 24h Idempotency-Key TTL の SDK ランタイム enforcement
- retry budget の per-tenant / per-method / per-aggregate sliding window
- circuit breaker open/half-open/closed の状態遷移
- capability negotiation の決定論性
- OTel Auto-Instrumentation の attach（disable 不能）

### 層 E: 物理 enforcement（OS / Kubernetes / OSS の物理機能）
- WebCrypto API（Browser）の non-extractable CryptoKey は JavaScript が export 不能（browser engine 物理機構）
- OS keychain（macOS Keychain / Windows DPAPI / Linux SecretService）の secret 取出は OS user 認証必須
- Cosign signed package のみ install 可能（Harbor admission policy で unsigned reject）
- Kyverno admission policy（infra 側、08_infra/14）:
  - `require-image-from-harbor`
  - `block-on-cve-backlog-threshold`
  - `require-cosign-signature`
  - `block-host-network-host-pid`
  - `require-tls-1.3-or-higher`
- Browser engine の CSP（Content Security Policy）が default `'self'` で SDK が外部 script eval 不能

## CI 不変条件（client 全体、merge 不可）
- 整合 1: 全 RPC method に `retry_policy_class` / `slo_class` / `quota_class` / `observability_class` / `capability_class` annotation
- 整合 2: 全 PII field に `field_pii` / `redaction_class` annotation
- 整合 3: 全言語 SDK の MAJOR.MINOR が一致（`sdk_inventory.lock.yaml`）
- 整合 4: 全言語 SDK の generated code が compile
- 整合 5: 全 capability_class について Pact provider / consumer contract test green
- 整合 6: 全言語 SDK で property test green（retry backoff / idempotency_key / capability negotiation / 4 layer reducer）
- 整合 7: SemConv attribute 名が tier1 09_観測適合仕様 の semconv バージョンと一致
- 整合 8: `capability_matrix.lock.yaml` が tier1 03_Server 系 の actual と双方向一致
- 整合 9: `sdk_conformance.lock.yaml` の `last_green_at` が 14 日以内
- 整合 10: 全 SDK package に Cosign signature + SBOM attestation 添付
- 整合 11: SDK 公開 API snapshot drift なし（言語別 public-api-snapshot ツール）
- 整合 12: SDK 内部で生 HTTP / LocalStorage / access_token getter / 自製 crypto 利用ゼロ
- 整合 13: 全 9 言語 SDK の公開 API が同型（method 名 / type 名 / annotation 値の言語横断 diff = 0）
- 整合 14: `connect_bidi` adapter の全 implementation_per_runtime（`@connectrpc/connect-web`、`connectrpc.com/connect`、`connect-kotlin`、`connect-swift`、`k1s0.Connect.NetCore` 等）が Connect Conformance Suite（`github.com/connectrpc/conformance`、Apache-2.0）の全 RPC form（unary / server-streaming / client-streaming / bidi-streaming）+ HTTP/2 / HTTP/3 + protobuf binary / JSON 全 case で green。green=false が連続 2 週で release block

## supply chain attestation
- 全 SDK package:
  - Cosign signed
  - SBOM (Syft) 添付
  - 脆弱性スキャン (Grype) 通過
  - SLSA provenance attestation（in-toto）添付
  - Build reproducibility verification（同 input → 同 output bit hash）
- 言語別:
  - npm: sigstore signing
  - PyPI: PEP 740 sigstore signing
  - cargo: cargo-sigstore 実験（v2）+ Harbor Cosign
  - NuGet: NuGet package signing + Cosign 二重署名
  - Maven: gpg signing + Cosign 二重署名
  - RubyGems: gem signing + Cosign

## break-glass
SDK CVE 緊急 hotfix で lockstep を一時的に破る場合:
1. break-glass ticket（Backstage で起票）
2. 影響 SDK 言語に限定して hotfix patch を release
3. 30 日以内に他言語の lockstep 復帰必須
4. break-glass 利用は 09 観測可能性 SoR に audit emit（`client.break_glass.event`）
5. postmortem PR 起票必須

## admission policy lifecycle（infra 側）
SDK package を pull する K8s Pod / Job は次の admission policy 配下:
- `require-image-from-harbor`（infra 14 と同期）
- `block-on-cve-backlog-threshold`
- `require-cosign-signature`

SDK 自体は package registry 配信のため infra cluster の admission policy を直接通らないが、enterprise 利用先の K8s deployment に SDK が同梱された image を pull する時に物理 reject される構造。

## SDK 公開 API snapshot
各言語 SDK で「公開 API（exported method / type）」の snapshot を git 管理:
- .NET: `public-api-snapshot.txt`（PublicApiAnalyzers）
- Java: japicmp diff（Maven plugin）
- Node.js: api-extractor (Microsoft)
- Browser: api-extractor
- Rust: cargo-public-api
- Python: `pyproject.toml` + 自製 snapshot
- Ruby: rubygems-yard-api 自製
- Go: 自製 go-public-api-snapshot

PR で snapshot 差分があれば release notes / migration codemod の存在を CI gate で要求。

## 採用しない強制機構
- OPA Gatekeeper（Kyverno L1+ 深耕）
- Snyk / WhiteSource（商用、OSS 系 Trivy / Grype のみ）
- SDK 公開 API として「dev mode」を提供して runtime check を緩める
- 言語別 retry semantics 違いを許容する型の例外

## 至高路線における立ち位置
- SDK の規律は文章に依存しない。CI / lint / contract test / property test / Cosign signed package / OS 物理機構の多重防御で物理 enforce
- 1.0.0 release 時に 14 不変条件 + 5 層 defense-in-depth が全 green であることを物理 ship blocker（`sdk_conformance.lock.yaml` の `last_green_at` が 14 日以内、cosign signed tag が release 物理 prerequisite）

## 関連参照
- [client 設計方針 index](../../03_概要設計/09_client設計方針/README.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [client 運用 UI](../04_運用UI開発者体験/05_client運用UI.md)
