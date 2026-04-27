# stock-reconciler

tier2 の在庫差分検出 / 同期バッチサービス。外部 WMS 等から「authoritative な在庫数量」を受け取り、k1s0 State 上の現在値と突き合わせ、差分があれば State を更新したうえで PubSub `stock.reconciled` イベントを発火する。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md`
- 内部構造: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md`

## アーキテクチャ

Clean Architecture 4 層構成（Onion Architecture）。

```text
internal/api/                   # HTTP エンドポイント
internal/application/usecases/  # ReconcileUseCase
internal/domain/                # entity / value / event / repository（interface のみ）
internal/infrastructure/        # k1s0 SDK ラッパー / Stock Repository 実装
```

依存方向: `api → application → domain ← infrastructure`。Domain 層は Infrastructure の interface 実装のみを知り、外部依存を一切持たない。

## エンドポイント

| メソッド | パス | 役割 |
|---|---|---|
| `POST` | `/reconcile/{sku}` | 1 SKU の reconcile を実行する |
| `GET` | `/healthz` | liveness probe |
| `GET` | `/readyz` | readiness probe |

### POST /reconcile/{sku} の入出力

リクエスト Body:

```json
{
  "authoritative": 42,
  "source_system": "wms-primary"
}
```

正常レスポンス:

```json
{
  "sku": "ABC-123",
  "before_quantity": 30,
  "after_quantity": 42,
  "delta": 12,
  "event_published": true,
  "event_id": "..."
}
```

エラー時は `errorResponseBody` 形式（E-T2-RECON-* コード + カテゴリ + メッセージ）で返却。HTTP status は `t2errors.Category.HTTPStatus()` の写像（VALIDATION→400 / UPSTREAM→502 / 等）。

## 起動

```bash
# tier2/go ルートから個別ビルド可能。
go build ./services/stock-reconciler/cmd/

# 環境変数（最小）。
export K1S0_TENANT_ID=tenant-dev
export K1S0_TARGET=tier1-state.k1s0-system.svc.cluster.local:50001
export PUBSUB_TOPIC=stock.reconciled

./stock-reconciler
```

## 環境変数

| 変数 | 既定値 | 必須 | 説明 |
|---|---|---|---|
| `SERVICE_VERSION` | `0.0.0-dev` | - | OTel resource attribute |
| `ENVIRONMENT` | `dev` | - | dev / staging / prod |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | （空） | - | 空なら OTel 初期化 skip |
| `HTTP_ADDR` | `:8080` | - | listen address |
| `K1S0_TARGET` | tier1-state | - | k1s0 facade gRPC target |
| `K1S0_TENANT_ID` | （無し） | ✓ | tier1 ガード必須 |
| `K1S0_SUBJECT` | `tier2/stock-reconciler` | - | 監査 identity |
| `K1S0_USE_TLS` | `false` | - | 本番 true |
| `K1S0_STATE_STORE` | `postgres` | - | k1s0 State Component 名 |
| `PUBSUB_COMPONENT` | `kafka` | - | k1s0 PubSub Component 名 |
| `PUBSUB_TOPIC` | `stock.reconciled` | - | publish topic |

## テスト

```bash
# tier2/go ルートから。
go test ./services/stock-reconciler/...
```

リリース時点 では unit / application 層のテストのみ。リリース時点 で testcontainers ベースの integration テストを `tests/integration/` 配下に追加する。

## エラーコード

| コード | カテゴリ | 説明 |
|---|---|---|
| `E-T2-RECON-001` | VALIDATION | SKU 形式不正 |
| `E-T2-RECON-002` | VALIDATION | authoritative が負数 |
| `E-T2-RECON-003` | VALIDATION | source_system が空 |
| `E-T2-RECON-010` | INTERNAL | 0 個 Stock 構築失敗 |
| `E-T2-RECON-011` | UPSTREAM | k1s0 State 取得失敗 |
| `E-T2-RECON-012` | INTERNAL | SyncTo 不変条件違反 |
| `E-T2-RECON-013` | UPSTREAM | k1s0 State 保存失敗 |
| `E-T2-RECON-014` | INTERNAL | イベント JSON 化失敗 |
| `E-T2-RECON-015` | UPSTREAM | PubSub 発火失敗（State は更新済） |
| `E-T2-RECON-100` | VALIDATION | リクエスト Body JSON デコード失敗 |
| `E-T2-INTERNAL` | INTERNAL | 上記以外（panic 等） |

## イベント

`stock.reconciled` トピックに JSON で publish される（`event.StockReconciled`）。

```json
{
  "sku": "ABC-123",
  "before_quantity": 30,
  "after_quantity": 42,
  "delta": 12,
  "source_system": "wms-primary",
  "occurred_at": "2026-04-27T12:00:00Z",
  "event_id": "..."
}
```

`event_id` は冪等性キー（PubSub 重複排除用）。`source_system` は監査要件で必須。
