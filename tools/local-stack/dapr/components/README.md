# `tools/local-stack/dapr/components/` — Dapr Components 定義

ローカル kind 上の本番再現スタックで配備する Dapr Components の YAML を保持する。`up.sh` の `apply_dapr` ステップで `kubectl apply -f` される。

## 一覧

| ファイル | 種別 | 用途 | バックエンド |
|---|---|---|---|
| `state-store-postgres.yaml` | state.postgresql | tier1/tier2/tier3 の状態管理（actor 含む） | CNPG (`k1s0-postgres-rw.cnpg-system`) |
| `pubsub-kafka.yaml` | pubsub.kafka | tier1/tier2 の event 駆動 | Strimzi (`k1s0-kafka-kafka-bootstrap.kafka`) |
| `bindings-http.yaml` | bindings.http | output binding テンプレート | 任意 HTTP |
| `bindings-cron.yaml` | bindings.cron | input binding テンプレート | — |
| `secretstore-openbao.yaml` | secretstores.hashicorp.vault | tier1/tier2 の secret 取り出し | OpenBao dev (`openbao.openbao`) |
| `configstore-flagd.yaml` | configuration.flagd | 機能フラグ参照（OpenFeature 連携） | flagd (`flagd.flagd`) |

namespace は `default` に固定。サービスごとに別 namespace に置く運用が必要になった時点で kustomize overlay へ昇格する。

## tier1 / tier2 / tier3 からの参照

Dapr sidecar が同 Pod に注入されている前提で、アプリは `localhost:3500` 経由で:

```bash
# state-store
curl -X POST http://localhost:3500/v1.0/state/k1s0-state \
  -H 'Content-Type: application/json' \
  -d '[{"key": "test", "value": "v"}]'

# pub-sub
curl -X POST http://localhost:3500/v1.0/publish/k1s0-pubsub/orders \
  -H 'Content-Type: application/json' \
  -d '{"orderId":"123"}'

# secret-store
curl http://localhost:3500/v1.0/secrets/k1s0-secrets/db-password
```

## 関連

- ADR-TIER1-001（Go+Rust ハイブリッド、Dapr facade）
- ADR-DATA-001/002（CNPG / Kafka）
- ADR-SEC-002（OpenBao）
- ADR-FM-001（flagd / OpenFeature）
