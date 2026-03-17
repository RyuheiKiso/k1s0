# パフォーマンステスト戦略

F-011: 負荷テスト戦略、ベンチマーク基準、パフォーマンス目標、ツール選定を定義する。

---

## 基本方針

- パフォーマンステストは **リリース前の品質ゲート** として位置づける
- 本番環境に近い構成のテスト環境で実行し、結果の信頼性を確保する
- パフォーマンス目標を SLO と連携させ、定量的な基準で合否を判定する
- ベンチマークの結果は時系列で蓄積し、パフォーマンスリグレッションを検知する

---

## パフォーマンス目標

### レイテンシ目標

| Tier | エンドポイント種別 | P50 | P95 | P99 | 備考 |
| --- | --- | --- | --- | --- | --- |
| system | gRPC API（認証・設定取得） | 10ms | 50ms | 100ms | 高頻度呼び出しのため厳格に設定 |
| system | REST API（BFF 経由） | 50ms | 200ms | 500ms | BFF → バックエンド → DB の往復を含む |
| business | gRPC API | 20ms | 100ms | 200ms | ドメインロジック処理を含む |
| service | REST API（BFF 経由） | 100ms | 300ms | 1000ms | 複数サービス呼び出しを含む場合あり |
| service | Saga トランザクション | 500ms | 2000ms | 5000ms | 複数サービスの非同期オーケストレーション |

### スループット目標

| Tier | サービス | 目標 RPS | 備考 |
| --- | --- | --- | --- |
| system | auth-server | 1,000 RPS | トークン検証・発行 |
| system | config-server | 500 RPS | 設定値の取得 |
| system | bff-proxy | 2,000 RPS | API ゲートウェイとしての集約処理 |
| system | saga-server | 200 RPS | 分散トランザクション管理 |
| service | 各ドメインサービス | 500 RPS | 個別ビジネスロジック |

### リソース使用量目標

| メトリクス | 正常時上限 | アラート閾値 |
| --- | --- | --- |
| CPU 使用率 | 60% | 80%（5 分間平均） |
| メモリ使用率 | 70% | 85%（5 分間平均） |
| DB コネクションプール使用率 | 60% | 80% |
| Kafka Consumer Lag | 1,000 メッセージ | 10,000 メッセージ |

---

## 負荷テスト戦略

### テスト種別

| 種別 | 目的 | 負荷パターン | 実行頻度 |
| --- | --- | --- | --- |
| ベースライン測定 | 通常時のパフォーマンス基準値を取得する | 定常負荷（目標 RPS の 50%） | リリースごと |
| 負荷テスト | 目標 RPS での正常動作を確認する | 定常負荷（目標 RPS の 100%） | リリースごと |
| ストレステスト | 限界性能と劣化パターンを把握する | 段階的に増加（目標 RPS の 150% → 200%） | 月次 |
| スパイクテスト | 急激な負荷増加に対する耐性を確認する | 瞬間的に目標 RPS の 300% | 四半期 |
| 耐久テスト | 長時間稼働でのリソースリークを検出する | 定常負荷を 2 時間継続 | 四半期 |

### 負荷テストシナリオ

#### シナリオ 1: 認証フロー負荷テスト

```
1. トークン取得（POST /auth/token）
2. トークン検証（GET /auth/verify）
3. ユーザー情報取得（GET /api/v1/users/me）
```

- **目標**: 1,000 RPS で P99 < 100ms
- **同時接続数**: 100 VU（Virtual Users）
- **持続時間**: 10 分

#### シナリオ 2: CRUD 操作負荷テスト

```
1. リソース作成（POST /api/v1/resources）
2. リソース一覧取得（GET /api/v1/resources?page=1&size=20）
3. リソース更新（PUT /api/v1/resources/{id}）
4. リソース削除（DELETE /api/v1/resources/{id}）
```

- **目標**: 500 RPS で P99 < 500ms
- **同時接続数**: 50 VU
- **持続時間**: 10 分

#### シナリオ 3: Saga 分散トランザクション負荷テスト

