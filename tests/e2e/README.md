# tests/e2e — End-to-End 横断シナリオ

tier1 → tier2 → tier3 を通じた end-to-end シナリオを Go で記述する（最大公約数）。kind cluster 上で `infra/environments/dev/` を apply し、`tools/local-stack/up.sh` で起動した Dapr / CNPG / Kafka 等を相手にする。

## 構造

```text
e2e/
├── README.md              # 本ファイル
├── go.mod                 # 独立 Go module（github.com/k1s0/k1s0/tests/e2e）
├── scenarios/             # シナリオごとの *_test.go
│   └── tenant_onboarding_test.go  # 雛形シナリオ（テナント新規オンボーディング）
├── helpers/               # cluster setup / auth / API client の共通ヘルパ
│   └── cluster.go
└── testdata/              # シナリオ固有の test data（fixtures/ 共有でないもの）
```

## 利用方法

```bash
# kind cluster + infra/environments/dev/ を起動
tools/local-stack/up.sh --role e2e

# E2E テスト実行
cd tests/e2e
go test ./scenarios/... -v -timeout=30m
```

## CI

`.github/workflows/_reusable-test.yml` の `e2e` job が PR ラベル `run-e2e` または週次 cron で発火。kind cluster は GitHub Actions runner 上に立ち上げる（runner 1 台で 4 core / 14 GB を消費する想定）。

## 雛形シナリオ

`scenarios/tenant_onboarding_test.go` は「テナント作成 → ユーザ登録 → 初回ログイン」の最小フローをスケルトンで記述する。リリース時点 では実装は `t.Skip("PHASE: release-initial — ...")` で stub 化し、採用初期 で実装を完成させる。
