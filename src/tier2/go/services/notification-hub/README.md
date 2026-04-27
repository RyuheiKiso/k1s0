# notification-hub

tier2 の通知ハブサービス。HTTP `POST /notify` を受け、k1s0 Binding（Dapr `bindings.smtp` / `bindings.http` 等）経由で email / slack / webhook に配信する。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md`
- 内部構造: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md`

## アーキテクチャ

Clean Architecture 4 層構成。stock-reconciler と同水準の構造をとる（採用側組織の業務サービスをこの範例で起こす）。

## エンドポイント

| メソッド | パス | 役割 |
|---|---|---|
| `POST` | `/notify` | 1 件の通知を配信する |
| `GET` | `/healthz` | liveness probe |
| `GET` | `/readyz` | readiness probe |

### POST /notify の入出力

リクエスト Body:

```json
{
  "channel": "email",
  "recipient": "user@example.com",
  "subject": "Reconcile completed",
  "body": "stock for ABC-123 reconciled to 42.",
  "metadata": {
    "priority": "high"
  }
}
```

正常レスポンス:

```json
{
  "notification_id": "...",
  "binding_name": "smtp-outbound",
  "channel": "email",
  "success": true
}
```

`channel` は `email` / `slack` / `webhook` の 3 値（domain.value.Channel で正規化）。`metadata` の値は Dapr Binding の Component 側に追加属性として渡る。

## 環境変数

| 変数 | 既定値 | 必須 | 説明 |
|---|---|---|---|
| `SERVICE_VERSION` | `0.0.0-dev` | - | OTel resource attribute |
| `ENVIRONMENT` | `dev` | - | dev / staging / prod |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | （空） | - | 空なら OTel 初期化 skip |
| `HTTP_ADDR` | `:8080` | - | listen address |
| `K1S0_TARGET` | tier1-state | - | k1s0 facade gRPC target |
| `K1S0_TENANT_ID` | （無し） | ✓ | tier1 ガード必須 |
| `K1S0_SUBJECT` | `tier2/notification-hub` | - | 監査 identity |
| `K1S0_USE_TLS` | `false` | - | 本番 true |
| `BINDING_EMAIL` | `smtp-outbound` | - | email Binding Component 名 |
| `BINDING_SLACK` | `slack-webhook` | - | slack Binding Component 名 |
| `BINDING_WEBHOOK` | `http-outbound` | - | webhook Binding Component 名 |

## エラーコード

| コード | カテゴリ | 説明 |
|---|---|---|
| `E-T2-NOTIF-001` | VALIDATION | チャネル不正 |
| `E-T2-NOTIF-002` | VALIDATION | 通知フィールド不正（recipient / subject / body 必須） |
| `E-T2-NOTIF-010` | INTERNAL | Binding Component 未設定 |
| `E-T2-NOTIF-011` | INTERNAL | 通知 ID 生成失敗 |
| `E-T2-NOTIF-012` | INTERNAL | payload JSON 化失敗 |
| `E-T2-NOTIF-013` | UPSTREAM | Binding Invoke 失敗 |
| `E-T2-NOTIF-100` | VALIDATION | リクエスト Body JSON デコード失敗 |
| `E-T2-INTERNAL` | INTERNAL | 上記以外（panic 等） |

## 将来拡張（リリース時点 → 採用後の運用拡大時）

- PubSub `notification.requested` 購読: HTTP API と同等のロジックを subscribe driven にする
- リトライ + DLQ: Binding 失敗を Kafka DLQ に流して手動再送
- テンプレート: subject / body の Mustache 風テンプレート展開
- multi-recipient: 1 通の配信で複数受信者を扱う（Binding 側で分岐）