```
1. Saga 開始（POST /api/v1/sagas）
2. ステップ実行の進捗監視（GET /api/v1/sagas/{id}）
3. 補償トランザクション発火（10% の確率で失敗を注入）
```

- **目標**: 200 RPS で P99 < 5000ms
- **同時接続数**: 30 VU
- **持続時間**: 15 分

---

## ツール選定

### 負荷テストツール

| ツール | 用途 | 対象 | 選定理由 |
| --- | --- | --- | --- |
| **k6** | HTTP / gRPC 負荷テスト | サービスエンドポイント | JavaScript でシナリオを記述でき、CI 統合が容易 |
| **wrk** | HTTP ベンチマーク | 単一エンドポイントの最大性能測定 | 軽量・高速で最大 RPS の測定に適する |
| **criterion** | マイクロベンチマーク | Rust 関数レベル | Rust エコシステム標準のベンチマークライブラリ |
| **ghz** | gRPC ベンチマーク | gRPC エンドポイント | gRPC 特化の負荷テストツール |

### k6 テストスクリプト例

```javascript
// tests/performance/scenarios/auth-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

// テスト設定: 段階的に負荷を増加させる
export const options = {
  stages: [
    { duration: '2m', target: 50 },    // ウォームアップ: 50 VU まで増加
    { duration: '5m', target: 100 },   // 定常負荷: 100 VU を維持
    { duration: '2m', target: 200 },   // ストレス: 200 VU まで増加
    { duration: '1m', target: 0 },     // クールダウン: 0 VU まで減少
  ],
  thresholds: {
    // P95 レイテンシが 200ms 未満であること
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    // エラー率が 1% 未満であること
    http_req_failed: ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// 認証フローの負荷テスト
export default function () {
  // 1. トークン取得
  const tokenRes = http.post(`${BASE_URL}/auth/token`, JSON.stringify({
    username: 'load-test-user',
    password: 'load-test-password',
    grant_type: 'password',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(tokenRes, {
    'トークン取得成功': (r) => r.status === 200,
    'アクセストークンが含まれる': (r) => JSON.parse(r.body).access_token !== undefined,
  });

  if (tokenRes.status !== 200) return;

  const token = JSON.parse(tokenRes.body).access_token;

  // 2. API 呼び出し
  const apiRes = http.get(`${BASE_URL}/api/v1/users/me`, {
    headers: { Authorization: `Bearer ${token}` },
  });

  check(apiRes, {
    'API 呼び出し成功': (r) => r.status === 200,
  });

  sleep(1);
}
```

### Rust criterion ベンチマーク例

```rust
// benches/serialization_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

/// シリアライゼーション性能のベンチマーク
fn bench_serialize_event(c: &mut Criterion) {
    let event = create_test_event();

    c.bench_function("event_serialize_json", |b| {
        b.iter(|| serde_json::to_string(&event).unwrap())
    });

    c.bench_function("event_serialize_protobuf", |b| {
        b.iter(|| event.encode_to_vec())
    });
}

criterion_group!(benches, bench_serialize_event);
criterion_main!(benches);
```

### wrk ベンチマーク例

```bash
# 単一エンドポイントの最大 RPS を測定する
# 12 スレッド、400 接続で 30 秒間実行
wrk -t12 -c400 -d30s http://localhost:8080/api/v1/health

# Lua スクリプトで POST リクエストのベンチマーク
wrk -t12 -c400 -d30s -s tests/performance/scripts/post-request.lua \
  http://localhost:8080/api/v1/resources
```

---

## ベンチマーク基準

### パフォーマンスリグレッション検知

ベンチマーク結果を時系列で保存し、前回リリースとの差分を自動比較する。

| メトリクス | 許容劣化幅 | アクション |
| --- | --- | --- |
| P50 レイテンシ | +10% | 警告（PR コメント） |
| P95 レイテンシ | +15% | 警告（PR コメント） |
| P99 レイテンシ | +20% | マージブロック |
| スループット（RPS） | -10% | 警告（PR コメント） |
| メモリ使用量 | +20% | 警告（PR コメント） |

### criterion ベースラインとの比較

