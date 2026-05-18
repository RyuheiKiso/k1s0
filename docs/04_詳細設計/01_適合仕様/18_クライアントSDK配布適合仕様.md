---
id: detail.client.sdk_distribution_conformance
axis: client
phase: detail
kind: conformance_spec
status: published
version: 1.0.0
depends_on:
  - arch.client.client_index
  - arch.client.sdk_distribution_policy
  - arch.client.transport_adaptation_policy
  - arch.client.offline_resilience_policy
  - arch.client.auth_context_propagation_policy
  - arch.client.distribution_semver_policy
  - arch.client.retry_backoff_policy
  - arch.client.observability_auto_instrumentation_policy
  - arch.client.codegen_policy
  - detail.client.client_enforcement
  - detail.security.build_provenance_conformance
  - detail.infra.clock_integrity_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_temporal_safety_proof
lock_artifacts:
  - sdk_inventory.lock.yaml
  - capability_matrix.lock.yaml
  - sdk_conformance.lock.yaml
---

# client クライアント SDK 配布適合仕様（v1）

## 一文定義
- クライアント SDK 配布適合仕様は、5 sdk_distribution_class × 7 dimension（language_runtime / distribution_channel / transport_capability_class / auto_instrumentation_mode / companion_provision_mode / device_at_rest_encryption_mode / refresh_token_storage_mode）の cross-product として宣言される SDK catalog の各 instance に対し、5 enforcement orchestrator のいずれかを必ず物理 pointer として bind し、conformance_cadence_days 以内に Pact + Property test + Conformance Suite で実演 green を物理 enforce する meta-axis である。

## 位置づけ
- tier1 / tier2 / tier3 / infra / data / security / formal / clock の 18 軸が散在的に宣言する SDK 配布形 / capability matrix / Auto-Instrumentation モード / refresh_token storage / device_at_rest_encryption / language runtime version / package registry / Cosign 署名形式 / SemVer lockstep を、機械可読な単一の真として一箇所に固定する仕様書
- client が物理的に唯一の SDK 配布 facade として振る舞う
- 19 軸 + meta の 1 軸として、本仕様が tier1 / tier2 / tier3 / infra / data / security / clock / formal の SDK 投影面を統合する

## 設計原則
- **sdk_distribution_class は bundle**: class 1 値が他 6 dimension を一意に導出
- **dimension override 禁止**: 「Browser SPA の refresh_token を localstorage に変えたい」要望は新 class を切る
- **dead spec を CI で殺す**: classes / scenarios / capability cell / SDK package のいずれも参照消失で CI fail
- **5 種類 enforcement orchestrator を全 sdk_distribution_class に必須化**: `buf_codegen_pipeline` / `cosign_supply_chain` / `language_lint_analyzer` / `pact_contract_test` / `property_based_test`
- **sdk_conformance を物理 property とする**: conformance_cadence_days 以内に必ず実演し green
- **defense-in-depth は 5 層で構成**

## v1 sdk_distribution_class セット（5 class）

| class | language_runtime | distribution_channel | transport_capability_class | auto_instrumentation_mode | companion_provision_mode | device_at_rest_encryption_mode | refresh_token_storage_mode |
|---|---|---|---|---|---|---|---|
| `v1_full_native_with_companion` | dotnet8 / java21 / node20 / rust / go122 / python312 / ruby33 | nuget / maven / npm / cargo / go / pypi / gems | full | agent_attach (.NET / Java) / library_auto (others) | optional | os_native | process_memory_only |
| `v1_legacy_dotnet_framework` | dotnet_framework_4_6_2_plus | nuget | limited | clr_profiler_attach | required | dpapi | dpapi_protected_blob |
| `v1_browser_spa_typescript` | browser (Chromium / Firefox / Safari latest LTS) | npm | web | library_auto | required | webcrypto_aes_gcm_non_extractable_cryptokey | in_memory + httponly_cookie |
| `v1_thick_native_via_tauri` | tauri (rust + webview) | cargo + npm + tauri bundle | full | dual | required (sidecar) | webcrypto_aes_gcm_with_os_keychain_key_wrap | os_keychain_secure_storage |
| `v1_thin_business_api_only` | any (polyglot, OpenAPI generated) | not_provided | business_api_only | side_companion_proxy | side_companion_proxy | not_applicable | not_applicable |

