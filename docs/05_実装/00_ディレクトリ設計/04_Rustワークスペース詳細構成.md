# 04. Rust ワークスペース詳細構成

本ファイルは `src/tier1/rust/` 配下の詳細ファイル配置を方式として固定化する。上流は概要設計 [DS-SW-COMP-129〜134（Cargo workspace・ビルド・Docker image）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)・[DS-SW-COMP-050〜079（自作 Rust 領域 3 Pod）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/03_自作Rust領域コンポーネント.md)・[DS-SW-COMP-112（Rust crate 公開範囲）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md)で、本ファイルは `crates/` 配下を **`pods/`（Pod 実装 bin crate）・`shared/`（横断 lib crate）・`sdk/`（外部公開 wrapper crate）の 3 カテゴリ**に分けたうえで、各 bin crate 内部を 4 層（domain / service / adapter / grpc）まで分解する。

## 本ファイルの位置付け

概要設計は Cargo workspace の `members` として `audit` / `decision` / `pii` / `common` / `proto-gen` / `otel-util` / `policy` の 7 crate を方式として確定した。本ファイルはその下の 2 階層を確定させる。第 1 階層として、Pod 実装である bin crate（`audit` / `decision` / `pii`）を `crates/pods/` 配下、横断 lib crate（`common` / `proto-gen` / `otel-util` / `policy`）を `crates/shared/` 配下、Phase 1b 以降に追加する外部公開 SDK wrapper crate を `crates/sdk/` 配下に分類する。第 2 階層として、bin crate 内部を `src/grpc/` / `src/service/` / `src/domain/` / `src/adapter/` の 4 層に分割する。lib crate は機能別モジュール分割とする。

Rust の場合、Go と違って「4 層アーキテクチャ」の慣例が弱く、プロジェクトごとに `handler` / `usecase` / `model` / `infra` や `controller` / `service` / `entity` / `repository` とバラバラの命名になりがちである。本ファイルで Go 側と揃った命名（ただし Rust 慣習に合わせて単語を調整）を強制することで、「Go 開発者が Rust コードを読む / Rust 開発者が Go コードを読む」時の認知負荷を下げる。

3 カテゴリ分離の根拠は 3 点ある。第 1 に、`pods/` と `shared/` は CODEOWNERS の要件が異なり（`shared/` はアーキテクト承認必須、`pods/` は各 Pod チーム自走）、物理配置で責務と承認権限を一致させることで PR レビューの流れが自然になる。第 2 に、Phase 1b で追加する `sdk/k1s0-sdk/` は外部公開 crate として独立した API 安定性契約を持つため、内部実装 crate（`shared/`）と混在させると破壊的変更の境界が曖昧になる。第 3 に、Phase 2 以降で新規 Pod（例: `auditjob`）を追加する際、`pods/` 配下に crate を 1 つ加えるだけで workspace の整合性が保たれ、`shared/` を触る必要がない（Pod 追加の線形コスト化）。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-081` 〜 `DS-IMPL-DIR-120` の 40 件である。

## Cargo workspace 全体構造

`src/tier1/rust/` の完全な構造は以下のとおり。`crates/` 直下は `pods/` / `shared/` / `sdk/` の 3 ディレクトリに分ける。

```
src/tier1/rust/
├── Cargo.toml                              # workspace 設定
├── Cargo.lock
├── rust-toolchain.toml                     # Rust 1.85 pin
├── README.md                               # Cargo workspace 概要とビルド手順
├── .cargo/
│   └── config.toml                         # rustflags / registry 設定
├── deny.toml                               # cargo-deny 設定（ライセンス・禁止 crate）
├── clippy.toml                             # clippy 設定
├── rustfmt.toml                            # rustfmt 設定
└── crates/
    ├── shared/                             # 横断 lib crate（変更頻度低・アーキテクト承認必須）
    │   ├── common/                         # k1s0-common（業務非依存ユーティリティ）
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   └── src/
    │   │       ├── lib.rs
    │   │       ├── id.rs                   # TenantID / UserID 型
    │   │       ├── clock.rs                # 時刻抽象
    │   │       ├── config.rs               # 設定ロード
    │   │       ├── context.rs              # request context 拡張
    │   │       └── error.rs                # 共通エラー型
    │   ├── proto-gen/                      # k1s0-proto（buf / tonic-build 生成）
    │   │   ├── Cargo.toml
    │   │   ├── build.rs                    # tonic-build 実行（drift 検証）
    │   │   ├── README.md
    │   │   └── src/
    │   │       ├── lib.rs                  # pub mod state / ... の手動 include
    │   │       └── generated/              # buf generate 出力（git 管理）
    │   │           └── k1s0.*.v1.rs        # 各 API の生成コード
    │   ├── otel-util/                      # k1s0-otel（OpenTelemetry 初期化）
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   └── src/
    │   │       ├── lib.rs
    │   │       ├── init.rs                 # Pod 起動時の初期化
    │   │       ├── trace.rs                # span ユーティリティ
    │   │       └── metric.rs               # meter ユーティリティ
    │   └── policy/                         # k1s0-policy（認証・認可・tenant）
    │       ├── Cargo.toml
    │       ├── README.md
    │       └── src/
    │           ├── lib.rs
    │           ├── jwt.rs                  # JWT 検証
    │           ├── tenant.rs               # Tenant 境界
    │           ├── rbac.rs                 # OPA Rego 評価
    │           ├── ratelimit.rs            # Rate Limit（Phase 1b）
    │           ├── idempotency.rs          # 冪等性（Phase 1c）
    │           └── interceptor.rs          # tonic Interceptor 実装
    ├── pods/                               # Pod 実装 bin crate（機能追加で頻繁に変更）
    │   ├── audit/                          # COMP-T1-AUDIT（bin crate）
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── migrations/                 # sqlx マイグレーション（PostgreSQL）
    │   │   │   ├── 0001_audit_events.sql
    │   │   │   ├── 0002_audit_hash_chain.sql
    │   │   │   └── 0003_audit_tenant_head.sql
    │   │   ├── src/
    │   │   │   ├── main.rs                 # Pod entrypoint（100 行以内）
    │   │   │   ├── lib.rs                  # Pod 内 public API（テスト用）
    │   │   │   ├── grpc/                   # tonic gRPC handler（gRPC 入口層）
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── audit_handler.rs
    │   │   │   │   └── log_handler.rs
    │   │   │   ├── service/                # ユースケース層
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── audit_service.rs
    │   │   │   │   └── log_service.rs
    │   │   │   ├── domain/                 # ドメインモデル（pure）
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── audit_event.rs
    │   │   │   │   ├── hash_chain.rs
    │   │   │   │   └── tenant_head.rs
    │   │   │   ├── adapter/                # 外部システム接続層
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── pg.rs               # PostgreSQL（sqlx）
    │   │   │   │   ├── kafka.rs            # Kafka Consumer（rdkafka）
    │   │   │   │   ├── valkey.rs           # Valkey（dedup SET）
    │   │   │   │   └── minio.rs            # MinIO（cold archive）
    │   │   │   └── config.rs               # Pod 固有設定
    │   │   ├── tests/                      # 統合テスト（Pod 内完結）
    │   │   │   ├── common/
    │   │   │   │   └── mod.rs              # testcontainers ヘルパ
    │   │   │   ├── audit_service_test.rs
    │   │   │   └── hash_chain_test.rs
    │   │   ├── benches/                    # Criterion ベンチマーク
    │   │   │   ├── hash_chain_bench.rs
    │   │   │   └── serialization_bench.rs
    │   │   └── examples/                   # 使い方サンプル
    │   │       └── verify_chain.rs
    │   ├── decision/                       # COMP-T1-DECISION（bin crate）
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src/
    │   │   │   ├── main.rs
    │   │   │   ├── lib.rs
    │   │   │   ├── grpc/
    │   │   │   │   ├── mod.rs
    │   │   │   │   └── decision_handler.rs
    │   │   │   ├── service/
    │   │   │   │   ├── mod.rs
    │   │   │   │   └── decision_service.rs
    │   │   │   ├── domain/
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── decision_request.rs
    │   │   │   │   └── decision_result.rs
    │   │   │   ├── adapter/
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── zen_engine.rs       # ZEN Engine 統合
    │   │   │   │   ├── pii.rs              # PII Pod への gRPC クライアント
    │   │   │   │   └── rule_store.rs       # ルール定義の永続化
    │   │   │   └── config.rs
    │   │   ├── tests/
    │   │   │   ├── common/
    │   │   │   ├── decision_service_test.rs
    │   │   │   └── zen_integration_test.rs
    │   │   ├── benches/
    │   │   │   └── zen_eval_bench.rs
    │   │   ├── examples/
    │   │   │   └── evaluate_sample.rs
    │   │   └── rules/                      # サンプルルール（開発用）
    │   │       └── dev/
    │   │           ├── credit_approval.json
    │   │           └── expense_policy.json
    │   └── pii/                            # COMP-T1-PII（bin crate）
    │       ├── Cargo.toml
    │       ├── README.md
    │       ├── src/
    │       │   ├── main.rs
    │       │   ├── lib.rs
    │       │   ├── grpc/
    │       │   │   ├── mod.rs
    │       │   │   └── pii_handler.rs
    │       │   ├── service/
    │       │   │   ├── mod.rs
    │       │   │   ├── mask_service.rs
    │       │   │   └── detect_service.rs
    │       │   ├── domain/
    │       │   │   ├── mod.rs
    │       │   │   ├── pii_category.rs     # PII カテゴリ enum
    │       │   │   ├── masking_policy.rs
    │       │   │   └── detection_result.rs
    │       │   ├── adapter/
    │       │   │   ├── mod.rs
    │       │   │   ├── detector.rs         # regex ベース detector
    │       │   │   ├── aes_siv.rs          # AES-SIV 暗号化
    │       │   │   └── audit_publisher.rs  # Audit Pod への通知（Kafka publish）
    │       │   └── config.rs
    │       ├── tests/
    │       │   ├── common/
    │       │   ├── mask_service_test.rs
    │       │   └── detect_service_test.rs
    │       ├── benches/
    │       │   ├── regex_detect_bench.rs
    │       │   └── aes_siv_bench.rs
    │       └── examples/
    │           └── mask_json.rs
    └── sdk/                                # 外部公開 wrapper crate（Phase 1b〜）
        └── k1s0-sdk/                       # tier2/tier3 Rust 消費者向け SDK（現時点は骨格のみ）
            ├── Cargo.toml
            ├── README.md
            └── src/
                └── lib.rs                  # Phase 1b で再 export を定義
