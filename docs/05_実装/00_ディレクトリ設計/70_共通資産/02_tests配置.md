# 02. tests 配置

本ファイルは `tests/` 配下の配置を確定する。tier 横断の E2E / Contract / Integration / Fuzz / Golden テストを集約する。

## tests/ の役割

各 tier 内部の unit test は `src/tier1/go/*/*_test.go` 等、言語標準の場所に配置する。`tests/` は以下の「横断テスト」を担う。

- **E2E**: tier1 → tier2 → tier3 を通じた end-to-end シナリオ
- **Contract**: tier 間 API 契約の整合テスト（Pact）
- **Integration**: testcontainers で Dapr / CNPG / Kafka を起動した統合テスト
- **Fuzz**: Protobuf / REST 入力の Fuzz（cargo-fuzz / go-fuzz）
- **Golden**: 出力固定テスト（scaffold 結果等の snapshot）
- **Fixtures**: 全テストで共有する test data / seed

## レイアウト

```text
tests/
├── README.md
├── e2e/
│   ├── README.md
│   ├── go.mod                      # 独立 Go module
│   ├── scenarios/
│   │   ├── tenant_onboarding_test.go
│   │   ├── payroll_full_flow_test.go
│   │   └── audit_pii_flow_test.go
│   ├── helpers/
│   │   ├── cluster_setup.go
│   │   ├── auth_setup.go
│   │   └── api_client.go
│   └── testdata/
├── contract/
│   ├── README.md
│   ├── pact/
│   │   ├── consumers/              # tier3 consumer テスト
│   │   │   ├── portal-bff/
│   │   │   └── admin-bff/
│   │   ├── providers/              # tier1 / tier2 provider 検証
│   │   │   ├── tier1-state/
│   │   │   └── tier2-payroll/
│   │   └── broker-config.yaml      # Pact Broker 設定
│   └── openapi-contract/
│       └── tier1-openapi-spec.yaml
├── integration/
│   ├── README.md
│   ├── dotnet/                     # .NET 統合テスト
│   │   ├── K1s0.Integration.sln
│   │   └── tests/
│   │       └── tier2-payroll.tests/
│   ├── go/                         # Go 統合テスト
│   │   ├── go.mod
│   │   └── tier1-facade/
│   │       └── service_invoke_integration_test.go
│   └── compose/                    # docker-compose / testcontainers
│       ├── dapr-compose.yaml
│       └── data-layer-compose.yaml
├── fuzz/
│   ├── README.md
│   ├── rust/
│   │   ├── Cargo.toml              # cargo-fuzz
│   │   └── fuzz_targets/
│   │       ├── proto_fuzz.rs
│   │       └── crypto_fuzz.rs
│   └── go/
│       ├── go.mod                  # go-fuzz
│       └── targets/
│           └── facade_fuzz.go
├── golden/
│   ├── README.md
│   ├── scaffold-outputs/
│   │   ├── tier2-go-service/
│   │   │   └── expected.tar.gz
│   │   └── tier3-web-app/
│   └── diff-tool/
│       └── compare-outputs.sh
└── fixtures/
    ├── README.md
    ├── seed-data/
    │   ├── users.json
    │   └── tenants.json
    ├── tls-certs/
    │   ├── test-ca.pem
    │   └── test-server.pem
    └── openapi-samples/
```

## e2e/ の構造

e2e テスト基盤は刷新中で本リリース時点では未配置。配置方針・ディレクトリ構造・実行経路は刷新後の新 ADR と本ファイル改訂で再確定する。

## contract/ の構造

Pact Broker を `tools/local-stack/` 上に起動し、以下のフローで契約を検証する。

1. **Consumer**（tier3 BFF）: 期待する tier1 / tier2 API レスポンスを Pact ファイルに記録
2. **Provider**（tier1 / tier2）: Pact Broker から Pact ファイルを取得、実装で再生し契約違反を検出
3. CI で Pact Broker を共有し、consumer と provider の整合を守る

OpenAPI 契約は `openapi-contract/` に spec ファイルを置き、tier3 BFF が spec 通りに応答するか `schemathesis` / `dredd` で検証する。

## integration/ の構造

testcontainers で Dapr sidecar / CloudNativePG / Kafka を起動。各言語でテスト実装。

```go
// tests/integration/go/tier1-facade/service_invoke_integration_test.go
package integration

import (
    "testing"
    "context"
    "github.com/testcontainers/testcontainers-go"
    "github.com/testcontainers/testcontainers-go/modules/compose"
)

// Dapr + tier1-facade を docker-compose で立ち上げて state API を叩く
func TestServiceInvokeIntegration(t *testing.T) {
    ctx := context.Background()
    c, err := compose.NewDockerCompose("../compose/dapr-compose.yaml")
    if err != nil {
        t.Fatal(err)
    }
    defer c.Down(ctx)

    if err := c.Up(ctx); err != nil {
        t.Fatal(err)
    }
    // ... actual test
}
```

## fuzz/ の構造

### rust/

cargo-fuzz で Protobuf decode / crypto primitive の fuzz。

### go/

go-fuzz で tier1 facade の HTTP handler fuzz。

発見された crasher は `fuzz/<lang>/fuzz_targets/corpus/` に追加し regression suite 化。

## golden/ の構造

雛形 CLI の出力を snapshot して、変更時に差分を検出。

```bash
# tests/golden/diff-tool/compare-outputs.sh
#!/usr/bin/env bash
set -euo pipefail

SCAFFOLD_NAME="$1"
k1s0 scaffold "$SCAFFOLD_NAME" -o /tmp/scaffold-out

tar -xzf "tests/golden/scaffold-outputs/${SCAFFOLD_NAME}/expected.tar.gz" -C /tmp/expected/
diff -r /tmp/scaffold-out /tmp/expected/
```

## fixtures/ の構造

全テストで共通の seed data / TLS 証明書 / OpenAPI サンプル。バイナリファイル（tar.gz、pem）は Git LFS で管理（ADR-DIR-004 で リリース時点 に判断）。

## 対応 IMP-DIR ID

- IMP-DIR-COMM-112（tests 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TEST-001（Test Pyramid + testcontainers でテスト戦略を正典化、ADR-DEVEX-003 を吸収）
- DX-GP-\* / NFR-A-AVL-\*
