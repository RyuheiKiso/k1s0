# tests/integration — 統合テスト（testcontainers）

Docker / testcontainers で Dapr sidecar / CloudNativePG / Kafka を起動し、tier1 facade / tier2 service を実コンポーネントと連携検証する。

## 構造

```text
integration/
├── README.md              # 本ファイル
├── go/                    # Go 統合テスト（独立 module）
│   ├── go.mod
│   └── tier1-facade/
│       └── service_invoke_integration_test.go
├── dotnet/                # .NET 統合テスト
│   ├── K1s0.Integration.sln
│   └── tests/
│       └── tier2-payroll.tests/
└── compose/               # docker-compose / testcontainers の共通定義
    ├── dapr-compose.yaml
    └── data-layer-compose.yaml
```

## 実行

```bash
# Go 側
cd tests/integration/go && go test ./...

# .NET 側
dotnet test tests/integration/dotnet/K1s0.Integration.sln

# 共通 compose を直接立ち上げて手動検証
docker compose -f tests/integration/compose/dapr-compose.yaml up
```

## CI

`.github/workflows/_reusable-test.yml` の `integration` job が PR 毎に実行。GitHub Actions の `docker` service と testcontainers を組合せ、各言語のテストを matrix 並列で走らせる。

## リリース時点 のスコープ

`go/tier1-facade/service_invoke_integration_test.go` を 1 件、雛形として配置する。Dapr sidecar 起動は `compose/dapr-compose.yaml` で記述し、testcontainers の `compose.NewDockerCompose` で読み込む。本格的な統合テスト追加は採用初期 で（FR-T1-* の各 API について少なくとも 1 件）実装する。