```

### DS-IMPL-DIR-081 Cargo workspace root のファイル構成

`src/tier1/rust/` 直下には以下 8 ファイルと 2 ディレクトリを配置する。

- `Cargo.toml` — workspace 設定
- `Cargo.lock` — ロック
- `rust-toolchain.toml` — Rust バージョン pin
- `README.md` — workspace 概要
- `.cargo/config.toml` — rustflags
- `deny.toml` — cargo-deny 設定
- `clippy.toml` — clippy 設定
- `rustfmt.toml` — rustfmt 設定
- `crates/` — 全 crate（配下は `pods/` / `shared/` / `sdk/` の 3 カテゴリ）
- （ビルド成果物 `target/` は .gitignore 対象）

root 直下に `Cargo.toml` 以外の crate ファイル（`src/` や `main.rs`）を置くことは禁止する。理由は workspace root が個別 crate と区別できなくなり、`cargo build` の挙動が混乱するためである。`crates/` 直下に crate を直接置くことも禁止する（必ず `pods/` / `shared/` / `sdk/` のいずれかのサブカテゴリ配下に置く）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-129。

### DS-IMPL-DIR-082 Cargo.toml の [workspace] 設定

root の `Cargo.toml` の `[workspace]` セクションは以下の構造で宣言する。

```toml
[workspace]
resolver = "3"
members = [
    "crates/shared/common",
    "crates/shared/proto-gen",
    "crates/shared/otel-util",
    "crates/shared/policy",
    "crates/pods/audit",
    "crates/pods/decision",
    "crates/pods/pii",
    # Phase 1b 以降
    # "crates/sdk/k1s0-sdk",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"
version = "0.1.0"
authors = ["k1s0 tier1 team"]
license = "Apache-2.0"
repository = "https://github.com/k1s0/k1s0"

[workspace.dependencies]
# 非同期ランタイム
tokio = { version = "1.37", features = ["rt-multi-thread", "macros", "signal", "sync", "time"] }
# gRPC
tonic = "0.12"
prost = "0.13"
tonic-build = "0.12"
# シリアライゼーション
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ciborium = "0.2"  # canonical CBOR（Audit hash 用）
# 観測性
tracing = "0.1"
tracing-opentelemetry = "0.27"
opentelemetry = "0.27"
opentelemetry-otlp = "0.27"
prometheus = "0.13"
# 暗号
sha2 = "0.10"
aes-siv = "0.7"
# データベース（AUDIT 専用）
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "migrate"] }
# Kafka（AUDIT 専用）
rdkafka = { version = "0.36", features = ["tokio"] }
# ZEN Engine（DECISION 専用）
zen-engine = "0.37"
# PII 検出（PII 専用）
regex = "1.10"
# テスト
anyhow = "1.0"
thiserror = "1.0"
testcontainers = "0.20"
```

`resolver = "3"` は Rust 1.84 以降の新 resolver。Edition は 2024（プロジェクト規約）。`[workspace.dependencies]` で全外部 crate のバージョンを集約し、子 crate では `tokio.workspace = true` の形式で参照する（DS-SW-COMP-130）。`members` は glob（`crates/pods/*` 等）ではなく明示列挙することで、新規 crate 追加が必ず ADR を通る運用を担保する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-E-ENC-001、ADR-TIER1-001。**上流**: DS-SW-COMP-130。

### DS-IMPL-DIR-083 rust-toolchain.toml の内容

`rust-toolchain.toml` で Rust バージョンと components / targets を固定する。

```toml
[toolchain]
channel = "1.85.0"
components = ["rustfmt", "clippy", "rust-src"]
targets = ["x86_64-unknown-linux-gnu"]
profile = "default"
```

Phase 2 で ARM 対応を追加する際は `aarch64-unknown-linux-gnu` を targets に追加する。channel は固定バージョン（`1.85.0`）を使い、`stable` / `beta` のような浮動チャンネルは使わない（CI の再現性確保のため）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-133。

### DS-IMPL-DIR-084 .cargo/config.toml の内容

`.cargo/config.toml` は以下を指定する。

```toml
[build]
rustflags = ["-D", "warnings"]  # CI では warning を error 扱い

[net]
git-fetch-with-cli = true  # proxy 環境で git fetch を安定化

[registries.crates-io]
protocol = "sparse"  # 高速 index
```

`rustflags = ["-D", "warnings"]` は CI ビルドで warnings を fail させる。開発者ローカルでは `CARGO_BUILD_RUSTFLAGS=""` で上書きして warnings を許容できる（Lint は `cargo clippy` 側で fail させる分担）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-133。

### DS-IMPL-DIR-085 deny.toml の内容

`deny.toml` は cargo-deny の設定で、以下を宣言する。

```toml
[licenses]
allow = ["Apache-2.0", "MIT", "BSD-3-Clause", "BSD-2-Clause", "ISC", "Unicode-3.0", "MPL-2.0"]
confidence-threshold = 0.95

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl" },       # rustls を強制
    { name = "openssl-sys" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]

[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
```

`openssl` を deny し、TLS は `rustls` に統一する。理由は libressl / openssl の動的リンクが distroless container と相性が悪く、かつ RUSTSEC 勧告のヒット率が openssl 側に偏るためである。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-E-ENC-001、NFR-SUP-\*。**上流**: DS-SW-COMP-114。

### DS-IMPL-DIR-086 clippy.toml の内容

`clippy.toml` は以下を指定する。

```toml
# 300 行を厳密に検査するわけではないが、大規模関数の検出に使う
too-many-arguments-threshold = 7
type-complexity-threshold = 250
cognitive-complexity-threshold = 30
```

さらに `Cargo.toml` の `[lints.clippy]` で以下を宣言する（workspace 継承）。

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
cargo = { level = "warn", priority = -1 }
# 個別抑制
module_name_repetitions = "allow"
missing_errors_doc = "allow"
```

`pedantic` を warn レベルで有効化し、違反は CI の `rustflags = ["-D", "warnings"]` で fail させる。`module_name_repetitions` を allow するのは、`audit_service.rs` 内の `audit::service::AuditService` のような命名を許容するためである。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-114。

### DS-IMPL-DIR-087 rustfmt.toml の内容

`rustfmt.toml` は以下を指定する。

```toml
edition = "2024"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
```

`imports_granularity = "Crate"` で `use std::{ io, fs };` のように同一 crate からの import を畳む。`group_imports = "StdExternalCrate"` で std / external / crate の 3 グループに分けて空行で区切る。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-130。

## shared/ 配下の lib crate 内部構造

### DS-IMPL-DIR-088 shared/common crate の配置

`crates/shared/common/` は lib crate で、業務非依存ユーティリティを提供する。ファイル分割は責務別（`id.rs` / `clock.rs` / `config.rs` / `context.rs` / `error.rs`）とし、各ファイルは 200 行以内。`lib.rs` は `pub mod id; pub mod clock; ...` の宣言のみで、ロジックを含まない。

```rust
// crates/shared/common/src/lib.rs
pub mod clock;
pub mod config;
pub mod context;
pub mod error;
pub mod id;

pub use error::{Error, Result};
```

Go 側の `internal/shared/common/` と API セマンティクスを揃える（Go は `NewTenantID()`、Rust は `TenantID::new()` のように snake_case / camelCase の差のみ）。全 Pod から依存される最頻参照 crate であるため、破壊的変更は ADR 必須とし、deprecation は Rust の `#[deprecated]` アトリビュートで 1 マイナー版以上維持する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-GP-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-108、DS-IMPL-DIR-050。

### DS-IMPL-DIR-089 shared/proto-gen crate の配置

`crates/shared/proto-gen/` は DS-IMPL-DIR-031 で詳細を確定済み。本ファイルでは再掲せず、[02_contracts詳細構成.md](02_contracts詳細構成.md) の記述を上位参照する。ファイル配置は以下。

- `Cargo.toml` — prost / tonic 依存
- `build.rs` — tonic-build 実行（drift 検証用）
- `src/lib.rs` — `pub mod v1 { pub mod <api> ... }` の階層 include（DS-IMPL-DIR-030）
- `src/generated/k1s0.*.v1.rs` — buf 生成コード（flat ファイル、prost 仕様）

02 章で規定した `buf.gen.yaml` の `out:` 指定を `src/tier1/rust/crates/shared/proto-gen/src/generated` に更新する必要がある（02 章 DS-IMPL-DIR-031 と整合を取る）。

**Go 側との対称性**: Go 側の生成コードは `paths=source_relative` により `shared/proto/v1/<api>/*.pb.go` のディレクトリ構造で配置される一方、Rust 側は prost 仕様上 flat ファイル命名が強制されるため物理配置が非対称になる。この非対称は `lib.rs` の `pub mod v1::<api>` 階層で吸収し、**Pod コードから見た module 階層は両言語で同一**（`v1::<api>::<Type>`）にする設計である。API の片側追加・削除は `tools/check-proto-symmetry/`（DS-IMPL-DIR-031）で CI ブロックする。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-132、DS-IMPL-DIR-030、DS-IMPL-DIR-031。

### DS-IMPL-DIR-090 shared/otel-util crate の配置

`crates/shared/otel-util/` は OpenTelemetry 初期化を配置する。ファイル分割は `init.rs`（Pod 起動時の初期化、`init(pod_name, namespace)` を公開）、`trace.rs`（span 生成ユーティリティ）、`metric.rs`（meter 生成ユーティリティ）の 3 ファイル構成。各ファイル 200 行以内。`tracing` + `tracing-opentelemetry` + `opentelemetry-otlp` + `prometheus` を依存し、全自作 Rust Pod から統一 API で呼び出す。

**確定フェーズ**: Phase 1a。**対応要件**: FR-T1-TELEMETRY-\*、NFR-D-MON-\*、NFR-D-TRACE-\*。**上流**: DS-SW-COMP-109、DS-SW-COMP-050。

### DS-IMPL-DIR-091 shared/policy crate の配置

`crates/shared/policy/` は Go 側の `internal/shared/policy/` と対称で、サブモジュール別に分割する。

- `jwt.rs` — JWT 検証（`jsonwebtoken` crate）
- `tenant.rs` — Tenant 境界確認
- `rbac.rs` — OPA Rego 評価（`openpolicyagent/rego` の Rust bindings、Phase 1b）
- `ratelimit.rs` — Rate Limit（Phase 1b）
- `idempotency.rs` — 冪等性（Phase 1c）
- `interceptor.rs` — tonic Interceptor 実装（全自作 Pod に組み込む）

`interceptor.rs` は `tonic::service::interceptor_fn` を使い、認証/認可/metric/trace の 4 段階を全リクエストに適用する（DS-SW-COMP-051 の Interceptor 仕様に準拠）。Phase 1a では JWT + Tenant のみ実装。

**確定フェーズ**: Phase 1a/1b/1c。**対応要件**: NFR-E-AC-\*、DX-GP-\*。**上流**: DS-SW-COMP-110、DS-SW-COMP-051。

## pods/ 配下の bin crate 4 層アーキテクチャ

### DS-IMPL-DIR-092 bin crate の 4 層（grpc / service / domain / adapter）

各 bin crate（`pods/audit` / `pods/decision` / `pods/pii`）の `src/` 配下は 4 層で統一する。Go 側の 4 層（handler / service / domain / repository）と対応関係を持つが、Rust 慣習に合わせて命名を調整する。

| 層（Rust） | 層（Go 対応） | 責務 |
|---|---|---|
| `grpc/` | `handler/` | tonic gRPC handler、proto 型 ↔ domain 型の変換、エラー変換 |
| `service/` | `service/` | ユースケースの実装、Policy Interceptor との連携 |
| `domain/` | `domain/` | ドメインモデル（pure Rust 構造体）、不変条件 |
| `adapter/` | `repository/` | 外部システム（PG / Kafka / ZEN / regex）への接続 |

Rust では「handler」は axum / actix-web 等の HTTP handler と紛らわしいため `grpc/` に改名する。Go の「repository」は Rust では永続化に限らない外部接続（Kafka / regex 等）を含むため `adapter/` に統一する（concept 的には clean architecture の infrastructure 層）。`pods/<pod>/` から他 Pod の crate（`pods/<other>/`）への path 依存は禁止し、Pod 間通信は gRPC 経由に強制する（Pod 独立性の物理的強制）。この制約は `Cargo.toml` の `[dependencies]` を CI で機械検査することで担保する（DS-IMPL-DIR-114 参照）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-050〜051、DS-IMPL-DIR-054。

### DS-IMPL-DIR-093 grpc/ の配置

`crates/pods/<crate>/src/grpc/` は tonic gRPC handler を配置する。ファイル分割は API 単位で、各ファイルは 200 行以内。例えば `pods/audit` crate は `audit_handler.rs`（Audit API）と `log_handler.rs`（Log API）の 2 ファイルを持つ。`mod.rs` には `pub mod audit_handler; pub mod log_handler;` のみを書く。各 handler は `tonic::async_trait` で proto 生成の `<Service>Server` trait を実装する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-051、DS-IMPL-DIR-055。

### DS-IMPL-DIR-094 service/ の配置

`crates/pods/<crate>/src/service/` はユースケース層を配置する。ファイル分割は grpc と 1:1 対応で、各ファイル 250 行以内。service 構造体は `Arc<dyn XxxAdapter>` を内部に持ち、依存性注入で test 時に mock に差し替えられる構造とする。Go の Wire DI と異なり、Rust では手動で `Arc::new()` する（DI フレームワーク不要）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-050、DS-IMPL-DIR-056。

### DS-IMPL-DIR-095 domain/ の配置

`crates/pods/<crate>/src/domain/` はドメインモデルを配置する。ファイル分割はドメイン概念単位で、各ファイル 150 行以内。`pods/audit` crate の domain は `audit_event.rs`（AuditEvent 構造体）、`hash_chain.rs`（ハッシュチェーン計算、SHA-256 と CBOR）、`tenant_head.rs`（テナント単位の先頭 hash 管理）の 3 ファイル。domain は pure Rust のみで、外部 crate は `serde` / `chrono` / `sha2` / `ciborium` 等の **計算に必要な library 型**のみを許可し、`sqlx` / `rdkafka` / `tonic` 等の **通信・永続化 library** を domain に含めることは禁止する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*、NFR-H-INT-001。**上流**: DS-SW-COMP-054〜055、DS-IMPL-DIR-057。

### DS-IMPL-DIR-096 adapter/ の配置

`crates/pods/<crate>/src/adapter/` は外部システム接続層を配置する。各 adapter は 1 つの外部システムに対応し、200 行以内。例えば `pods/audit` crate の adapter は `pg.rs`（PostgreSQL、sqlx）、`kafka.rs`（Kafka Consumer、rdkafka）、`valkey.rs`（Valkey dedup、redis crate）、`minio.rs`（MinIO cold archive、aws-sdk-s3）の 4 ファイル。各 adapter は trait（例: `PgAdapter`）を公開し、実装（`PgAdapterImpl`）と分離する。service は trait 側を使うことで、test 時に mock 実装に差し替えられる。

**確定フェーズ**: Phase 1a（pg / kafka）、Phase 1c（valkey / minio）。**対応要件**: DX-TEST-\*、NFR-A-CONT-003。**上流**: DS-SW-COMP-054、DS-IMPL-DIR-058。

## 各 bin crate の固有配置

### DS-IMPL-DIR-097 pods/audit crate の migrations/ 配置

`crates/pods/audit/migrations/` は sqlx のマイグレーションファイルを配置する。ファイル命名は `NNNN_<name>.sql`（0001_audit_events.sql / 0002_audit_hash_chain.sql / ...）で、4 桁連番。sqlx は `sqlx migrate add <name>` で自動生成し、`sqlx migrate run --database-url=<url>` で適用する。migration は追加のみ許可し、既存 migration の編集は禁止する（運用開始後に schema drift を起こさないため）。schema 変更が必要な場合は新 migration を追加する。

**確定フェーズ**: Phase 1b。**対応要件**: FR-T1-AUDIT-\*、NFR-H-INT-001。**上流**: DS-SW-COMP-054〜056。

### DS-IMPL-DIR-098 pods/audit crate の内部モジュール対応

`pods/audit` crate の 4 層は DS-SW-COMP-054 の 5 モジュール（kafka_consumer / dedup / hash_chainer / pg_writer / minio_archiver）と以下の対応で配置する。

- grpc/audit_handler.rs — Log API の gRPC handler（tier1 ファサードから直接呼ばれる監査ログ取得 API）
- service/audit_service.rs — `kafka_consumer` + `dedup` + `hash_chainer` の orchestration
- service/log_service.rs — Log API の service（検索・取得）
- domain/audit_event.rs — イベント型
- domain/hash_chain.rs — `hash_chainer` のロジック（pure 計算）
- domain/tenant_head.rs — テナント先頭 hash 管理
- adapter/pg.rs — `pg_writer` 実装
- adapter/kafka.rs — `kafka_consumer` 実装
- adapter/valkey.rs — `dedup` 実装
- adapter/minio.rs — `minio_archiver` 実装

5 モジュールの呼び出し順序（kafka → dedup → hash → pg）は service 層の `audit_service.rs` が orchestrate する。

**確定フェーズ**: Phase 1b（kafka + hash + pg）、Phase 1c（dedup + minio）。**対応要件**: FR-T1-AUDIT-\*、NFR-H-INT-001。**上流**: DS-SW-COMP-054。

### DS-IMPL-DIR-099 pods/decision crate の rules/ 配置

`crates/pods/decision/rules/` は DECISION Pod の評価対象となる ZEN ルール定義のサンプルを配置する。本番では Backstage からアップロードされたルールを DB / S3 に保存するが、開発時のテスト用に `rules/dev/<rule_name>.json` のサンプルを置く。Phase 1a では `credit_approval.json`（与信判定のサンプル）と `expense_policy.json`（経費規程のサンプル）の 2 ファイル。rules/ は production ビルドには含めない（Dockerfile で除外）。

**確定フェーズ**: Phase 1a。**対応要件**: FR-T1-DECISION-\*、DX-GP-\*。**上流**: DS-SW-COMP-057〜058。

### DS-IMPL-DIR-100 pods/pii crate の検出ルール

`crates/pods/pii/` の PII 検出ルール（正規表現）は `src/adapter/detector.rs` に埋め込むのではなく、`src/domain/pii_category.rs` で PII カテゴリ enum を定義し、各カテゴリに対応する正規表現を `adapter/detector.rs` で `lazy_static!` / `once_cell::Lazy` パターンで保持する。カテゴリは日本固有（マイナンバー、電話番号、郵便番号、クレジットカード、メールアドレス、氏名、住所）を Phase 1a で実装し、Phase 1b で国際対応（SSN / VAT ID 等）を追加する。

**確定フェーズ**: Phase 1a（日本）、Phase 1b（国際）。**対応要件**: FR-T1-PII-\*、NFR-G-PROT-\*。**上流**: DS-SW-COMP-070〜074。

## tests/ と benches/ と examples/

### DS-IMPL-DIR-101 crate 内 tests/ の配置原則

各 crate の `tests/` は**統合テスト**（integration test）を配置する。Rust の慣例で、`tests/` 配下は外部から crate を使うコードとしてコンパイルされるため、crate の public API をテストする場所である。ファイル命名は `<対象モジュール>_test.rs` 形式（例: `audit_service_test.rs`）。各ファイルは 300 行以内に収め、大規模になったらモジュール内部で `mod` 分割する。Pod 内の unit test（private 関数のテスト）は `src/**/*.rs` 内の `#[cfg(test)] mod tests { ... }` で書く。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*。**上流**: DS-SW-COMP-136。