### 各 class の不変条件

#### v1_full_native_with_companion
- 用途: tier1 Library を直接組み込めるサーバ側 / 太い client（.NET 8 LTS / OpenJDK 21 LTS / Node.js 20 / Rust stable / Go 1.22 / Python 3.12 / Ruby 3.3）
- transport_capability_class: full（gRPC bidi 完全 + HTTP/3 + WebTransport optional）
- conformance_cadence_days: 14 日

#### v1_legacy_dotnet_framework
- 用途: レガシー .NET Framework 4.6.2+ exe / Service
- transport_capability_class: limited。Connect-RPC bidi も WinHttpHandler の HTTP/2 制約により採用しない（sse_paired を既定）
- wire 出口は **05_tier1/16 役割 H2-A' で常設される v1_legacy_http11 専用 listener**（別ポート 8443-legacy、TLS + HTTP/1.1、ALPN h2 offer なし）に向ける
- Companion 起動時に WinHttpHandler の ALPN h2 probe を行い、(a) 成功 → 業務 listener (h2/h3) を選択（昇格して `v1_full_native_with_companion` 相当の経路）、(b) 失敗 → 当該 legacy listener に固定
- conformance_cadence_days: 30 日

#### v1_browser_spa_typescript
- 用途: Browser SPA（TypeScript / 任意 UI framework 非依存、Chromium / Firefox / Safari の latest LTS のみ）
- companion_provision_mode: required（`@k1s0/companion-web`、Service Worker registration を含む）
- refresh_token_storage_mode: in_memory_only + httponly_cookie_for_refresh（HttpOnly + Secure + SameSite=Lax 固定。CSRF は Double-Submit Cookie + Origin/Referer 二重検査で担保）
- conformance_cadence_days: 14 日

#### v1_thick_native_via_tauri
- 用途: Tauri デスクトップアプリ（Rust + WebView）
- transport_capability_class: full（Tauri sidecar が Rust 側で gRPC bidi / HTTP/3 + WebTransport / WebSocket を担当）
- conformance_cadence_days: 14 日

#### v1_thin_business_api_only
- 用途: tier1 Library / Companion を組み込めない polyglot 環境向け（C++ / Erlang / Elixir / COBOL / その他）
- distribution_channel: not_provided。OpenAPI yaml のみ Harbor 配信
- conformance_cadence_days: 30 日

5 class で v1 covering。新 class は purely additive で追加（v2 拡張ルール）。

## 5 enforcement orchestrator
- `buf_codegen_pipeline`: proto / Apicurio schema を input として全言語 SDK の RPC stub / type / SemConv helper / capability matrix / retry policy table / PII field map を 1 pipeline で生成
- `cosign_supply_chain`: SDK package の Cosign signed + SBOM (Syft) + 脆弱性スキャン (Grype) + SLSA provenance attestation を物理 enforce
- `language_lint_analyzer`: 言語別 lint（Roslyn / SpotBugs / ESLint / clippy / pylint / rubocop / golangci-lint）
- `pact_contract_test`: 全 capability_class について Pact provider / consumer contract test を実行
- `property_based_test`: 全 9 言語 SDK で property based test を実行（retry backoff / idempotency_key / capability negotiation / 4 layer reducer）

## 単一の真

### `classes.yaml`（軸 sdk_distribution_class enum）
- class bundle 定義。class 名 → 6 dimension + instance attributes_required の純粋関数テーブル

### `sdk_inventory.lock.yaml`（build artifact）
- 全 SDK package のバージョン / SHA / SBOM 場所 / Cosign signature / SLSA provenance を記録
- 17 軸 build_provenance の `artifact_inventory.lock.yaml` と cross-axis double-bound

### `capability_matrix.lock.yaml`（build artifact）
- tier1 03_Server 系 が公開する actual capability descriptor と SDK 側 expected capability の双方向 lock
- ua_subclass 軸（`chrome_edge_direct` / `chrome_edge_via_corp_proxy` / `firefox_safari` / `dotnet_companion` / `tauri_native`）と offline 軸 ua_offline_subclass（`ios_safari` / `android_chrome` / `windows_chromium_edge`）の二軸を併存
- 全 capability cell は `(class, adapter, ua_subclass, ua_offline_subclass)` の四軸で 1 entry 以上

