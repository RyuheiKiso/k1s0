# 運用 FAQ

## このドキュメントについて

k1s0 システムの運用者向けのよくある障害パターンと対処法をまとめたガイド。
アラート発火時の初動判断、原因特定、復旧手順を Q&A 形式で記載している。

各 Q&A の構成:
- **症状**: 何が起きているか
- **初動判断**: SEV レベル判定と即座の確認コマンド
- **よくある原因と対処**: パターン別の復旧手順
- **関連ドキュメント**: Runbook・設計書へのリンク

---

## よくある障害パターンと対処法

### Q1: `SYS_AUTH_INVALID_TOKEN` が多発している

#### 症状
- auth-server から 401 Unauthorized が増加
- クライアントがアクセスできなくなっている

#### 初動判断

```bash
# エラー率を Prometheus で確認
rate(http_requests_total{service="auth-server",status="401"}[5m])

# Grafana: auth-server ダッシュボード → Error Rate パネル
```

- エラー率が SLO 目標（< 0.05%）以下 → **SEV3**（監視継続）
- SLO 目標の 2 倍以上 → **SEV2**（詳細調査）
- 1% 超かつ 5 分以上継続 → **SEV1**（即時対応）

#### よくある原因と対処

| パターン | 原因 | 対処 |
|---------|------|------|
| A | Keycloak との鍵同期ずれ（ローテーション後） | auth-server を再起動してキャッシュをクリア |
| B | サーバー間の時刻ずれ（NTP 不整合） | NTP 同期確認、時刻を正確に合わせる |
| C | クライアントが古いトークンを送信 | クライアント側のトークン更新ロジックを確認 |

```bash
# パターン A: auth-server のキャッシュをリセット
kubectl rollout restart deployment/auth-server -n k1s0-system

# パターン B: Pod の時刻を確認
kubectl exec -n k1s0-system deploy/auth-server -- date
```

#### 関連ドキュメント
- [Runbook: エラー率高騰](runbooks/common/high-error-rate.md)
- [auth サービス固有 Runbook](runbooks/services/auth.md)

---

### Q2: `SYS_AUTH_PERMISSION_DENIED` が急増している

#### 症状
- クライアントが 403 Forbidden を受け取る
- 特定の操作がすべてブロックされている

#### 初動判断

```bash
# どのエンドポイントで発生しているか確認
kubectl logs -n k1s0-system deploy/auth-server | grep "PERMISSION_DENIED" | tail -20
```

- 特定エンドポイントのみ → **SEV3**（クライアント・ロール設定の問題）
- 全ユーザーに発生 → **SEV2**（キャッシュ・設定の問題）

#### よくある原因と対処

```bash
# Keycloak でユーザーのロール割当を確認
kubectl port-forward -n keycloak svc/keycloak 8080:8080
# http://localhost:8080/auth/admin → Users → {user} → Role Mappings

# 権限キャッシュをリセット
kubectl rollout restart deployment/auth-server -n k1s0-system
```

---

### Q3: `SYS_SESSION_NOT_FOUND` / `SYS_SESSION_EXPIRED` が発生している

#### 症状
- ユーザーが突然ログアウト状態になる
- セッションが予期なく失効する

#### 初動判断

```bash
# Redis の状態確認
kubectl exec -n k1s0-system redis-0 -- redis-cli INFO memory | grep used_memory_human
kubectl exec -n k1s0-system redis-0 -- redis-cli DBSIZE
```

- 散発的で自然回復 → **SEV3**（正常な有効期限切れ）
- 全ユーザーが同時に失効 → **SEV1**（Redis 障害疑い）

#### よくある原因と対処

| パターン | 原因 | 対処 |
|---------|------|------|
| A | Redis メモリ不足でキー削除 | Redis のメモリ制限を増やすか、削除ポリシーを確認 |
| B | session-server のクラッシュ | session-server を再起動 |
| C | 正常な有効期限切れ | クライアントのリフレッシュトークン実装を確認 |