### DS-IMPL-DIR-102 tests/common/ の共通ヘルパ

各 crate の `tests/common/mod.rs` は統合テスト共通のヘルパ（testcontainers 起動、fixture ロード）を配置する。Rust の `tests/` は各ファイルが独立した bin crate としてコンパイルされるため、共通ヘルパを使うには `mod common;` で明示的に取り込む。`common/mod.rs` は `pub fn setup_postgres() -> PgContainer` のような関数を公開する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-TEST-\*、DX-LD-\*。**上流**: DS-SW-COMP-136、DS-IMPL-DIR-065。

### DS-IMPL-DIR-103 benches/ の配置と Criterion

各 bin crate の `benches/` は Criterion ベンチマークを配置する。ファイル命名は `<対象>_bench.rs`。`pods/audit` crate は `hash_chain_bench.rs`（SHA-256 計算速度）と `serialization_bench.rs`（CBOR シリアライズ速度）、`pods/decision` crate は `zen_eval_bench.rs`（ZEN 評価 p99 1ms 目標の継続計測）、`pods/pii` crate は `regex_detect_bench.rs`（regex 検出速度）と `aes_siv_bench.rs`（AES-SIV 暗号速度）。`Cargo.toml` に `[[bench]] name = "hash_chain_bench" harness = false` を追加して Criterion を使う。ベンチは CI で毎 PR 実行せず、nightly で実行する（時間がかかるため）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-B-PERF-\*、DX-TEST-\*。**上流**: DS-SW-COMP-050、DS-SW-COMP-133。