### `sdk_conformance.lock.yaml`（build artifact）
- 各 sdk_distribution_class の最新 conformance 実演履歴
- conformance_cadence_days 以内に green であることが ship blocker

### `scenarios.yaml`（軸 catalog）
- conformance / contract / property / E2E test の単一カタログ
- 各 scenario の id / class / level（contract / property / e2e）/ 期待 / 実装場所を宣言

## 5 層 defense-in-depth
- 層 A: compile（codegen / 型 check）— protoc + Buf 単一 pipeline、method annotation 必須、build artifact 手書き禁止
- 層 B: lint（規約 check）— 言語別 lint、SDK 公開 API として「生 HTTP」「LocalStorage 直書込」「access_token getter」「自製 crypto」が露出していないこと
- 層 C: integration test / contract test — Pact provider / consumer contract test、Testcontainers、Playwright、39 適合仕様 製造業 pack stress test scenario の SDK 投影テスト
- 層 D: runtime（SDK ランタイムの物理 enforcement）— WebCrypto / OS keychain / DPAPI、24h Idempotency-Key TTL、retry budget sliding window、circuit breaker、capability negotiation の決定論性、OTel Auto-Instrumentation の attach
- 層 E: 物理 enforcement — WebCrypto API の non-extractable CryptoKey、OS keychain の OS user 認証、Cosign signed package のみ install 可能、Kyverno admission policy、Browser engine の CSP

## CI 不変条件（merge 不可）
- 整合 1: classes.yaml の全 class が `sdk_inventory.lock.yaml` の少なくとも 1 entry でカバー（`v1_thin_business_api_only` は OpenAPI artifact entry でカバー）
- 整合 2: `sdk_inventory.lock.yaml` の全 SDK package が classes.yaml のいずれかの class に属する（dead package 検出）
- 整合 3: `capability_matrix.lock.yaml` の expected が tier1/07/03 server actual capability descriptor と双方向一致
- 整合 4: 全 RPC method に `retry_policy_class` / `slo_class` / `quota_class` / `observability_class` / `capability_class` annotation
- 整合 5: 全 PII field に `field_pii` / `redaction_class` annotation
- 整合 6: 全言語 SDK の MAJOR.MINOR が一致
- 整合 7: 全 capability_class について Pact provider / consumer contract test green
- 整合 8: 全言語 SDK で property test green（retry backoff / idempotency_key / capability negotiation / 4 layer reducer）
- 整合 9: SemConv attribute 名が tier1 09 適合仕様 の semconv バージョンと一致
- 整合 10: scenarios.yaml の全 scenario が CI で実行され、`last_green_at` が `conformance_cadence_days` 以内
- 整合 11: 全 SDK package が build_provenance_class 注釈必須（`v1_thin_business_api_only` → `v1_thin_business_api_artifact`、他 4 → `v1_library_sdk_release`）。10_security/17_build_provenance 適合仕様 の `artifact_inventory.lock.yaml` と本仕様 `sdk_inventory.lock.yaml` が cross-axis double-bound
- 整合 12: SDK 公開 API snapshot drift なし、または PR で codemod / migration 提供
- 整合 13: SDK 内部で生 HTTP / LocalStorage / access_token getter / 自製 crypto 利用ゼロ
- 整合 14: 全 9 言語 SDK の公開 API が同型（method 名 / type 名 / annotation 値の言語横断 diff = 0）
- 整合 15: dead spec 検出（参照されない class / scenario / capability / SDK package は CI fail）
- 整合 16: refresh_token_storage_mode と device_at_rest_encryption_mode の組合せが各 class で固定値（dimension override 検出）
- 整合 17: SDK の time API 公開 surface から `now()` / `Date.now()` / `time.Now()` / `DateTime.UtcNow` / `SystemTime::now()` を撤去、`monotonic_now()` / `hlc_now()` のみ exposing。Roslyn analyzer / ESLint custom rule / Rust `#[deny]` lint / Go vet で wall-clock API 直接 import を error level で reject

