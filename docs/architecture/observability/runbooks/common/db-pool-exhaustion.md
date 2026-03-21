# アラート: DB プール枯渇

対象アラート: `DbPoolExhaustion`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical |
| **影響範囲** | 対象サービスの全 DB アクセス（リクエストが詰まりタイムアウトが発生） |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1（15分以内） |

## アラート発火条件

- DB コネクションプール使用率 > 90% で 5分継続

## 初動対応（5分以内）

### 1. 現在のプール使用率を確認

```bash
# Prometheus でプール状況確認
# db_pool_connections_in_use{service="{service-name}"} / db_pool_connections_max{service="{service-name}"}
```

Grafana → データベースダッシュボード → Connection Pool パネルを確認:
- [データベースダッシュボード](http://grafana.k1s0.internal/d/k1s0-db/database-dashboard)

### 2. 影響を受けているリクエストの確認

```bash
# コネクション待ちでエラーが増えているか確認
kubectl logs -n {namespace} deploy/{service-name} --tail=50 | grep -i "pool\|connection\|timeout"
```

### 3. 即時判断

- [ ] 使用率 100% でリクエストエラーが急増 → SEV1（即時エスカレーション）
- [ ] 90〜95% で安定中 → SEV2（詳細調査）

## 詳細調査

### よくある原因

1. **コネクションリーク**: コネクションを返却しない処理があり、使用中コネクション数が増え続ける
2. **トラフィック急増**: リクエスト数の増加によりプールが不足
3. **スロークエリ**: 長時間クエリが実行されコネクションを占有
4. **DB サーバーのハング**: DB が応答遅延でコネクションが詰まる

### 調査コマンド

```bash
# DB 側でのコネクション状況確認（PostgreSQL）
kubectl exec -n {namespace} deploy/{service-name} -- \
  psql $DATABASE_URL -c "
    SELECT state, count(*), max(now() - state_change) as max_duration
    FROM pg_stat_activity
    GROUP BY state
    ORDER BY count DESC;
  "

# 長時間実行クエリの確認
kubectl exec -n {namespace} deploy/{service-name} -- \
  psql $DATABASE_URL -c "
    SELECT pid, now() - query_start as duration, query, state
    FROM pg_stat_activity
    WHERE state != 'idle' AND now() - query_start > interval '30 seconds'
    ORDER BY duration DESC;
  "
```

### Prometheus クエリ例

```promql
# プール使用率の推移
db_pool_connections_in_use{namespace=~"k1s0-.*"}
/ db_pool_connections_max{namespace=~"k1s0-.*"}

# コネクション数の増加傾向
rate(db_pool_connections_in_use{service="{service-name}"}[10m])
```

## 復旧手順

### パターン A: コネクションリーク（緊急）

```bash
# アイドルコネクションを強制的に終了（DB 側）
kubectl exec -n {namespace} deploy/{service-name} -- \
  psql $DATABASE_URL -c "
    SELECT pg_terminate_backend(pid)
    FROM pg_stat_activity
    WHERE state = 'idle' AND now() - state_change > interval '5 minutes';
  "

# サービスの再起動でコネクションを解放（最終手段）
kubectl rollout restart deployment/{service-name} -n {namespace}
```

### パターン B: トラフィック急増

```bash
# スケールアウトによるプール容量の増加
kubectl scale deployment/{service-name} -n {namespace} --replicas={n}

# プールサイズの一時的な増加（環境変数で設定している場合）
kubectl set env deployment/{service-name} -n {namespace} DATABASE_POOL_SIZE=20
```

### パターン C: スロークエリ

```bash
# 長時間クエリの強制終了（DBA に相談の上）
kubectl exec -n {namespace} deploy/{service-name} -- \
  psql $DATABASE_URL -c "SELECT pg_cancel_backend({pid});"
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] コネクション使用率 100% でリクエストエラーが急増
- [ ] サービス再起動後も使用率が回復しない（リークが疑われる）
- [ ] DB サーバー自体に問題がある可能性
- [ ] 15分以内に使用率が 80% 未満に回復しない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- コネクションを返却しているかコードレビューで確認（トランザクション内の早期 return など）
- DB 側の `pg_stat_activity` で長期間 idle-in-transaction なコネクションがないか確認
- プールサイズの設定値がトラフィックに対して適切かどうか見直す

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [ログ設計](../../ログ設計.md)