### DS-IMPL-DIR-104 examples/ の配置

各 bin crate の `examples/` は crate の使い方サンプルを配置する。`cargo run --example <name>` で実行可能。`pods/audit` crate は `verify_chain.rs`（既存 hash chain の検証）、`pods/decision` crate は `evaluate_sample.rs`（ZEN ルールのサンプル評価）、`pods/pii` crate は `mask_json.rs`（JSON 中の PII マスキング実演）。examples は Pod 本体から独立した bin として、crate の内部 API を使いたい開発者向けの学習素材として機能する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-GP-\*、DX-LD-\*。**上流**: DS-DEVX-LOCAL-\*、DS-SW-COMP-050。

## bin crate の main.rs 骨格

### DS-IMPL-DIR-105 bin crate の main.rs

各 bin crate の `src/main.rs` は 100 行以内で、以下の 7 ステップのみを記述する。

```rust
// crates/pods/audit/src/main.rs の骨格
#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    // 1. 設定ロード
    let cfg = config::Config::load()?;

    // 2. OTel 初期化
    let _otel_guard = k1s0_otel::init("audit", &cfg.namespace)?;

    // 3. Logger 初期化（tracing）— otel-util 内部で済む

    // 4. Policy Interceptor 準備
    let policy = Arc::new(k1s0_policy::Interceptor::new(&cfg)?);

    // 5. Adapter 初期化
    let pg = Arc::new(adapter::pg::PgAdapterImpl::new(&cfg).await?);
    let kafka = Arc::new(adapter::kafka::KafkaAdapterImpl::new(&cfg)?);

    // 6. Service 組み立て
    let audit_svc = Arc::new(service::AuditService::new(pg.clone(), kafka.clone()));

    // 7. tonic サーバ起動（blocking）
    grpc::serve(cfg, policy, audit_svc).await
}
```

