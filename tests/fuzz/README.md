# tests/fuzz — Protobuf / REST 入力 Fuzz

Protobuf decode / crypto primitive / tier1 facade HTTP handler を Fuzz する。

## 構造

```text
fuzz/
├── README.md
├── rust/                  # cargo-fuzz（src/tier1/rust の Rust crate を対象）
│   ├── Cargo.toml
│   └── fuzz_targets/
│       ├── proto_fuzz.rs
│       └── crypto_fuzz.rs
└── go/                    # go-fuzz / std fuzzing（src/tier1/go の facade を対象）
    ├── go.mod
    └── targets/
        └── facade_fuzz.go
```

## 実行

```bash
# Rust（cargo-fuzz）
cd tests/fuzz/rust && cargo fuzz run proto_fuzz -- -max_total_time=300

# Go（std fuzzing、Go 1.18+）
cd tests/fuzz/go && go test ./targets/... -fuzz=FuzzFacadeRequest -fuzztime=5m
```

発見された crasher は `<lang>/fuzz_targets/corpus/`（rust）/ `targets/testdata/fuzz/`（go）に追加して regression suite 化する。

## CI

CPU 時間が大きいため週次（cron）で実行。PR でも `run-fuzz` ラベルがあれば短時間（5 分）で起動する。