Rust のマイクロベンチマークは criterion のベースライン機能で自動比較する。

```bash
# ベースラインの保存（main ブランチ）
cargo bench -- --save-baseline main

# 現在のブランチとの比較
cargo bench -- --baseline main
```

---

## CI/CD パイプライン統合

### 実行タイミング

| トリガー | テスト種別 | 目的 |
| --- | --- | --- |
| PR 時 | criterion マイクロベンチマーク | Rust コードレベルのリグレッション検知 |
| main マージ後 | k6 負荷テスト（ベースライン） | リリース候補のパフォーマンス検証 |
| 月次スケジュール | k6 ストレステスト | 限界性能の把握 |
| 四半期スケジュール | k6 耐久テスト + スパイクテスト | 長時間稼働の安定性検証 |

### GitHub Actions ワークフロー

```yaml
# .github/workflows/performance-test.yaml
name: Performance Test

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 19 * * 0'   # JST 月曜 04:00（週次）
  workflow_dispatch:
    inputs:
      test_type:
        description: 'テスト種別 (baseline / stress / spike / endurance)'
        required: true
        default: 'baseline'

jobs:
  performance-test:
    runs-on: ubuntu-latest
    timeout-minutes: 120
    steps:
      - uses: actions/checkout@v4

      # テスト環境のセットアップ
      - name: Setup performance test environment
        run: |
          helm upgrade --install perf-env infra/helm/environments/performance \
            --namespace perf-test \
            --create-namespace \
            --wait --timeout 10m

      # k6 のインストール
      - name: Install k6
        run: |
          curl -sL https://github.com/grafana/k6/releases/latest/download/k6-linux-amd64.tar.gz | tar xz
          mv k6-linux-amd64/k6 /usr/local/bin/

      # 負荷テストの実行
      - name: Run load test
        run: |
          k6 run tests/performance/scenarios/auth-load-test.js \
            --out json=results/auth-load-test.json \
            -e BASE_URL=http://bff-proxy.perf-test.svc.cluster.local:8080

      # 結果の保存
      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: performance-test-results
          path: results/

      # 環境のクリーンアップ
      - name: Cleanup
        if: always()
        run: |
          helm uninstall perf-env --namespace perf-test || true
          kubectl delete namespace perf-test || true
```

### テスト結果の可視化

パフォーマンステストの結果は Grafana ダッシュボードで可視化する。

| ダッシュボード | 表示内容 |
| --- | --- |
| Performance Overview | レイテンシ P50/P95/P99 の時系列推移 |
| Throughput Trends | RPS の時系列推移と目標ライン |
| Resource Usage | CPU / メモリ / DB コネクション使用率 |
| Regression Detection | 前回リリースとの差分比較 |

---

## テストデータ

パフォーマンステスト用のテストデータは E2E テストとは別に管理する。

### データ量

| データ種別 | レコード数 | 理由 |
| --- | --- | --- |
| ユーザー | 10,000 | 本番想定規模の 10% |
| 設定項目 | 1,000 | 全サービスの設定を網羅 |
| トランザクション履歴 | 100,000 | ページネーション・検索の性能検証 |

### テストユーザー

負荷テスト用のテストユーザーは Keycloak に一括登録する。

```bash
# 負荷テスト用ユーザーの一括作成（100 ユーザー）
for i in $(seq 1 100); do
  kcadm.sh create users -r k1s0 \
    -s username="perf-user-${i}" \
    -s enabled=true \
    -s "credentials=[{\"type\":\"password\",\"value\":\"perf-test-password\"}]"
done
```

---

## 関連ドキュメント

- [テスト戦略](./test-strategy.md) -- テストピラミッド・言語別フレームワーク・カバレッジ目標
- [E2Eテスト戦略](./e2e-strategy.md) -- E2E テストの詳細設計
- [SLO設計.md](../observability/SLO設計.md) -- SLO/SLA 定義・エラーバジェット
- [可観測性設計.md](../observability/可観測性設計.md) -- Prometheus・Grafana 設定
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md) -- CI/CD パイプライン設計
