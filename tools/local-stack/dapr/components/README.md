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

namespace は `default` に固定。サービスごとに別 namespace に置く運用が必要になった時点で kustomize overlay へ昇格する。

flagd（FR-T1-FEATURE-001）は Dapr Component 経由ではなく、tier1 facade が OpenFeature
gRPC SDK で `flagd.flagd.svc.cluster.local:8013` に直結する。Dapr 1.17.5 には
`configuration.flagd` component type は存在せず、誤って Component を配備すると
**namespace 内の全 Dapr enabled Pod が daprd 起動時に fatal で死ぬ**ため絶対に置かない。
ADR-FM-001（flagd / OpenFeature 採用）と整合する直結経路を維持する。

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