```bash
# パターン A: Redis メモリポリシー確認
kubectl exec -n k1s0-system redis-0 -- redis-cli CONFIG GET maxmemory-policy

# パターン B: session-server を再起動
kubectl rollout restart deployment/session-server -n k1s0-system
```

---

### Q4: Kafka コンシューマーラグが増え続けている

#### 症状
- メッセージ処理が遅延している
- DLQ にメッセージが蓄積し始めた
- Grafana Kafka ダッシュボードで Consumer Lag が増加中

#### 初動判断

```bash
# コンシューマーラグを確認
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group {consumer-group-name}
```

- ラグが増加傾向、1000 超 → **SEV1**（コンシューマー停止疑い）
- ラグが横ばい → **SEV2**（処理速度不足）
- ラグが自然減少中 → **SEV3**（リバランス中、経過観察）

#### よくある原因と対処

```bash
# コンシューマー Pod の状態確認
kubectl get pods -n {namespace} -l app={consumer-service}

# OOMKilled されていないか確認
kubectl describe pod -n {namespace} {pod-name} | grep -i "oom\|killed\|restart"

# コンシューマーを再起動
kubectl rollout restart deployment/{consumer-service} -n {namespace}

# 処理速度が不足している場合はスケールアウト
kubectl scale deployment/{consumer-service} -n {namespace} --replicas=3
```

#### 関連ドキュメント
- [Runbook: Kafka コンシューマーラグ](runbooks/common/kafka-consumer-lag.md)

---

### Q5: graphql-gateway のレスポンスが遅い / タイムアウトが増加している

#### 症状
- P99 レイテンシが増加
- 504 Gateway Timeout が増える
- クライアントが遅延を報告

#### 初動判断

```bash
# graphql-gateway のログでタイムアウト発生を確認
kubectl logs -n k1s0-system deploy/graphql-gateway | grep -i "timeout\|deadline" | tail -20

# Grafana: graphql-gateway ダッシュボード → P99 Latency パネル
# Jaeger でどのバックエンドが遅いか確認（Min Duration: 1s で絞り込み）
```

- P99 > 5 秒かつ 10 分継続 → **SEV2**（詳細調査）
- SLO バーンレートアラートも同時発火 → **SEV1** へのエスカレーション検討

#### よくある原因と対処

| パターン | 原因 | 対処 |
|---------|------|------|
| A | 依存バックエンドのいずれかが遅い | Jaeger で遅いスパンを特定し、該当サービスの Runbook を参照 |
| B | 全バックエンド同一タイムアウト（既知の制限） | タイムアウト値の調整（config.docker.yaml の `timeout_ms`） |
| C | N+1 クエリ問題 | 開発チームに通知し DataLoader 実装を確認 |

> **既知の制限**: graphql-gateway は全バックエンドに同一タイムアウト値を適用しているため、
> 1 つのバックエンドが遅くなると全クエリのレイテンシに波及する。

#### 関連ドキュメント
- [Runbook: レイテンシ高騰](runbooks/common/high-latency.md)

---

### Q6: DB プール枯渇（connection pool exhausted）エラーが出ている

#### 症状
- ログに「connection pool exhausted」エラー
- サービスが DB へアクセスできず停止寸前

**これは critical アラート（SEV1）。即座に対応する。**

#### 初動判断

```bash
# サービスのログでプール枯渇を確認
kubectl logs -n {namespace} deploy/{service} | grep -i "pool\|exhausted\|connection.*timeout" | tail -20

# Grafana: Database ダッシュボード → Connection Pool Usage パネル
```

#### 緊急対処

```bash
# Pod を再起動してコネクションをリセット（最速の応急処置）
kubectl rollout restart deployment/{service-name} -n {namespace}

# スケールアウトで並列接続数を分散
kubectl scale deployment/{service-name} -n {namespace} --replicas=3
```

#### 根本原因調査

```bash
# PostgreSQL でアイドル状態の接続を確認
kubectl exec -n postgres postgres-0 -- psql -U postgres -c \
  "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# 長時間実行中のクエリを確認
kubectl exec -n postgres postgres-0 -- psql -U postgres -c \
  "SELECT pid, now() - pg_stat_activity.query_start AS duration, query
   FROM pg_stat_activity
   WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';"
```

