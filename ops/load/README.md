# k1s0 負荷テスト（k6）

本ディレクトリは k1s0 tier1 facade の HTTP/JSON gateway を対象にした
[k6](https://k6.io/) シナリオを集約する。

## 設計正典

- `docs/03_要件定義/30_非機能要件/B_性能.md`（NFR-B-PERF-*）
- `docs/03_要件定義/30_非機能要件/A_可用性.md`（NFR-A-SLA-*）
- `docs/03_要件定義/00_共通規約.md`（§ レート制限 / § 冪等性と再試行）
- `ops/runbooks/daily/error-code-alert-policy.md`

## ディレクトリ構成

```
ops/load/
├── README.md            ← 本ファイル
├── k6/
│   └── helpers/
│       └── common.js    ← 共通 helper（環境変数 / endpoint / auth header）
└── scenarios/
    ├── state_baseline.js      ← State Save/Get/Delete の baseline 性能（p95/p99 SLA threshold）
    ├── rate_limit.js          ← RateLimitInterceptor の 429 動作確認
    └── idempotency_replay.js  ← 同一 idempotency_key で dedup 動作確認
```

## 起動例

### 前提

- k6 0.50+ がインストール済（`brew install k6` / `apt install k6` 等）
- 対象 Pod が `TIER1_HTTP_LISTEN_ADDR` で HTTP/JSON gateway を起動済
  （`TIER1_AUTH_MODE=off` でも可、`hmac` の場合は token を渡す）

### 環境変数

| 変数 | 用途 | 既定値 |
|---|---|---|
| `K6_TARGET_BASE` | 対象 BFF / tier1 facade の base URL | `http://localhost:50080` |
| `K6_AUTH_TOKEN`  | `Authorization: Bearer <token>` の token | `dev` |
| `K6_TENANT_ID`   | request body の `context.tenantId` | `demo-tenant` |

### baseline 性能

```bash
# 1 VU / 60 秒で State.Save→Get→Delete をループ
K6_TARGET_BASE=http://t1-state:50080 K6_AUTH_TOKEN="dev" \
  k6 run ops/load/scenarios/state_baseline.js
```

threshold 違反時はゼロ以外の exit code で fail。CI で gating 可能。

### レート制限の動作確認

```bash
# 別 shell で Pod を burst 用設定で起動
TIER1_AUTH_MODE=off TIER1_RATELIMIT_RPS=2 TIER1_RATELIMIT_BURST=3 \
  TIER1_HTTP_LISTEN_ADDR=:50080 ./t1-pii &

# k6 を流す
K6_TARGET_BASE=http://localhost:50080 \
  k6 run ops/load/scenarios/rate_limit.js
```

期待結果: 12 件 burst のうち 3-4 件が 200、残りが 429。1.5 秒待機後の
recovery req は 200 を返す（token bucket 補充の証跡）。

### idempotency dedup の動作確認

```bash
K6_TARGET_BASE=http://t1-audit:50080 K6_AUTH_TOKEN="$JWT" \
  k6 run ops/load/scenarios/idempotency_replay.js
```

期待結果: 同一 idempotency_key で 5 回 Audit.Record しても全 5 件が
同じ `audit_id` を返し、verifyChain の `checked_count` は 1 件のみ
増える（hash chain 二重追記が防がれている）。

## CI 統合

GitHub Actions ワークフロー `.github/workflows/_reusable-test.yml` で
`k6` job を有効化したら本シナリオが自動実行される（実装は post-MVP）。

## 採用初期 で追加予定のシナリオ

- マルチテナント分離負荷（テナント A の RPS が上がっても B が SLO 維持）
- Workflow.Start の入力 fan-out（背圧バックプレッシャ確認）
- Audit.Export server-streaming（gRPC 経路のため別途 grpcurl ベースに移行想定）
