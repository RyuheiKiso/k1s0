# `examples/tier1-go-facade/` — Go Dapr ファサードの最小例

tier1 Go ファサード（stable Dapr Go SDK を直接使う層）の典型的な実装パタンを示す例。

## 目的

- `src/tier1/go/` 配下の Pod 実装と同じ構造（cmd / internal/common / internal/adapter）を
  新規メンバーが真似できる
- Dapr Go SDK を `internal/adapter/dapr/` に閉じ込める典型を示す（ADR-TIER1-001 隔離方針）
- gRPC server の最小骨格 + health protocol + reflection + graceful shutdown のテンプレート

## 想定読者

- tier1 Go ファサードの新規コミッタ
- 既存業務サービスから tier1 へ API を追加する開発者
- Dapr Go SDK の k1s0 内利用パタンを学びたい人

## scope（リリース時点）

リリース時点では以下 3 点のみを満たす最小骨格を配置する:

1. `go.mod`（k1s0 monorepo path-style: `github.com/k1s0/k1s0/examples/tier1-go-facade`）
2. `cmd/example-facade/main.go`（gRPC bootstrap + standard health protocol）
3. `internal/` ディレクトリ構造（adapter / handler 分離）

**未実装（採用初期に拡張予定）:**

- proto 定義 → handler 登録の完動例
- Dapr Go SDK 経由の State / PubSub / Secrets 呼び出し例
- OTel interceptor 接続例
- integration test（Testcontainers + Dapr Local）
- Dockerfile（`src/tier1/go/Dockerfile.state` 構造を踏襲）
- catalog-info.yaml

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md`
- ADR-TIER1-001（Go + Rust ハイブリッド tier1）
- ADR-TIER1-002（Protobuf gRPC）
- ADR-TIER1-003（言語不可視）

## 起動方法（採用初期完成後の想定）

```bash
cd examples/tier1-go-facade
go run ./cmd/example-facade
# 別ターミナルで疎通確認
grpcurl -plaintext localhost:50001 grpc.health.v1.Health/Check
```

## 参照する tier1 API

本 example 自体は tier1 facade「実装側」の例なので、tier1 API を「使う」のではなく
「提供する」側として参照される。tier2 / tier3 から tier1 を呼ぶ例は
`examples/tier2-go-service/` / `examples/tier3-bff-graphql/` を参照。
