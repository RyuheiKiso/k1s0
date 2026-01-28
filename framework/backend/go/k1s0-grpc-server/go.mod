module github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-grpc-server

go 1.22.7

toolchain go1.24.1

require (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error v0.0.0
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability v0.0.0
	go.uber.org/zap v1.27.0
	google.golang.org/grpc v1.68.0
)

require (
	go.opentelemetry.io/otel v1.32.0 // indirect
	go.opentelemetry.io/otel/trace v1.32.0 // indirect
	go.uber.org/multierr v1.10.0 // indirect
	golang.org/x/net v0.29.0 // indirect
	golang.org/x/sys v0.28.0 // indirect
	golang.org/x/text v0.21.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240903143218-8af14fe29dc1 // indirect
	google.golang.org/protobuf v1.35.1 // indirect
)

replace (
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error => ../k1s0-error
	github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability => ../k1s0-observability
)
