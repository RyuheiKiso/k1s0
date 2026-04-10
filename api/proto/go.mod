module github.com/k1s0-platform/api

go 1.25.0

require (
	// HIGH-007 監査対応: proto 生成ファイルが buf.build/gen/go/bufbuild/protovalidate を
	// import しているため、モジュール依存として明示的に宣言する。
	buf.build/gen/go/bufbuild/protovalidate/protocolbuffers/go v1.36.5-20250219211840-80ab13bee0bf.1
	google.golang.org/grpc v1.79.3
	google.golang.org/protobuf v1.36.11
)

require (
	go.opentelemetry.io/otel v1.41.0 // indirect
	go.opentelemetry.io/otel/sdk/metric v1.41.0 // indirect
	golang.org/x/net v0.52.0 // indirect
	golang.org/x/sys v0.42.0 // indirect
	golang.org/x/text v0.35.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260209200024-4cfbd4190f57 // indirect
)
