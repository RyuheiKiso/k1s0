# `examples/tier2-go-service/` — tier2 Go サービス完動例

tier2 Go ドメイン共通サービスの典型的な実装パタンを示す完動例。

## 目的

- `src/tier2/go/services/{notification-hub, stock-reconciler}` と同じ構造（cmd / internal /
  domain / application）を新規メンバーが真似できる
- `github.com/k1s0/sdk` Go モジュール経由で tier1 gRPC を呼ぶ典型例
- 構造化ログ・OTel trace 伝搬・graceful shutdown のテンプレート

## scope

| 段階 | 提供範囲 |
|---|---|
| リリース時点 | 本 README のみ（構造規定） |
| 採用初期 | `cmd/example-service/main.go` + `internal/{domain,application,infrastructure}` + Pact 契約テスト |
| 採用後の運用拡大時 | OutBox / Saga / Idempotency Key の典型例 |

## 想定構成（採用初期）

```text
tier2-go-service/
├── README.md                       # 本ファイル
├── go.mod                          # github.com/k1s0/k1s0/examples/tier2-go-service
├── cmd/
│   └── example-service/
│       └── main.go
├── internal/
│   ├── domain/                     # entity / value object
│   ├── application/                # use case / orchestration
│   └── infrastructure/             # k1s0 sdk wrapping / outbox
├── tests/
│   ├── unit/
│   └── contract/                   # Pact consumer side
├── Dockerfile
└── catalog-info.yaml
```

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md`
- ADR-DEV-001（Paved Road）
- ADR-TIER1-002（Protobuf gRPC）

## 参照する tier1 API（採用初期想定）

- StateService（業務状態の永続化）
- PubSubService（イベント発行）
- AuditService（業務操作の監査ログ）
- WorkflowService（長時間ワークフロー駆動）