`tokio::main(flavor = "multi_thread")` は DS-SW-COMP-052 の tokio チューニングに準拠。`k1s0_otel::init` は `shared/otel-util` crate の公開 API。handler/service/adapter の接続ロジックは `grpc::serve` 関数（`src/grpc/mod.rs`）に集約する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-A-\*、DX-GP-\*。**上流**: DS-SW-COMP-050〜052、DS-IMPL-DIR-044。

### DS-IMPL-DIR-106 bin crate の lib.rs

各 bin crate の `src/lib.rs` は test から内部モジュールを見えるようにするための minimal な公開宣言を行う。

```rust
// crates/pods/audit/src/lib.rs
pub mod adapter;
pub mod config;
pub mod domain;
pub mod grpc;
pub mod service;
```

`lib.rs` を置く理由は、`tests/` の統合テストが `audit::service::AuditService` のようにモジュールを参照できるようにするため。`Cargo.toml` に `[lib]` と `[[bin]]` を両方宣言することで、bin としても lib としてもビルド可能にする。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*。**上流**: DS-SW-COMP-131。

### DS-IMPL-DIR-107 Cargo.toml の [lib] [[bin]] 両立

各 bin crate の `Cargo.toml` は以下のように bin と lib を両立し、social な path 依存は `shared/` 配下の相対パスで解決する。