#### 関連ドキュメント
- [Runbook: DB プール枯渇](runbooks/common/db-pool-exhaustion.md)

---

### Q7: DLQ メッセージが溜まっている

#### 症状
- Dead Letter Queue（DLQ）トピックにメッセージが増加
- `{topic}.dlq` の Kafka オフセットが増え続けている

#### 初動判断

```bash
# DLQ の最新メッセージを確認（最大5件）
kubectl exec -n kafka kafka-0 -- \
  kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic {original-topic}.dlq --max-messages=5 --from-beginning

# DLQ Manager の API で件数を確認
curl http://dlq-manager.k1s0-system/api/v1/messages?status=pending
```

- 緩やかに増加 → **SEV3**（原因調査を開始）
- 急激に増加 → **SEV2**（コンシューマーが停止している可能性）

#### よくある原因と対処

| パターン | 原因 | 対処 |
|---------|------|------|
| A | 処理ロジックのバグ（特定メッセージが常に失敗） | 失敗メッセージの内容を確認、バグ修正・再デプロイ |
| B | 外部依存サービスの一時障害 | 依存サービスを復旧後、DLQ を再処理 |
| C | スキーマ非互換（メッセージ形式の変更） | スキーマバージョンと実装を確認 |

```bash
# DLQ メッセージを再処理（DLQ Manager API）
curl -X POST http://dlq-manager.k1s0-system/api/v1/messages/{id}/retry

# 一括再処理
curl -X POST http://dlq-manager.k1s0-system/api/v1/messages/retry-all \
  -H 'Content-Type: application/json' \
  -d '{"topic": "{original-topic}.dlq"}'
```

---

### Q8: あるサービスだけ `/healthz` が落ちている

#### 症状
- Prometheus で特定サービスが DOWN 表示
- ServiceDown アラートが発火

#### 初動判断

```bash
# Pod の状態確認
kubectl get pods -n {namespace} -l app={service-name}

# ヘルスチェックエンドポイントへの疎通確認
kubectl exec -n {namespace} {pod-name} -- curl -sf http://localhost:8080/healthz
```

- Pod が CrashLoopBackOff → **SEV1**（即時対応）
- Pod が Running だがエンドポイントが応答しない → **SEV2**（詳細調査）

#### 対処

```bash
# ログを確認して起動失敗の原因を特定
kubectl logs -n {namespace} deployment/{service-name} --tail=50

# Pod を再起動
kubectl rollout restart deployment/{service-name} -n {namespace}

# 起動状態を監視
kubectl rollout status deployment/{service-name} -n {namespace}
```

#### 関連ドキュメント
- [Runbook: サービスダウン](runbooks/common/service-down.md)
- [Runbook: Pod 再起動頻発](runbooks/common/pod-restart.md)

---

### Q9: `SYS_SAGA_COMPENSATION_FAILED` が出ている

#### 症状
- 分散トランザクション（Saga）の補償処理（ロールバック相当）が失敗
- 注文・在庫・決済の状態が不整合になっている可能性

#### 初動判断

```bash
# saga-server のログを確認
kubectl logs -n k1s0-system deploy/saga -n k1s0-system | grep -i "compensation\|FAILED" | tail -20

# 関連サービスの状態確認
kubectl get pods -n k1s0-service
```

- 補償処理がリトライ中 → **SEV3**（自動回復を待つ）
- 補償処理が完全停止 → **SEV2**（手動補正が必要）

#### 対処

```bash
# 依存サービスがダウンしているなら復旧
kubectl rollout restart deployment/{dependency-service} -n k1s0-service

# saga-server のログで対象トランザクション ID を確認
kubectl logs -n k1s0-system deploy/saga | grep "transaction_id={id}" | tail -20

# saga-server を再起動してリトライをトリガー
kubectl rollout restart deployment/saga -n k1s0-system
```

> データ不整合が発生している場合は DBA に連絡し、DB レベルで手動補正を検討する。

