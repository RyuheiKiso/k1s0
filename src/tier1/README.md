# src/tier1 — Go ファサード + Rust コアの Hybrid 実装（公開 12 API）

ADR-TIER1-001（Go + Rust Hybrid）/ ADR-TIER1-002（Protobuf gRPC）/ ADR-TIER1-003（内部言語の不可視化）に従い、
tier2 / tier3 から内部言語が見えない形で公開 12 API を提供する。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/`](../../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/)。

## 6 Pod 構成（DS-SW-COMP-005〜010）

| Pod | 言語 | 主な責務 | 公開 API |
|---|---|---|---|
| `t1-state` | Go | Dapr State / PubSub / Binding / ServiceInvoke / Feature の入口 + Log / Telemetry の adapter 内蔵 | State / PubSub / ServiceInvoke / Binding / Feature / Log / Telemetry |
| `t1-secret` | Go | OpenBao 経由の Secrets API + envelope encryption | Secrets |
| `t1-workflow` | Go | Dapr Workflow（短期）+ Temporal（長期）の振り分け | Workflow |
| `t1-decision` | Rust | ZEN Engine 統合、JDM 評価 | Decision |
| `t1-audit` | Rust | WORM ストア StatefulSet、ハッシュチェーン改ざん検知 | Audit |
| `t1-pii` | Rust | PII 分類・マスキング（純関数ステートレス） | Pii |

## 配置

```text
tier1/
├── go/                                    # Dapr Go ファサード（3 Pod）
│   ├── go.mod
│   ├── cmd/
│   │   ├── state/main.go                  # t1-state 起動
│   │   ├── secret/main.go                 # t1-secret 起動
│   │   └── workflow/main.go               # t1-workflow 起動
│   ├── internal/
│   │   ├── common/                        # gRPC bootstrap / config / retry / timeout
│   │   ├── otel/                          # OpenTelemetry 初期化
│   │   ├── adapter/dapr/                  # Dapr SDK adapter（5 building block 別）
│   │   ├── state/                         # State + PubSub + Binding + Invoke + Feature handler
│   │   ├── secret/                        # Secrets handler
│   │   └── workflow/                      # Workflow handler
│   └── proto/v1/                          # buf 生成 internal proto stub
└── rust/                                  # Rust core（3 Pod）
    ├── Cargo.toml                         # workspace（edition 2024）
    ├── crates/
    │   ├── decision/                      # k1s0-tier1-decision（main.rs + ZEN Engine 統合予定）
    │   ├── audit/                         # k1s0-tier1-audit（main.rs + WORM 統合予定）
    │   ├── pii/                           # k1s0-tier1-pii（main.rs + PII 検出予定）
    │   ├── proto-gen/                     # buf 生成 internal proto を Rust module 階層に束ねる
    │   ├── common/                        # 共通 runtime（plan 04-08 で実装）
    │   ├── otel-util/                     # OTel 初期化（plan 04-08）
    │   ├── policy/                        # ポリシー評価（plan 04-08）
    │   └── proto/                         # 公開 proto stub（plan 04-08）
    └── Dockerfile.{decision,audit,pii}
```

## ローカル起動

```sh
# 4. contracts を再生成して tier1 facade のいずれかを起動
buf generate
cd src/tier1/go && go run ./cmd/state           # State + PubSub + 3 API を全部
# or: go run ./cmd/secret                       # Secrets API
# or: go run ./cmd/workflow                     # Workflow API

# Rust 側は別プロセスで起動
cd src/tier1/rust && cargo run -p k1s0-tier1-decision   # Decision API
# or: cargo run -p k1s0-tier1-audit
# or: cargo run -p k1s0-tier1-pii
```

## 関連設計

- [ADR-TIER1-001](../../docs/02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md) — Go + Rust Hybrid
- [ADR-TIER1-002](../../docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md) — Protobuf gRPC
- [ADR-TIER1-003](../../docs/02_構想設計/adr/ADR-TIER1-003-language-invisibility.md) — 内部言語不可視化
- [docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md](../../docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md)
