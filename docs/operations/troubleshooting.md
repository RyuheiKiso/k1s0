# トラブルシューティングガイド

本ドキュメントは、k1s0 における一般的な問題とその解決方法を定義する。

## 1. よくある問題と解決方法

### 1.1 Pod が起動しない

#### 症状

```bash
$ kubectl get pods -l app={service_name}
NAME                           READY   STATUS             RESTARTS   AGE
user-service-xxx-yyy           0/1     CrashLoopBackOff   5          10m
```

#### 診断手順

```bash
# Pod の詳細確認
kubectl describe pod -l app={service_name}

# ログ確認（前回のコンテナ）
kubectl logs -l app={service_name} --previous

# イベント確認
kubectl get events --sort-by='.lastTimestamp' | grep {service_name}
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| 設定ファイルエラー | `config/{env}.yaml` の構文を確認 |
| Secret 参照エラー | Secret が存在し、キーが正しいか確認 |
| DB 接続失敗 | DB の状態と接続情報を確認 |
| ポート競合 | 他の Pod とポートが重複していないか確認 |
| リソース不足 | resources.requests/limits を調整 |

### 1.2 ヘルスチェック失敗

#### 症状

```bash
$ kubectl get pods -l app={service_name}
NAME                           READY   STATUS    RESTARTS   AGE
user-service-xxx-yyy           0/1     Running   0          5m
```

#### 診断手順

```bash
# ヘルスチェック設定確認
kubectl get pod {pod_name} -o jsonpath='{.spec.containers[0].readinessProbe}'

# 手動ヘルスチェック
kubectl exec -it {pod_name} -- curl http://localhost:8080/healthz
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| 依存サービス未起動 | 依存サービスの状態を確認 |
| DB マイグレーション未完了 | マイグレーションを実行 |
| 初期化タイムアウト | initialDelaySeconds を増加 |
| メモリ不足 | resources.limits.memory を増加 |

### 1.3 高レイテンシ

#### 症状

アラート: `HighLatency` が発火

#### 診断手順

```bash
# 現在のレイテンシ確認（Prometheus）
curl 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,sum(rate(k1s0_{service}_request_duration_seconds_bucket[5m]))by(le))'

# トレース確認
# Jaeger UI で該当時間帯のトレースを検索
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| DB クエリ遅延 | スロークエリログを確認、インデックス追加 |
| 外部 API 遅延 | タイムアウト設定を確認、サーキットブレーカー検討 |
| リソース逼迫 | Pod のスケールアウト |
| N+1 クエリ | コード修正（バッチ取得に変更） |

### 1.4 高エラーレート

#### 症状

アラート: `HighErrorRate` が発火

#### 診断手順

```bash
# エラーの内訳確認（Prometheus）
curl 'http://prometheus:9090/api/v1/query?query=sum(rate(k1s0_{service}_request_failures_total[5m]))by(error_code)'

# エラーログ確認
kubectl logs -l app={service_name} --tail=200 | jq 'select(.level=="ERROR")'
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| 入力バリデーションエラー | クライアント側の修正を依頼 |
| 認証・認可エラー | 認証トークンの有効性を確認 |
| DB 接続エラー | DB の状態を確認、接続プール設定を確認 |
| 外部依存の障害 | 依存サービスの状態を確認 |

### 1.5 メモリリーク

#### 症状

アラート: `HighMemoryUsage` が発火、または OOMKilled

#### 診断手順

```bash
# メモリ使用量の推移確認
kubectl top pod -l app={service_name}

# OOMKilled の確認
kubectl describe pod {pod_name} | grep -A5 "Last State"
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| メモリリーク | プロファイリングで原因特定、コード修正 |
| 設定が小さすぎる | resources.limits.memory を増加 |
| キャッシュ肥大化 | キャッシュ TTL/サイズ制限を設定 |

### 1.6 DB 接続エラー

#### 症状

ログ: `error.code: db.connection_failed`

#### 診断手順

```bash
# DB への疎通確認
kubectl exec -it {pod_name} -- nc -zv {db_host} 5432

# DB 接続数確認（PostgreSQL）
kubectl exec -it {db_pod} -- psql -c "SELECT count(*) FROM pg_stat_activity"
```

#### 原因と対処

| 原因 | 対処 |
|------|------|
| DB ダウン | DB を再起動、レプリカへフェイルオーバー |
| 接続数上限 | max_connections を増加、接続プール導入 |
| ネットワーク障害 | NetworkPolicy、Service 設定を確認 |
| 認証情報誤り | Secret の内容を確認 |

## 2. ログの読み方

### 2.1 基本的なログフィルタリング

```bash
# 直近のログ取得
kubectl logs -l app={service_name} --tail=100

# JSON ログのパース（jq 使用）
kubectl logs -l app={service_name} --tail=100 | jq '.'

# エラーログのみ
kubectl logs -l app={service_name} --tail=500 | jq 'select(.level=="ERROR")'

# 特定の trace_id を追跡
kubectl logs -l app={service_name} --tail=1000 | jq 'select(.trace_id=="abc123def456")'

# 特定のエラーコード
kubectl logs -l app={service_name} --tail=500 | jq 'select(."error.code"=="db.connection_failed")'
```

### 2.2 複数 Pod のログ統合

```bash
# stern を使用（推奨）
stern {service_name} --tail=100 --output json | jq '.'

# kubectl で全 Pod のログ
kubectl logs -l app={service_name} --all-containers --tail=50
```

### 2.3 時間範囲でのフィルタリング

```bash
# 特定時間以降（Loki 使用時）
# Grafana Explore で以下のクエリを実行
{service_name="{service_name}"} |= "ERROR" | json

