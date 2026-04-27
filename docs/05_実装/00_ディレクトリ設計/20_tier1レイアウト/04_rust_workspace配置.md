# 04. Rust workspace 配置

本ファイルは `src/tier1/rust/` 配下の Cargo workspace レイアウトを確定する。DS-SW-COMP-129 / 130 / 131 / 132 の規定を物理配置レベルで展開し、ADR-DIR-001（contracts 昇格）による proto 入力 path の影響を明示する。

## レイアウト

```text
src/tier1/rust/
├── README.md
├── Cargo.toml              # workspace 設定
├── Cargo.lock
├── rust-toolchain.toml     # stable 1.85+ pin
├── .cargo/
│   └── config.toml         # rustflags / registry / target-dir
├── Dockerfile.decision     # t1-decision Pod 用
├── Dockerfile.audit        # t1-audit Pod 用
├── Dockerfile.pii          # t1-pii Pod 用
└── crates/
    ├── audit/              # COMP-T1-AUDIT 本体（bin crate）
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── main.rs
    │   │   └── ...
    │   └── tests/
    ├── decision/           # COMP-T1-DECISION 本体（bin crate）
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── main.rs
    │   │   └── ...
    │   └── tests/
    ├── pii/                # COMP-T1-PII 本体（bin crate）
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── main.rs
    │   │   └── ...
    │   └── tests/
    ├── common/             # k1s0-common（lib crate）
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── proto-gen/          # k1s0-proto（buf 生成）
    │   ├── Cargo.toml
    │   ├── build.rs
    │   └── src/
    │       ├── lib.rs
    │       └── v1/
    │           ├── mod.rs
    │           └── ...（include! された生成コード）
    ├── otel-util/          # k1s0-otel（lib crate）
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── policy/             # k1s0-policy（lib crate）
        ├── Cargo.toml
        └── src/lib.rs
```

## workspace Cargo.toml の推奨サンプル

DS-SW-COMP-130 の方針に従う。

```toml
[workspace]
resolver = "2"
members = [
  "crates/audit",
  "crates/decision",
  "crates/pii",
  "crates/common",
  "crates/proto-gen",
  "crates/otel-util",
  "crates/policy",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"
authors = ["k1s0 contributors"]
rust-version = "1.85"

[workspace.dependencies]
# 非同期ランタイム
tokio = { version = "1", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"

# シリアライゼーション
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# トレーシング
tracing = "0.1"
tracing-subscriber = "0.3"
opentelemetry = "0.27"
opentelemetry-otlp = "0.27"

# メトリクス
prometheus = "0.13"

# AUDIT 固有
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }
rdkafka = { version = "0.36", features = ["ssl", "zstd"] }

# DECISION 固有
zen-engine = "0.37"

# PII 固有
regex = "1.10"
sha2 = "0.10"
aes-siv = "0.7"

# エラー
thiserror = "1.0"
anyhow = "1.0"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"
opt-level = 3

[profile.dev]
opt-level = 1
```

## proto-gen crate の配置

DS-SW-COMP-132 に従う。

```toml
# crates/proto-gen/Cargo.toml
[package]
name = "k1s0-proto"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
tonic.workspace = true
prost.workspace = true

[build-dependencies]
tonic-build = "0.12"
```

`build.rs` は以下の形式で `src/contracts/tier1/v1/*.proto` + `src/contracts/internal/v1/*.proto` を読み込む。

```rust
// build.rs
// Protobuf から Rust コードを生成する
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // src/contracts/ への相対パス
    let contracts_root = "../../../contracts";

    // 公開 tier1 API の proto ファイル
    let tier1_protos = [
        "tier1/v1/state.proto",
        "tier1/v1/pubsub.proto",
        "tier1/v1/serviceinvoke.proto",
        "tier1/v1/secrets.proto",
        "tier1/v1/binding.proto",
        "tier1/v1/workflow.proto",
        "tier1/v1/log.proto",
        "tier1/v1/telemetry.proto",
        "tier1/v1/decision.proto",
        "tier1/v1/audit.proto",
        "tier1/v1/feature.proto",
    ];

    // tier1 内部 API の proto ファイル
    let internal_protos = [
        "internal/v1/common.proto",
        "internal/v1/errors.proto",
        "internal/v1/pii.proto",
    ];

    // 全 proto ファイルのフルパスを集約
    let all_protos: Vec<String> = tier1_protos.iter()
        .chain(internal_protos.iter())
        .map(|p| format!("{}/{}", contracts_root, p))
        .collect();

    // tonic-build で Rust コードを生成
    // out_dir は crate root (Cargo.toml 位置) からの相対。tonic-build は
    // proto package 名を `.` → `_` に変換したフラットファイルを出力する。
    // k1s0.tier1.v1 → src/k1s0.tier1.v1.rs
    // k1s0.tier1.internal.v1 → src/k1s0.tier1.internal.v1.rs
    // （階層ディレクトリは掘らない。lib.rs で include! マクロで再エクスポートする）
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src")
        .compile(&all_protos, &[contracts_root])?;

    Ok(())
}
```