---

### Q10: bff-proxy が認証エラーを返し続けている（Keycloak 関連）

#### 症状
- bff-proxy が 401/403 を大量に返す
- ユーザーがログイン画面にリダイレクトされ続ける

#### 初動判断

```bash
# bff-proxy のログを確認
kubectl logs -n k1s0-system deploy/bff-proxy | grep -i "auth\|keycloak\|401\|403" | tail -20

# Keycloak の Pod 状態確認
kubectl get pods -n keycloak

# Keycloak への疎通確認
kubectl exec -n k1s0-system deploy/bff-proxy -- \
  curl -sf https://keycloak.internal/auth/health || echo "Keycloak unreachable"
```

- Keycloak が停止 → **SEV1**（即時復旧）
- Keycloak は稼働しているが bff-proxy が疎通できない → **SEV2**

#### 対処

```bash
# Keycloak を再起動
kubectl rollout restart deployment/keycloak -n keycloak
kubectl rollout status deployment/keycloak -n keycloak

# bff-proxy のセッションキャッシュをリセット
kubectl rollout restart deployment/bff-proxy -n k1s0-system

# Keycloak DB（PostgreSQL）の状態確認
kubectl exec -n keycloak deploy/keycloak -- \
  psql -h postgres-keycloak -U keycloak -d keycloak -c "SELECT 1;"
```

> **注意**: Keycloak はシステム全体の認証の単一障害点（SPOF）。
> 停止すると全ユーザーが認証不能になる。
> Keycloak の HA 構成は長期改善計画として検討中。

---

## 既知の制限事項

| 項目 | 説明 | 影響 | 回避策 |
|------|------|------|--------|
| **graphql-gateway の全バックエンド同一タイムアウト** | 全バックエンドに同一の `timeout_ms` を適用 | 1 つのバックエンドが遅くなると全クエリに波及 | 設定ファイルで個別のタイムアウト値を調整（中期改善計画） |
| **Keycloak が認証の単一障害点** | 認証を Keycloak に一元化 | Keycloak 停止で全認証が機能しなくなる | JWKS キャッシュによるフォールバック機構（長期改善計画） |
| **Go BFF Proxy のエラーレスポンス形式** | Rust サービスの ADR-0005 形式と異なる | クライアントの型チェックに例外処理が必要 | BFF エラー形式を ADR-0005 準拠に統一（中期改善計画） |
| **Redis のメモリ制限によるセッション削除** | maxmemory 設定でキーが自動削除 | セッションが予期なく失効 | Redis メモリ制限の増加、または `allkeys-lru` ポリシーに変更 |

---

## エラーコード → Runbook 逆引きテーブル

| エラーコードパターン | 関連サービス | 参照 Runbook | FAQ |
|------|------|------|------|
| `SYS_AUTH_INVALID_TOKEN` | auth-server | [エラー率高騰](runbooks/common/high-error-rate.md) | Q1 |
| `SYS_AUTH_PERMISSION_DENIED` | auth-server | [エラー率高騰](runbooks/common/high-error-rate.md) | Q2 |
| `SYS_SESSION_NOT_FOUND`, `SYS_SESSION_EXPIRED` | session-server | [エラー率高騰](runbooks/common/high-error-rate.md) | Q3 |
| `SYS_GQLGW_UPSTREAM_ERROR` | graphql-gateway | [レイテンシ高騰](runbooks/common/high-latency.md) | Q5 |
| `SYS_SAGA_COMPENSATION_FAILED` | saga-server | — | Q9 |
| `SYS_DLQ_PROCESS_FAILED` | dlq-manager | [Kafka コンシューマーラグ](runbooks/common/kafka-consumer-lag.md) | Q7 |
| `SYS_QUOTA_EXCEEDED`, `SYS_RATELIMIT_RATE_EXCEEDED` | quota/ratelimit | [エラー率高騰](runbooks/common/high-error-rate.md) | — |
| `*_INTERNAL_ERROR`（全サービス） | 各サービス | [サービスダウン](runbooks/common/service-down.md) | Q8 |

