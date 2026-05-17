---
id: detail.tier2.tier2_enforcement
axis: tier2
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.tier2.tier2_index
  - arch.tier2.industry_extension_model
  - arch.tier2.multi_tenant_policy
  - arch.tier2.business_asset_ownership
  - arch.tier2.business_error_audit_compliance
  - detail.tier1.observability_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier2 強制機構

## 一文方針
- tier2 facade 規律（OSS 直接利用禁止 / tier1 経由必須 / 業界中立性 / 拡張点 contract / テナント識別子強制注入 / Domain Event emit 必須化 / atomic 三表書込）は CI / lint / 公開 API snapshot / cross-tenant integration test / PostgreSQL RLS + pgaudit / Kyverno admission policy の 8 層多重防御で物理 enforce、手書き drift / dimension override / dead spec はすべて CI fail。

## 8 層 defense-in-depth

### 層 1: tier3 リポジトリ側
- tier3 が「tier2 経由」を回避して tier1 Library / Companion / OSS を直接利用する経路を、各言語の標準ツールで機械的に禁止
  - Rust: `cargo-deny` の `[bans]` で tier1 Library クライアント crate / OSS クライアント crate を deny。tier2 Library crate のみを許可
  - C# (.NET 8+): BannedApiAnalyzers の `BannedSymbols.txt` で tier1 / OSS の名前空間を禁止。Central Package Management で tier2 NuGet のみ参照許可
  - Go: golangci-lint の `depguard` で tier1 / OSS パッケージの import を deny
  - TypeScript: `eslint-plugin-import` の `no-restricted-imports` と `eslint-plugin-boundaries` で tier1 / OSS パッケージを禁止
- 上記設定は Backstage の Software Template に同梱、tier3 リポジトリ生成時点で組み込む
- tier3 リポジトリの CI で拡張点 contract test を必須実行。skip / 改変は CI fail

### 層 2: tier2 公開 API 表面（業界中立性）
- 業界横断層の公開 API snapshot に業界固有語が出現したら CI fail
  - Rust: `cargo public-api`
  - C#: `Microsoft.CodeAnalysis.PublicApiAnalyzers`
  - Go: `go-apidiff` と自製 `go/analysis`
  - TypeScript: `api-extractor` の `.d.ts` rollup
- 禁止語リストは業界 pack ごとに YAML 宣言、CI で照合
- 拡張点 / 不変条件 YAML 宣言と公開 API snapshot を CI で照合し、宣言と実装の乖離を fail

### 層 3: tier2 内部の依存方向
- 業界横断層 → 業界 pack の依存を機械的に禁止。業界 pack 間の相互依存も禁止
  - Rust: cargo-deny / workspace 構成
  - C# (.NET 8+): ProjectReference 方向検査 + Roslyn analyzer
  - Go: depguard
  - TypeScript: eslint-plugin-boundaries
- tier2 内部のドメイン層から tier1 Library / Service への直接依存を禁止（クリーンアーキテクチャの依存方向）

### 層 4: 第二業界 stub conformance
- 業界横断層の API が第二業界 stub（CI 専用、サービス業想定）から素直に消費できることを CI 必須
- stub で破綻する API はその時点で「業界横断層に適格でない」とみなし、業界共通 Lv 以下に降格させる（major version up として扱う）

### 層 5: tier2 リポジトリ抽象による強制注入
- `tenant_id` 述語の強制注入:
  - tier2 リポジトリ抽象は `tenant_id` 述語を強制注入
  - 業務コードから `tenant_id` を省略した検索 / 更新を行う API を提供しない
  - 「テナント識別子を引数で受ける API」を tier2 公開 API に置くことを CI で禁止。識別子は必ず Authorization context から取得
- 業務イベント emit の必須化:
  - aggregate 状態変更は必ず Domain Event を emit する経路に乗る
  - emit を抑制する API（raw SQL 実行等）を tier2 公開 API に置かない
- 上記 2 項は方針宣言に留め、static / runtime 二重防御の機械可読仕様（compile 時 Repository abstraction、lint、cross-tenant integration test、PostgreSQL Row Level Security、pgaudit の 5 層）は [テナント分離適合仕様](../01_適合仕様/10_テナント分離適合仕様.md) に分離。本層 5 の検査は同仕様の整合 1〜10 を CI 必須として組み込むことで実体化