## 1.0.0 ship blocker
- 5 sdk_distribution_class × 9 言語 SDK 全てで conformance test green
- 全 SDK package に Cosign signature + SBOM + SLSA provenance attestation 添付
- `sdk_inventory.lock.yaml` / `capability_matrix.lock.yaml` / `sdk_conformance.lock.yaml` が build artifact と git の完全一致
- 製造業 pack 9 業務シナリオ全て green
- `release_gate.lock.yaml` の client cells 全 green、cosign signed tag が物理 prerequisite

## tier1 / tier2 / tier3 / infra / data / security / formal / clock との接合

| client (本仕様) | server / infra / data / security 側 |
|---|---|
| `transport_capability_class` | tier1/07_Bidi 適合仕様 conformance_class（双方向 lock） |
| `auto_instrumentation_mode` | tier1/09_観測適合仕様 semconv_version（双方向 lock） |
| `refresh_token_storage_mode` | tier1/10_認証適合仕様 SessionContext + tier1/11_鍵管理適合仕様 device_bound_key（共有） |
| `retry_policy_class`（method 単位） | tier1/13_SLO 適合仕様 deadline propagation（共有） |
| `retry_budget` | tier1/15_テナント容量適合仕様 noisy_neighbor_isolation（双方向 lock） |
| Buf workflow + method annotation | tier1/12_スキーマ進化適合仕様 compatibility（共有） |
| `build_provenance_class` 注釈 | 10_security/17_build_provenance 適合仕様 5 class（applies_to_distribution_class、双方向 lock） |
| Cosign signed package | 10_security/15_脅威モデル適合仕様 supply chain attestation + 17_build_provenance cosign_rekor_witness_chain |
| `device_at_rest_encryption_mode` | data01_データ保全適合仕様 at_rest_encryption（共有） |
| `capability_matrix.lock.yaml` | tier1/07/03 server actual capability descriptor（双方向 lock） |
| `sdk_distribution_class` 注釈 | infra/15_クラスタ位相適合仕様 topology_class（cross-axis enumerate、preferred_locality routing） |
| 4 layer reducer の SDK 投影 | tier3/39_クライアント状態適合仕様 4 layer / conflict tree（SDK が 39 を 1:1 投影） |
| `sdk_conformance.lock.yaml` | 00_軸登録適合仕様 `release_gate.lock.yaml` AND-gate（cosign signed tag が物理 prerequisite） |
| time API surface（monotonic_now / hlc_now のみ） | 08_infra/16_時刻整合適合仕様 `v1_application_hlc_only` / `v1_browser_client_skew_tolerant`（双方向 lock） |

### sdk_distribution_class と clock_integrity_class の写像
- `v1_full_native_with_companion` → `v1_application_hlc_only`
- `v1_legacy_dotnet_framework` → `v1_browser_client_skew_tolerant`
- `v1_browser_spa_typescript` → `v1_browser_client_skew_tolerant`
- `v1_thick_native_via_tauri` → `v1_application_hlc_only`
- `v1_thin_business_api_only` → `v1_browser_client_skew_tolerant`

deadline propagation の clock source は呼出元の monotonic + sender-issued HLC + receiver-side HLC compare。RPC deadline = `monotonic_now() + timeout_ms` で算出し、wire 上は HLC tuple として propagate（wall-clock subtraction を禁止）。

## 採用しない設計
- sdk_distribution_class dimension override
- 言語ごとの独立 SDK 開発（手書き stub）
- SDK 公開 API での auto-instr 完全 disable
- localStorage / SessionStorage への PII 平文書込
- 自製 crypto / 生 HTTP getter / access_token getter

## 関連参照
- [client 設計方針 index](../../03_概要設計/09_client設計方針/README.md)
- [client 強制機構](../02_強制機構/08_client強制機構.md)
- [Companion OTel 拡張](../03_クロスカッティング適合仕様/11_companion_otel_extension.md)
- [Connect-RPC UA-aware adapter](../03_クロスカッティング適合仕様/12_UA_aware_adapter.md)
- [.NET 8 LTS Connect-RPC 自製実装](../03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
- [build_provenance 適合仕様](16_build_provenance適合仕様.md)
- [時刻整合適合仕様](13_時刻整合適合仕様.md)