# 時間範囲指定
{service_name="{service_name}"} |= "ERROR" | json
# Time range: Last 1 hour
```

### 2.4 重要なログフィールド

| フィールド | 用途 |
|-----------|------|
| `trace_id` | 分散トレーシングとの紐付け |
| `span_id` | スパン単位での追跡 |
| `error.kind` | エラー分類（dependency, validation 等） |
| `error.code` | 詳細なエラーコード |
| `grpc.method` / `http.route` | どの API で発生したか |
| `duration_ms` | 処理時間 |

## 3. デバッグ手順

### 3.1 Pod 内でのデバッグ

```bash
# Pod にシェルで接続
kubectl exec -it {pod_name} -- /bin/sh

# 環境変数確認
kubectl exec -it {pod_name} -- env | sort

# 設定ファイル確認
kubectl exec -it {pod_name} -- cat /app/config/config.yaml

# ネットワーク疎通確認
kubectl exec -it {pod_name} -- nc -zv {host} {port}

# DNS 解決確認
kubectl exec -it {pod_name} -- nslookup {service_name}
```

### 3.2 エフェメラルコンテナでのデバッグ

```bash
# デバッグコンテナを追加
kubectl debug -it {pod_name} --image=busybox --target={container_name}

# より高機能なデバッグイメージ
kubectl debug -it {pod_name} --image=nicolaka/netshoot --target={container_name}
```

### 3.3 ローカル再現

```bash
# ポートフォワードで依存サービスに接続
kubectl port-forward svc/postgres 5432:5432 &
kubectl port-forward svc/redis 6379:6379 &

# ローカルで環境変数を設定して実行
export DATABASE_URL="postgres://user:pass@localhost:5432/db"
cargo run  # または go run .
```

### 3.4 トレースの追跡

1. Jaeger/Tempo UI を開く
2. Service を選択
3. 問題の時間帯を指定
4. エラーのあるトレースを選択
5. スパンを展開して詳細を確認

### 3.5 プロファイリング（Go）

```bash
# CPU プロファイル取得
kubectl exec -it {pod_name} -- curl http://localhost:6060/debug/pprof/profile?seconds=30 > cpu.pprof

# メモリプロファイル取得
kubectl exec -it {pod_name} -- curl http://localhost:6060/debug/pprof/heap > heap.pprof

# プロファイル分析
go tool pprof -http=:8080 cpu.pprof
```

## 4. エスカレーション基準

### 4.1 即時エスカレーション

以下の場合は即座にエスカレーション:

- [ ] 複数サービスが同時にダウン
- [ ] 本番 DB に問題
- [ ] セキュリティインシデントの可能性
- [ ] 30 分以上復旧できない

### 4.2 エスカレーション先

| 問題 | エスカレーション先 |
|------|-------------------|
| アプリケーション障害 | サービスオーナー |
| インフラ障害 | インフラチーム |
| DB 障害 | DBA チーム |
| セキュリティ | セキュリティチーム |

## 5. 便利なコマンド集

### 5.1 状態確認

```bash
# 全体の状態確認
kubectl get pods -n k1s0
kubectl get svc -n k1s0
kubectl get ingress -n k1s0

# リソース使用量
kubectl top pods -n k1s0
kubectl top nodes

# 最近のイベント
kubectl get events -n k1s0 --sort-by='.lastTimestamp' | tail -20
```

### 5.2 ログ関連

```bash
# 複数コンテナのログ
stern -n k1s0 {service_name} --tail=100

# 前回のコンテナログ
kubectl logs -l app={service_name} --previous
```

### 5.3 ネットワーク診断

```bash
# Service の Endpoint 確認
kubectl get endpoints {service_name}

# DNS 確認
kubectl run -it --rm debug --image=busybox --restart=Never -- nslookup {service_name}
```

## 6. k1s0 Docker / Playground のトラブルシューティング

### 6.1 Docker コマンドの問題

#### k1s0 docker build が失敗する

```bash
# コンテナ状態を確認
k1s0 docker status
k1s0 docker status --json

# キャッシュを無効にして再ビルド
k1s0 docker build --no-cache

# プロキシ環境の場合
k1s0 docker build --http-proxy http://proxy:8080
```

| 原因 | 対処 |
|------|------|
| Dockerfile の構文エラー | Dockerfile を確認（K060 ルールも参照） |
| ベースイメージ取得失敗 | ネットワーク接続・プロキシ設定を確認 |
| ディスク容量不足 | `docker system prune` で不要イメージを削除 |

#### k1s0 docker compose up が失敗する

```bash
# ログを確認
k1s0 docker compose logs -f
k1s0 docker compose logs {service_name}

# サービスを停止してボリュームごと削除し再起動
k1s0 docker compose down -v
k1s0 docker compose up -d --build
```

### 6.2 Playground コマンドの問題

#### playground が起動しない

```bash
# 利用可能なテンプレートを確認
k1s0 playground list

# 実行中の playground を確認
k1s0 playground status
k1s0 playground status --json

# 起動（オプション指定例）
k1s0 playground start --type backend-rust --mode full --with-db --with-cache
```

| 原因 | 対処 |
|------|------|
| ポート競合 | `--port-offset` で別ポートを指定 |
| 既存 playground が残っている | `k1s0 playground stop --name {name} -v` で削除後に再起動 |
| Docker デーモン未起動 | Docker Desktop / dockerd を起動 |

#### playground の停止・削除

```bash
# 特定の playground を停止
k1s0 playground stop --name {name}

# ボリュームも含めて削除（確認スキップ）
k1s0 playground stop --name {name} -v -y
```

## 関連ドキュメント

- [モニタリング・アラート](monitoring.md)
- [デプロイメント手順](deployment.md)
- [サービス再起動](runbooks/service-restart.md)
- [インシデント対応](runbooks/incident-response.md)