エラーコードの完全一覧と説明は [エラーコードカタログ](../conventions/エラーコードカタログ.md) を参照。

---

## アラート発火時の初動フロー

```
[Alertmanager / Teams 通知受信]
    ↓
[SEV レベルを確認]
    ├─ critical → SEV1（15 分以内に初動、Teams #alert-critical に報告）
    ├─ warning  → SEV2（30 分以内に対応開始）
    └─ info     → SEV3（翌営業日 or 経過観察）
    ↓
[上記 FAQ の該当 Q を開く]
    ↓
[「初動判断」コマンドを実行して SEV レベルを確認]
    ↓
[「よくある原因と対処」のパターン A/B/C を順番に試す]
    ↓
[30 分経過しても解決しない → エスカレーション]
    ↓
[復旧後にメトリクス / アラートが正常に戻ったことを確認]
    ↓
[SEV1/SEV2 はポストモーテムを作成]
    └─ テンプレート: runbooks/postmortem-template.md
```

---

## よく使うデバッグコマンド集

### Prometheus クエリ例

```promql
# サービスのエラー率（5 分間）
rate(http_requests_total{service="{service}",status=~"5.."}[5m])
/ rate(http_requests_total{service="{service}"}[5m])

# P99 レイテンシ
histogram_quantile(0.99,
  sum(rate(http_request_duration_seconds_bucket{service="{service}"}[5m])) by (le)
)

# Kafka コンシューマーラグ
kafka_consumergroup_lag_sum{consumer_group="{group}"}

# DB 接続プール使用率
db_pool_connections_in_use / db_pool_connections_max
```

### Loki ログ検索

```
# 特定エラーコードを検索
{namespace="k1s0-system"} |= "SYS_AUTH_INVALID_TOKEN" | json

# 特定サービスのエラーログ
{namespace="k1s0-system", service="{service}"} | json | level="error"

# 直近 5 分のタイムアウトログ
{namespace="k1s0-system"} |= "timeout" | since "5m"
```

### kubectl コマンド集

```bash
# Pod のリソース使用状況を確認
kubectl top pods -n {namespace}

# Pod のイベント・ステータス確認
kubectl describe pod -n {namespace} {pod-name}

# 直前のバージョンへロールバック
kubectl rollout undo deployment/{service} -n {namespace}

# リアルタイムログを確認
kubectl logs -n {namespace} deployment/{service} -f

# Exec で Pod 内からコマンド実行
kubectl exec -n {namespace} {pod-name} -- {command}
```

---

## エスカレーション先

| 条件 | エスカレーション先 | 通知方法 |
|------|-----------------|--------|
| 15 分以内に復旧見通しが立たない（SEV1） | チームリード | Teams #alert-critical |
| 30 分以内に復旧見通しが立たない（SEV2） | チームリード | Teams #alert-warning |
| 複数サービスへの連鎖障害 | チームリード + アーキテクト | Teams + 直接連絡 |
| データ損失・セキュリティ侵害の可能性 | マネージャー + セキュリティチーム | 即座に連絡 |

エスカレーション時の報告内容と詳細フローは [インシデント管理設計](./インシデント管理設計.md) を参照。

---

## 関連ドキュメント

| ドキュメント | 説明 |
|------------|------|
| [エラーコードカタログ](../conventions/エラーコードカタログ.md) | 全エラーコード一覧・HTTP ステータス・説明 |
| [サービス依存関係マップ](../overview/サービス依存関係マップ.md) | サービス間の依存関係・障害影響分析 |
| [インシデント管理設計](./インシデント管理設計.md) | SEV 分類・エスカレーションフロー |
| [SLO 設計](./SLO設計.md) | SLI/SLO/SLA・エラーバジェット運用 |
| [可観測性設計](./可観測性設計.md) | 監視基盤の全体方針 |
| [監視アラート設計](./監視アラート設計.md) | アラートルール・ダッシュボード |
| [Runbook インデックス](./runbooks/README.md) | 全 Runbook 一覧 |
