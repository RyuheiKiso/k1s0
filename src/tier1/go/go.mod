// k1s0 tier1 Go ファサード（plan 04-01 / IMP-DIR-002 / IMP-BUILD-002）
//
// 役割:
//   stable Dapr Go SDK を直接叩く Go ファサード層。tier2/tier3 は本層が露出する gRPC のみを使う
//   （ADR-TIER1-003: tier1 内部言語は不可視）。
//
// scope（リリース時点）:
//   plan 04-01 完了条件の最小骨格として `cmd/k1s0d` の gRPC server 起動 + 標準 gRPC health
//   protocol（grpc.health.v1.Health/Check）応答までを提供。Dapr / Temporal / Keycloak / OpenBao
//   の各 client wrapper、12 API ハンドラの実装は plan 04-02 〜 04-13 で順次追加する。
//
// module path: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md 正典に準拠（monorepo path-style）
module github.com/k1s0/k1s0/src/tier1/go

go 1.25.0

require (
	github.com/dapr/go-sdk v1.11.0
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	github.com/openbao/openbao/api/v2 v2.5.1
	go.opentelemetry.io/otel v1.43.0
	go.opentelemetry.io/otel/log v0.19.0
	go.opentelemetry.io/otel/metric v1.43.0
	go.opentelemetry.io/otel/sdk v1.39.0
	go.opentelemetry.io/otel/sdk/metric v1.39.0
	go.opentelemetry.io/otel/trace v1.43.0
	go.temporal.io/sdk v1.42.0
	google.golang.org/grpc v1.79.3
	google.golang.org/protobuf v1.36.11
)

require (
	github.com/cenkalti/backoff/v4 v4.3.0 // indirect
	github.com/cespare/xxhash/v2 v2.3.0 // indirect
	github.com/dapr/dapr v1.14.0 // indirect
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/facebookgo/clock v0.0.0-20150410010913-600d898af40a // indirect
	github.com/go-jose/go-jose/v4 v4.1.3 // indirect
	github.com/go-logr/logr v1.4.3 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/go-viper/mapstructure/v2 v2.4.0 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/mock v1.6.0 // indirect
	github.com/google/uuid v1.6.0 // indirect
	github.com/grpc-ecosystem/go-grpc-middleware/v2 v2.3.2 // indirect
	github.com/grpc-ecosystem/grpc-gateway/v2 v2.22.0 // indirect
	github.com/hashicorp/errwrap v1.1.0 // indirect
	github.com/hashicorp/go-cleanhttp v0.5.2 // indirect
	github.com/hashicorp/go-multierror v1.1.1 // indirect
	github.com/hashicorp/go-retryablehttp v0.7.8 // indirect
	github.com/hashicorp/go-secure-stdlib/parseutil v0.2.0 // indirect
	github.com/hashicorp/go-secure-stdlib/strutil v0.1.2 // indirect
	github.com/hashicorp/go-sockaddr v1.0.7 // indirect
	github.com/hashicorp/hcl v1.0.1-vault-7 // indirect
	github.com/mitchellh/mapstructure v1.5.1-0.20220423185008-bf980b35cac4 // indirect
	github.com/nexus-rpc/sdk-go v0.6.0 // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	github.com/robfig/cron v1.2.0 // indirect
	github.com/ryanuber/go-glob v1.0.0 // indirect
	github.com/stretchr/objx v0.5.2 // indirect
	github.com/stretchr/testify v1.11.1 // indirect
	go.opentelemetry.io/auto/sdk v1.2.1 // indirect
	go.temporal.io/api v1.62.7 // indirect
	golang.org/x/net v0.49.0 // indirect
	golang.org/x/sync v0.19.0 // indirect
	golang.org/x/sys v0.40.0 // indirect
	golang.org/x/text v0.33.0 // indirect
	golang.org/x/time v0.14.0 // indirect
	google.golang.org/genproto/googleapis/api v0.0.0-20260120221211-b8f7ae30c516 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260120221211-b8f7ae30c516 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

// docs 正典: docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
// 同一リポジトリ内で SDK Go を参照するため replace directive を使う。
// SDK が外部 registry に publish されたら本 directive は削除する（リリース時点 削除）。
replace github.com/k1s0/sdk-go => ../../sdk/go