```toml
[package]
name = "k1s0-audit"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
path = "src/lib.rs"

[[bin]]
name = "k1s0-audit"
path = "src/main.rs"

[dependencies]
# workspace 共通
tokio.workspace = true
tonic.workspace = true
# crate 固有
sqlx.workspace = true
rdkafka.workspace = true
# 社内 crate（path 依存、shared/ 配下への相対参照）
k1s0-common = { path = "../../shared/common" }
k1s0-proto = { path = "../../shared/proto-gen" }
k1s0-otel = { path = "../../shared/otel-util" }
k1s0-policy = { path = "../../shared/policy" }

[dev-dependencies]
testcontainers.workspace = true
anyhow.workspace = true

[[bench]]
name = "hash_chain_bench"
harness = false
```

crate 名は `k1s0-audit` 形式（kebab-case）、bin 名も同様に `k1s0-audit`。内部で使う Rust identifier は `k1s0_audit`（underscore）で、これは Cargo が自動変換する。path 依存は全て `../../shared/<crate>` 形式で、`pods/<other>` への依存は書けないようディレクトリ階層で物理的に防ぐ。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-131、DS-SW-COMP-112。

## ビルドと Docker image

### DS-IMPL-DIR-108 ビルドコマンドの集約

Rust ビルドは `cd src/tier1/rust && cargo build --release --workspace --locked` で実行する。`--locked` を付けることで CI 環境で `Cargo.lock` の drift を検出する。各 Pod の個別ビルドは `cargo build --release --bin k1s0-audit` で可能。ビルド時間短縮のため sccache（Phase 1b）を導入し、`.cargo/config.toml` に `[build] rustc-wrapper = "sccache"` を追加する。

**確定フェーズ**: Phase 1a（基本）、Phase 1b（sccache）。**対応要件**: DX-CICD-\*、NFR-C-NOP-002、DX-MET-\*。**上流**: DS-SW-COMP-133、DS-SW-COMP-135。

### DS-IMPL-DIR-109 各 bin crate の Dockerfile

各 bin crate の container image は `crates/pods/<crate>/Dockerfile` で定義する。multi-stage build の第 1 stage は `rust:1.85` で `cargo build --release --bin k1s0-<crate>`、第 2 stage は `gcr.io/distroless/cc-debian12:nonroot`（cc 版、glibc + libgcc 含む）でバイナリをコピーする（DS-SW-COMP-134）。image 名は Phase 1a で `ghcr.io/k1s0/tier1-<crate>:<version>` 形式、Phase 1b 以降 `harbor.k1s0.internal/tier1/<crate>:<version>` placeholder に移行する（Harbor の本番 FQDN は 07 章 DS-IMPL-DIR-191 の sed 手順で置換する）。

**確定フェーズ**: Phase 1a（ghcr）、Phase 1b（Harbor placeholder）。**対応要件**: DX-CICD-\*、NFR-F-ENV-\*。**上流**: DS-SW-COMP-134、DS-SW-COMP-137、DS-IMPL-DIR-213。

### DS-IMPL-DIR-110 release profile の最適化