なお `build.rs` 方式と pre-generated 方式を同時に有効化すると生成先が衝突するため、どちらか一方のみ有効とする。リリース時点 は pre-generated を正、`build.rs` は drift 検出用に `--dry-run` 相当（生成後 diff 確認のみ）で運用する。

## 生成物の commit

DS-SW-COMP-132 で論じた通り、リリース時点 では `build.rs` 方式（OUT_DIR）と pre-generated 方式の両方を準備する。pre-generated は `crates/proto-gen/src/` 直下にフラット commit し（`k1s0.tier1.v1.rs` / `k1s0.tier1.internal.v1.rs`）、`build.rs` で生成した結果との diff を CI で検出する。

リリース時点 で運用が安定したら、pre-generated 一本化（`build.rs` 廃止）を検討する。

## 各 Pod crate の内部構造

### crates/decision/（t1-decision Pod）

```text
crates/decision/
├── Cargo.toml
├── src/
│   ├── main.rs             # Pod entrypoint
│   ├── config.rs           # 設定ロード
│   ├── grpc.rs             # tonic サーバ実装
│   ├── zen_runtime.rs      # ZEN Engine 組み込み
│   ├── rule_loader.rs      # ルール DSL ロード
│   └── cache.rs            # 決定結果キャッシュ
└── tests/
    ├── integration/
    └── fixtures/
```

### crates/audit/（t1-audit Pod）

```text
crates/audit/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── grpc.rs
│   ├── ingress.rs          # Kafka consumer（audit イベント取込）
│   ├── storage.rs          # PostgreSQL append-only 書込
│   ├── integrity.rs        # 完整性チェック（hash chain）
│   └── export.rs           # 監査証跡のエクスポート
└── tests/
```

### crates/pii/（t1-pii Pod）

```text
crates/pii/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── grpc.rs
│   ├── detector.rs         # regex + ML ハイブリッド PII 検出
│   ├── masker.rs           # PII マスキング
│   ├── cryptor.rs          # 暗号化（aes-siv）
│   └── audit_client.rs     # audit Pod への gRPC クライアント
└── tests/
```

## Container image

DS-SW-COMP-134 に従い multi-stage build。

```dockerfile
# Dockerfile.decision
# build context はリポジトリルート（docker build -f src/tier1/rust/Dockerfile.decision .）
FROM rust:1.85 AS builder
WORKDIR /workspace
COPY src/tier1/rust/Cargo.toml src/tier1/rust/Cargo.lock src/tier1/rust/rust-toolchain.toml ./
COPY src/tier1/rust/crates/ ./crates/
# contracts も build.rs が参照するためコピー（build context がリポジトリルートのため src/ から参照可能）
COPY src/contracts/ /workspace-contracts/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/target \
    cargo build --release --bin decision && \
    cp target/release/decision /workspace/t1-decision

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /workspace/t1-decision /usr/local/bin/t1-decision
USER nonroot
EXPOSE 50001 9090
ENTRYPOINT ["/usr/local/bin/t1-decision"]
```

build context は **常にリポジトリルート** とする。Docker の `COPY` は build context 外部（`../` 参照）を禁止しているため、contracts を参照するには build context をリポジトリルートに設定し `src/contracts/` パスで参照する。CI は以下のように呼び出す。

```bash
docker build -f src/tier1/rust/Dockerfile.decision -t ghcr.io/k1s0/t1-decision:$TAG .
```

## テスト戦略

- **unit test**: `#[cfg(test)]` モジュール内（`cargo nextest run` で並列実行）
- **integration test**: `crates/<crate>/tests/integration/` で testcontainers を使って外部リソースを起動
- **coverage**: `cargo llvm-cov`、目標 80%（リリース時点）
- **fuzz**: `cargo-fuzz` を pii crate で使い、正規表現 DoS 対策を検証

## スパースチェックアウト cone

- `tier1-rust-dev` cone に `src/tier1/rust/` を含む
- `tier1-go-dev` cone には含まない

## 対応 IMP-DIR ID

- IMP-DIR-T1-024（src/tier1/rust/ 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-001（Go+Rust ハイブリッド）/ ADR-RULE-001（ZEN Engine）/ ADR-DIR-001
- DS-SW-COMP-129 / 130 / 131 / 132 / 133 / 134
- FR-DECISION / FR-AUDIT / FR-PII 系 / NFR-E-ENC-001 / NFR-H-AUD-\*
