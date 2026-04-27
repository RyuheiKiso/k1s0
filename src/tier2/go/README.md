# tier2 Go services

本ディレクトリは k1s0 tier2 層の Go サービス群を格納する。`src/tier2/go/go.mod` 1 本を全サービスで共有し、`services/<service>/` 配下に Clean Architecture（Onion Architecture）4 層構成の実装を置く。

## 配置正典

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md`
- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md`
- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/06_依存管理.md`

## レイアウト

```text
src/tier2/go/
├── README.md                        # 本ファイル
├── go.mod                           # tier2 Go 全サービス共通 module
├── go.sum
├── services/
│   ├── stock-reconciler/            # 在庫同期サービス
│   └── notification-hub/            # 通知ハブサービス
└── shared/                          # tier2 Go 内部の共通 lib（外部公開 API ではない）
    ├── dapr/                        # k1s0 SDK Client 初期化と Dapr building block ラッパー
    ├── otel/                        # OpenTelemetry 初期化ボイラープレート
    └── errors/                      # tier2 専用エラー型（E-T2-* 体系）
```

## サービス一覧

| サービス | 役割 | 公開ポート |
|---|---|---|
| `services/stock-reconciler/` | 在庫の差分検出 / 同期バッチ。HTTP `POST /reconcile/{sku}` を受け、k1s0 State から現在値を読み、外部システム差分を計算し、PubSub `stock.reconciled` を発火する | 8080 |
| `services/notification-hub/` | 通知ハブ。HTTP `POST /notify` または PubSub `notification.requested` を受け、テンプレ展開のうえ k1s0 Binding 経由で配信する | 8080 |

## ビルド

各サービスは独立にビルド可能（CI の path-filter を生かす設計）。

```bash
# 全サービス静的検査
go vet ./...

# 個別ビルド
go build ./services/stock-reconciler/cmd/
go build ./services/notification-hub/cmd/

# テスト
go test ./...
```

## Dockerfile / CI

各サービス配下に `Dockerfile` と `catalog-info.yaml` を配置する。Dockerfile の build context は `src/tier2/go/` をルートに取る（`docker build -f services/<svc>/Dockerfile .`）。

## 依存方向と禁止事項

- 許可: `src/sdk/go/` を経由して tier1 にアクセス
- 禁止: tier1 / tier3 / contracts の直接 import、他サービスの `internal/` 直接参照
- 共通ロジック: `shared/` に置く（tier2 内部 API、外部公開しない）

`shared/` の可視性は CODEOWNERS + import-boundary lint + 依存方向テストの 3 層防御で強制する（詳細は `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md` 参照）。

## shared サブパッケージ安定度

| サブパッケージ | 安定度ラベル |
|---|---|
| `shared/dapr/` | Alpha（リリース時点 開始、リリース時点 で Beta 目指す） |
| `shared/otel/` | Alpha |
| `shared/errors/` | Alpha |

## 関連 ID

- IMP-DIR-T2-041 / IMP-DIR-T2-043 / IMP-DIR-T2-044 / IMP-DIR-T2-046
- ADR-TIER1-003（内部言語不可視）
- DS-SW-COMP-019