`Cargo.toml` の root で `[profile.release]` を以下に設定する（DS-SW-COMP-133）。

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
panic = "abort"
opt-level = 3
```

`lto = "thin"` でリンク時最適化を有効化、`codegen-units = 1` で最大最適化（ビルド時間は増えるが runtime 速度向上）、`strip = "symbols"` でバイナリサイズ削減、`panic = "abort"` で panic 時に即終了（unwind を行わない）。distroless イメージでは unwind 情報も削除されるため、`panic = "abort"` と整合する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-B-PERF-\*、NFR-F-ENV-\*。**上流**: DS-SW-COMP-133。

## テストとカバレッジ

### DS-IMPL-DIR-111 cargo nextest の採用

unit + integration test は `cargo nextest run --workspace --locked` で実行する。標準の `cargo test` より 2〜3 倍高速で、ファイル分離 test bin を並列実行する。`nextest.toml` で test profile（`ci` / `default`）を分け、CI では `--profile ci` を指定して retry 回数・timeout を明示する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*、DX-MET-\*。**上流**: DS-SW-COMP-136。

### DS-IMPL-DIR-112 カバレッジ計測（cargo llvm-cov）

coverage 計測は `cargo llvm-cov nextest --workspace --html --output-dir target/coverage` で実施する。`llvm-cov` は LLVM 由来の正確な coverage を計測する。目標は全体 80% / 公開 API 関数 100%（DS-SW-COMP-136）。CI で coverage レポートを Artifacts に保存し、Phase 1c で Backstage プラグインに統合する。

**確定フェーズ**: Phase 1a（計測）、Phase 1c（Backstage 統合）。**対応要件**: DX-TEST-\*、DX-BS-\*。**上流**: DS-SW-COMP-136。

### DS-IMPL-DIR-113 test fixture の配置

Rust の test fixture は各 crate の `tests/fixtures/` に配置する（慣例）。例えば `crates/pods/audit/tests/fixtures/sample_event.cbor` のような CBOR ファイル、または `crates/pods/decision/tests/fixtures/credit_rule.json` のような JSON ルール。大規模 fixture（10MB 以上）はリポジトリ root の `tests/fixtures/` に集約し、Git LFS で管理する（詳細は [06_テストとフィクスチャ配置.md](06_テストとフィクスチャ配置.md)）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-TEST-\*。**上流**: DS-SW-COMP-136。

## 依存管理

### DS-IMPL-DIR-114 path 依存の使用ルールと Pod 間横断禁止

`crates/` 配下の path 依存は以下の 2 ルールで厳格に管理する。

1. `pods/<pod>/` の依存は `shared/<lib>` と `sdk/<crate>` にのみ許可し、`pods/<other>/` への path 依存は禁止する
2. `shared/<lib>` の依存は `shared/<other lib>` にのみ許可し、`pods/` や `sdk/` への path 依存は禁止する（shared は Pod に依存しない）

ルール違反は CI の `tools/check-rust-deps/` スクリプトで検査する（`Cargo.toml` を AST 解析して path 依存を列挙し、上記ルールに反する行があれば exit code 1）。これは go-cleanarch に相当する Rust 側の依存検査である。workspace 内 crate を git や crates-io から取得することも禁止し、workspace の完結性を保つ（`cargo publish` 時の意図せぬ公開を防ぐ）。Phase 2 以降で tools/ と crates/ の間で依存が必要になる場合も path 依存（相対パス）で接続する（DS-IMPL-DIR-008 参照）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-C-NOP-002、DX-CICD-\*。**上流**: DS-SW-COMP-112、DS-SW-COMP-131。

### DS-IMPL-DIR-115 外部 crate 追加の手続

外部 crate（crates.io）を新規追加する場合は ADR 起票を要する（DS-SW-COMP-114 準拠）。ADR には以下を含める。

1. 代替案の検討（既存の依存で代替不可な理由）
2. ライセンス確認（`cargo deny check licenses` が pass）
3. 脆弱性確認（`cargo audit` が pass）
4. メンテナンス性確認（GitHub stars・最終更新日・`unmaintained` 勧告の有無）
5. バイナリサイズ影響（`cargo bloat` で推定）

ADR 承認後に `[workspace.dependencies]` に追加し、各 crate の `Cargo.toml` から `<name>.workspace = true` で参照する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-SUP-\*、NFR-E-ENC-001。**上流**: DS-SW-COMP-114、DS-SW-COMP-117。

### DS-IMPL-DIR-116 Renovate 設定

`.github/renovate.json`（Phase 1b 以降）で Rust 依存の自動更新を設定する。`workspace.dependencies` 経由の依存のみを対象にし、個別 crate の `Cargo.toml` は対象外とする（workspace 集約のため二重 update を避ける）。update 頻度は週次（セキュリティ）+ 月次（機能）。major バージョン更新は自動マージせず、ADR 起票を要する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-SUP-\*。**上流**: DS-SW-COMP-119。

## CODEOWNERS と命名規約

### DS-IMPL-DIR-117 Rust crate の CODEOWNERS

`src/tier1/rust/` 配下の変更は `@k1s0/tier1-rust-team` が responsible。`crates/shared/` 配下の横断 crate は `@k1s0/tier1-architects` も承認者に加える（アーキテクチャ影響が大きく、全 Pod を破壊する変更経路となるため）。`crates/pods/<pod>/` は `@k1s0/tier1-rust-team` のみで承認可能とし、Pod チームが自走できるよう権限を分散する。`crates/pods/audit/migrations/` は schema 変更を伴うため `@k1s0/tier1-architects` と `@k1s0/security-team` の両方の承認を必須とする。`crates/sdk/k1s0-sdk/`（Phase 1b〜）は外部公開 API の安定性契約を持つため `@k1s0/tier1-architects` と `@k1s0/api-leads` の両方の承認を必須とする。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-C-NOP-001、NFR-H-INT-001、DX-CICD-\*。**上流**: DS-IMPL-DIR-018。

### DS-IMPL-DIR-118 ファイル命名規約（Rust 特有）

Rust のファイル命名は snake_case（`audit_service.rs` / `hash_chain.rs`）。Go 側（DS-IMPL-DIR-077）と統一する。crate 名は kebab-case（`k1s0-audit`）、モジュール名（`mod audit_service;`）は snake_case、struct/enum 名は UpperCamelCase（`AuditService` / `HashChain`）。これは Rust の慣例（RFC 430）に従う。`pods` / `shared` / `sdk` という集約ディレクトリは Cargo の path としてのみ使用し、crate 名に含めない（crate 名は `k1s0-audit` であり `k1s0-pods-audit` ではない）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-C-NOP-002。**上流**: DS-IMPL-DIR-024、DS-IMPL-DIR-077。

### DS-IMPL-DIR-119 `pub` 修飾子の最小化

`pub` は最小限にし、crate 内部でのみ使う型は `pub(crate)` で制限する（DS-SW-COMP-112）。モジュール横断で使う型は `pub` にするが、他 crate から見せる必要がない場合は `pub(crate)` を優先する。`pub(super)` や `pub(in path)` は原則使わない（可視性範囲が複雑化するため）。`crates/shared/<lib>` の `pub` API は全 Pod の破壊的変更経路となるため、新規 `pub` 追加も ADR で審査する運用とする。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-C-NOP-002。**上流**: DS-SW-COMP-112。

### DS-IMPL-DIR-120 新規 crate 追加の手続

新規 crate 追加は DS-SW-COMP-117 の依存審査に加え、本ファイルの 4 層構造を採用する（bin crate の場合）または明確な責務分離を宣言する（lib crate の場合）。3 カテゴリ（pods / shared / sdk）のどこに配置するかも ADR で明示する。手順は以下。

1. ADR 起票（`docs/02_構想設計/adr/ADR-TIER1-NNN-*.md` または `ADR-SDK-NNN-*.md`）。カテゴリ判定（pods / shared / sdk）を記述
2. `Cargo.toml` の `[workspace] members` に `crates/<category>/<newcrate>` を追加（glob ではなく明示列挙）
3. `crates/<category>/<newcrate>/Cargo.toml` で `[workspace.dependencies]` から必要な依存を引く
4. `CODEOWNERS` に `/src/tier1/rust/crates/<category>/<newcrate>/` を追加
5. `.github/workflows/pr-rust.yml` の対象に追加（paths フィルタ）
6. README.md で crate の責務と API 概要を記述

新 Pod 追加は `shared/` を変更しない（変更する場合は別 PR で ADR 必須）。この手続を厳守することで、Pod 追加が線形コストで収まり、既存 Pod への副作用を遮断する。

**確定フェーズ**: 各 Phase（新 crate 追加時）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-SW-COMP-117、DS-SW-COMP-138。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-081 | Cargo workspace root のファイル構成（3 カテゴリ） | Phase 1a |
| DS-IMPL-DIR-082 | Cargo.toml の [workspace] 設定（pods/shared/sdk） | Phase 1a |
| DS-IMPL-DIR-083 | rust-toolchain.toml の内容 | Phase 1a |
| DS-IMPL-DIR-084 | .cargo/config.toml の内容 | Phase 1a |
| DS-IMPL-DIR-085 | deny.toml の内容 | Phase 1a |
| DS-IMPL-DIR-086 | clippy.toml の内容 | Phase 1a |
| DS-IMPL-DIR-087 | rustfmt.toml の内容 | Phase 1a |
| DS-IMPL-DIR-088 | shared/common crate の配置 | Phase 1a |
| DS-IMPL-DIR-089 | shared/proto-gen crate の配置 | Phase 1a |
| DS-IMPL-DIR-090 | shared/otel-util crate の配置 | Phase 1a |
| DS-IMPL-DIR-091 | shared/policy crate の配置 | Phase 1a/1b/1c |
| DS-IMPL-DIR-092 | pods bin crate の 4 層と Pod 間禁止 | Phase 1a |
| DS-IMPL-DIR-093 | grpc/ の配置 | Phase 1a |
| DS-IMPL-DIR-094 | service/ の配置 | Phase 1a |
| DS-IMPL-DIR-095 | domain/ の配置 | Phase 1a |
| DS-IMPL-DIR-096 | adapter/ の配置 | Phase 1a/1c |
| DS-IMPL-DIR-097 | pods/audit crate の migrations/ 配置 | Phase 1b |
| DS-IMPL-DIR-098 | pods/audit crate の内部モジュール対応 | Phase 1b/1c |
| DS-IMPL-DIR-099 | pods/decision crate の rules/ 配置 | Phase 1a |
| DS-IMPL-DIR-100 | pods/pii crate の検出ルール | Phase 1a/1b |
| DS-IMPL-DIR-101 | crate 内 tests/ の配置原則 | Phase 1a |
| DS-IMPL-DIR-102 | tests/common/ の共通ヘルパ | Phase 1b |
| DS-IMPL-DIR-103 | benches/ の配置と Criterion | Phase 1b |
| DS-IMPL-DIR-104 | examples/ の配置 | Phase 1b |
| DS-IMPL-DIR-105 | bin crate の main.rs | Phase 1a |
| DS-IMPL-DIR-106 | bin crate の lib.rs | Phase 1a |
| DS-IMPL-DIR-107 | Cargo.toml の [lib] [[bin]] 両立と path 依存 | Phase 1a |
| DS-IMPL-DIR-108 | ビルドコマンドの集約 | Phase 1a/1b |
| DS-IMPL-DIR-109 | 各 bin crate の Dockerfile | Phase 1a |
| DS-IMPL-DIR-110 | release profile の最適化 | Phase 1a |
| DS-IMPL-DIR-111 | cargo nextest の採用 | Phase 1a |
| DS-IMPL-DIR-112 | カバレッジ計測（cargo llvm-cov） | Phase 1a/1c |
| DS-IMPL-DIR-113 | test fixture の配置 | Phase 1a |
| DS-IMPL-DIR-114 | path 依存ルールと Pod 間横断禁止 | Phase 1a |
| DS-IMPL-DIR-115 | 外部 crate 追加の手続 | Phase 1a |
| DS-IMPL-DIR-116 | Renovate 設定 | Phase 1b |
| DS-IMPL-DIR-117 | Rust crate の CODEOWNERS（pods/shared/sdk 分離） | Phase 1a |
| DS-IMPL-DIR-118 | ファイル命名規約 | Phase 1a |
| DS-IMPL-DIR-119 | `pub` 修飾子の最小化 | Phase 1a |
| DS-IMPL-DIR-120 | 新規 crate 追加の手続（カテゴリ明示） | 各 Phase |

### 対応要件一覧

- NFR-A-\*、NFR-A-CONT-003、NFR-B-PERF-\*、NFR-C-NOP-001、NFR-C-NOP-002、NFR-E-AC-\*、NFR-E-ENC-001、NFR-E-MON-\*、NFR-F-ENV-\*、NFR-G-PROT-\*、NFR-H-INT-001、NFR-H-KEY-\*、NFR-SUP-\*
- FR-T1-AUDIT-\*、FR-T1-DECISION-\*、FR-T1-PII-\*、FR-T1-TELEMETRY-\*
- DX-CICD-\*、DX-GP-\*、DX-TEST-\*、DX-LD-\*、DX-MET-\*、DX-BS-\*

### 上流設計 ID

DS-SW-COMP-050〜079（自作 Rust 領域 3 Pod）、DS-SW-COMP-108〜110（共通ライブラリ）、DS-SW-COMP-112（Rust crate 公開範囲）、DS-SW-COMP-114（Rust 依存検査）、DS-SW-COMP-117（crate 追加審査）、DS-SW-COMP-119（OSS 更新）、DS-SW-COMP-129〜134（Cargo workspace・ビルド・Docker image）、DS-SW-COMP-136（テスト方針）、DS-SW-COMP-138（変更手続）。DS-IMPL-DIR-018（CODEOWNERS）、DS-IMPL-DIR-031（proto-gen crate）、DS-IMPL-DIR-077（Go ファイル命名）、DS-IMPL-DIR-213（Harbor placeholder）、DS-IMPL-DIR-225（SDK 境界）と双方向トレースする。
