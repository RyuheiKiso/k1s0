# tests/fuzz — Protobuf / REST 入力 Fuzz

Protobuf decode / crypto primitive / tier1 facade HTTP handler を Fuzz する。

## 構造

```text
fuzz/
├── README.md
├── rust/                       # cargo-fuzz（src/tier1/rust の Rust crate を対象）
│   ├── Cargo.toml
│   ├── fuzz_targets/
│   │   ├── proto_fuzz.rs
│   │   └── crypto_fuzz.rs
│   └── standalone/             # libfuzzer 不在環境向け代替 harness
│       ├── Cargo.toml
│       └── src/
│           ├── proto_fuzz.rs   # ChaCha8 PRNG + edge cases、panic なきこと検証
│           └── crypto_fuzz.rs
└── go/                         # std fuzzing（Go 1.18+、src/tier1/go facade を対象）
    ├── go.mod
    └── targets/
        └── facade_fuzz_test.go # 4 protojson decoder の panic 検出
```

## 実行

```bash
# Rust（cargo-fuzz、libfuzzer-sys + LLVM ASAN linker が必要）
cd tests/fuzz/rust && cargo fuzz run proto_fuzz --fuzz-dir . -- -max_total_time=300
cd tests/fuzz/rust && cargo fuzz run crypto_fuzz --fuzz-dir . -- -max_total_time=300

# Rust 代替（cargo-fuzz が build できない環境向け、PRNG ベース）
cd tests/fuzz/rust/standalone && cargo run --release --bin proto_fuzz_standalone -- 500000 42
cd tests/fuzz/rust/standalone && cargo run --release --bin crypto_fuzz_standalone -- 500000 42

# Go（std fuzzing、Go 1.18+）
cd tests/fuzz/go && go test -fuzz=FuzzStateSetJSON -fuzztime=5m ./targets/
cd tests/fuzz/go && go test -fuzz=FuzzAuditRecordJSON -fuzztime=5m ./targets/
cd tests/fuzz/go && go test -fuzz=FuzzPubSubPublishJSON -fuzztime=5m ./targets/
cd tests/fuzz/go && go test -fuzz=FuzzWorkflowStartJSON -fuzztime=5m ./targets/
```

発見された crasher は `<lang>/fuzz_targets/corpus/`（rust）/ `targets/testdata/fuzz/`（go）に追加して regression suite 化する。

## 直近実走実績（2026-04-30）

- Go std fuzzing: 4 target（FuzzStateSetJSON / FuzzAuditRecordJSON / FuzzPubSubPublishJSON / FuzzWorkflowStartJSON）× 30s × 20 worker で **計 ~2.7M execs、0 panic**
- Rust standalone: proto_fuzz / crypto_fuzz 各 500,000 iters + edge cases で **0 panic / 0 abort**
- Rust cargo-fuzz: 当該環境の C++ toolchain（zigcxx）が ASAN linker と非互換のため未実走（standalone で代替検証済）

## CI

CPU 時間が大きいため週次（cron）で実行。PR でも `run-fuzz` ラベルがあれば短時間（5 分）で起動する。