### 層 6: 供給経路（内部レジストリ）
- tier1 と同枠で、tier2 配布物（NuGet / crate / Go module / npm package / OCI image）も内部レジストリを唯一の取得元に固定
  - crate: 内部 cargo registry を `[source.crates-io]` で置換
  - C#: 内部 NuGet feed のみ `nuget.config` に登録
  - Go: 内部 Athens module proxy を `GOPROXY` に固定
  - TypeScript: 内部 Verdaccio を npm registry に固定
  - Service image: 内部 Harbor を OCI registry に固定
- 内部レジストリには tier1 / tier2 の承認済み配布物のみを公開

### 層 7: Library / Service 等価性
- 同一機能を Library / Service の両形態で提供するカテゴリは、共通 conformance test suite で両方 green を merge 条件
- 等価性破れを検出した API は両形態併存対象から外し、Library 専用 / Service 専用にカタログ上で明示分類

### 層 8: 監査 / 観測可能性のスキーマ整合
- tier2 が emit する span / metric / 監査エントリの属性スキーマを OpenTelemetry Semantic Conventions の tier2 拡張として宣言し、各言語の定数コードを OpenTelemetry Weaver で生成
- 実行時に emit される属性をゴールデン値と CI で照合
- 5 signal class の SoR / 派生関係、4 dimension layer の合成、`trace_id` を primary key とする cross-signal cross-tier 相関契約、PII redaction の単一の真、Collector プロセッサ設定の build artifact 化、Companion 最小 NuGet との接合の機械可読仕様は [観測適合仕様](../01_適合仕様/03_観測適合仕様.md) に分離。`tier2_ext` の `aggregate_class` / `purpose` / `actor_id` / `business_event` の attribute 集合も同仕様が保持

## サプライチェーン
- tier1 と整合し、Harbor / Cosign / Trivy / SBOM 生成 を tier2 配布物にも適用
- 各層の CI 設定は Backstage の Software Template に同梱して、tier2 / tier3 リポジトリ生成時点で組み込む

## CI 不変条件（tier2 全体、merge 不可）
- 整合 1: 全 tier3 repository の OSS / tier1 直接 import がゼロ
- 整合 2: 業界横断層の公開 API snapshot に業界固有語ゼロ
- 整合 3: 業界横断層 → 業界 pack の依存ゼロ、業界 pack 間相互依存ゼロ
- 整合 4: 第二業界 stub conformance test green
- 整合 5: 全 tier2 公開 API で `tenant_id` 省略可 API ゼロ、SQL 文字列受付 API ゼロ
- 整合 6: 全 tier2 配布物が内部レジストリ経由
- 整合 7: Library / Service 両形態提供カテゴリの conformance test 両 green
- 整合 8: 観測スキーマ Weaver 生成定数とゴールデン値一致
- 整合 9: dead spec 検出

## 1.0.0 ship blocker
- 全 8 層の green
- テナント分離適合仕様 10 整合 1〜10 green
- protoc-gen-go-fsm 4 言語等価強度 green（[protoc_gen_go_fsm](../03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)）
- SLO protection layers 4 層 + 自動昇格 green（[SLO_protection_layers](../03_クロスカッティング適合仕様/05_SLO_protection_layers.md)）

## 採用しない強制機構
- 文章のみの規律（必ず CI / lint / build artifact / Kyverno に物理転写）
- OSS 直接 import の例外承認 path
- tier2 公開 API での dev mode runtime check 緩和

## 至高路線における立ち位置
- tier2 facade 規律は文章に依存しない。8 層多重防御で物理 enforce
- 1.0.0 release 時に 9 不変条件 + 8 層が全 green であることを物理 ship blocker

## 関連参照
- [tier2 設計方針 index](../../03_概要設計/03_tier2設計方針/README.md)
- [業界拡張モデル](../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md)
- [マルチテナント方針](../../03_概要設計/03_tier2設計方針/05_マルチテナント方針.md)
- [業務エラー監査コンプライアンス](../../03_概要設計/03_tier2設計方針/08_業務エラー監査コンプライアンス.md)
- [テナント分離適合仕様](../01_適合仕様/10_テナント分離適合仕様.md)
- [protoc_gen_go_fsm](../03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- [SLO_protection_layers](../03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
