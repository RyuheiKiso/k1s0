# docker-compose 可観測性サービス設計

docker-compose における Prometheus・Grafana・Loki・Jaeger の詳細設定を定義する。基本方針・プロファイル設計は [docker-compose設計.md](docker-compose設計.md) を参照。設計の全体像は [可観測性設計](../../architecture/observability/可観測性設計.md) を参照。

---

## 起動方法

### 一括起動コマンド

observability スタック（Grafana・Prometheus・Jaeger・Loki・Promtail）は以下のいずれかのコマンドで一括起動できる。

```bash
# MEDIUM-003 / LOW-003 監査対応: "local-up-" プレフィックスで統一されたエイリアス（推奨）
just local-up-obs

# 上記と同等のコマンド（実体は同じ）
just observability-up
```

`just local-up-dev`（フル開発環境起動）でも `--profile observability` が含まれるため、通常の開発環境セットアップでは個別起動は不要。

### サービスポート一覧

| サービス | ホストポート | 用途 |
| --- | --- | --- |
| Grafana | **3200** | ダッシュボード UI（`http://localhost:3200`）。ポート 3000 はフロントエンド開発サーバーと競合するため 3200 を使用。 |
| Prometheus | **9090** | メトリクス収集 UI・クエリ（`http://localhost:9090`） |
| Jaeger | **16686** | 分散トレーシング UI（`http://localhost:16686`） |
| Loki | **3100** | ログ集約 API（Grafana からのクエリ専用。直接アクセス不要） |
| Promtail | **9080** | ログ収集エージェント状態確認（`http://localhost:9080/ready`） |

> **Grafana ポートの注意**: ホストポートは `3200` であり、一般的な `3000` ではない。詳細は「Grafana アクセスポートに関する注意（MED-002 対応）」セクションを参照。

---

## Observability サービス詳細設定

### Prometheus scrape 設定

```yaml
# infra/docker/prometheus/prometheus.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/recording_rules.yaml
  - /etc/prometheus/alerting_rules.yaml

scrape_configs:
  # Prometheus 自身
  # HIGH-003 監査対応: web.yml で Basic Auth が有効化されているため、
  # セルフスクレイプにも basic_auth を設定して 401 エラーを防止する。
  - job_name: prometheus
    basic_auth:
      username: prometheus_admin
      password: prometheus_admin
    static_configs:
      - targets: ["localhost:9090"]

  # auth-server (Rust)
  - job_name: auth-server-rust
    static_configs:
      - targets: ["auth-rust:8080"]
        labels:
          service: auth-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # config-server (Rust)
  - job_name: config-server-rust
    static_configs:
      - targets: ["config-rust:8080"]
        labels:
          service: config-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # saga-server (Rust)
  - job_name: saga-server-rust
    static_configs:
      - targets: ["saga-rust:8080"]
        labels:
          service: saga-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # dlq-manager (Rust)
  - job_name: dlq-manager
    static_configs:
      - targets: ["dlq-manager:8080"]
        labels:
          service: dlq-manager
          tier: system
          lang: rust
    metrics_path: /metrics

  # bff-proxy (Go)
  # MED-001 監査対応: bff-proxy は /metrics を公開ポート(8080)ではなく
  # 内部ポート(9090)で提供する（server.internal_port: 9090）。
  - job_name: bff-proxy-go
    static_configs:
      - targets: ["bff-proxy:9090"]
        labels:
          service: bff-proxy
          tier: system
          lang: go
    metrics_path: /metrics

  # Kong API Gateway
  - job_name: kong
    static_configs:
      - targets: ["kong:8001"]
    metrics_path: /metrics
```

### Grafana 自動プロビジョニング

#### データソース

```yaml
# infra/docker/grafana/provisioning/datasources/datasources.yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    editable: false
    jsonData:
      derivedFields:
        - datasourceUid: jaeger
          matcherRegex: '"trace_id":"([a-f0-9]+)"'
          name: TraceID
          url: '$${__value.raw}'

  - name: Jaeger
    type: jaeger
    access: proxy
    uid: jaeger
    url: http://jaeger:16686
    editable: false
```

#### ダッシュボードプロビジョニング

```yaml
# infra/docker/grafana/provisioning/dashboards/dashboard.yml
apiVersion: 1

providers:
  - name: k1s0
    orgId: 1
    folder: k1s0
    type: file
    disableDeletion: false
    editable: true
    updateIntervalSeconds: 30
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: false
```

### Loki 設定

```yaml
# ローカル開発用（シングルインスタンス）
# 設定ファイル: infra/docker/loki/loki-config.yaml をマウント
# Kubernetes 環境では Promtail（DaemonSet）がログを収集し Loki に転送するが、
# ローカルでは各コンテナの stdout を直接 docker compose logs で確認する。
# Loki はダッシュボード経由でのログ検索用途で提供する。
```

### Promtail 設定

```yaml
# infra/docker/promtail/promtail-config.yaml をマウント
# 役割: ログ収集エージェント
#   - Docker SD（Service Discovery）でコンテナを自動検出
#   - JSON ログをパースしてラベル付け
#   - 収集したログを Loki へフォワーディング
```

```yaml
# docker-compose における Promtail サービス定義例
services:
  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./infra/docker/promtail/promtail-config.yaml:/etc/promtail/config.yaml:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - /var/run/docker.sock:/var/run/docker.sock
    command: -config.file=/etc/promtail/config.yaml
    depends_on:
      - loki
```

| 項目 | 設定 |
| --- | --- |
| イメージ | `grafana/promtail:latest` |
| 設定ファイル | `./infra/docker/promtail/promtail-config.yaml` |
| ログ収集方式 | Docker SD（コンテナ自動検出） |
| ログ形式 | JSON パース |
| 転送先 | `loki:3100` |

#### docker.sock マウントのセキュリティリスクと代替案（M-8 対応）

`/var/run/docker.sock` をコンテナにマウントすると、Docker デーモンへの**完全なアクセス権**が付与される。悪用された場合、コンテナからホストの全コンテナを操作・停止・削除できる。

| リスク | 影響 |
| --- | --- |
| コンテナエスケープ | docker.sock 経由でホスト権限を取得可能 |
| 任意コンテナ起動 | privileged コンテナを起動してホストへアクセス可能 |
| サービス妨害 | 全コンテナを停止・削除可能 |

**本番環境での代替案**:

1. **Kubernetes DaemonSet + hostPath**（推奨）: Promtail を DaemonSet として各ノードに配置し、`/var/lib/docker/containers` をマウントして直接ログファイルを読み込む。docker.sock は不要。
2. **Fluent Bit**（軽量代替）: eBPF ベースのログ収集が可能。docker.sock 不要で最小権限での実行が可能。
3. **Docker ログドライバー変更**: `logging.driver: syslog` または `gelf` に変更することで Promtail 自体が不要になる。

**ローカル開発環境での判断**: docker.sock マウントは開発環境での手軽さを優先したトレードオフ。本番 Kubernetes 環境では DaemonSet 方式を採用すること。

### Jaeger 設定

```yaml
# OTLP プロトコルで各サービスからトレースデータを受信する
# - OTLP gRPC: 4317（サービスからの送信先）
# - OTLP HTTP: 4318（HTTP 経由の送信先）
# - UI: 16686（Jaeger UI）
```

| 項目 | 設定 |
| --- | --- |
| OTLP gRPC | `jaeger:4317` |
| OTLP HTTP | `jaeger:4318` |
| UI | `localhost:16686` |

---

## Healthcheck 設定（M-4 対応）

全 observability サービスに Docker Compose の `healthcheck` を設定し、依存サービスが起動完了後に利用可能になるまで待機できるようにする。

| サービス | ヘルスチェックエンドポイント | 用途 |
| --- | --- | --- |
| jaeger | `http://localhost:14269/health` | 管理 API ヘルスエンドポイント |
| prometheus | `http://localhost:9090/-/healthy` | Prometheus 組み込みヘルスエンドポイント |
| loki | `http://localhost:3100/ready` | Loki ready エンドポイント |
| promtail | `http://localhost:9080/ready` | Promtail ready エンドポイント |
| grafana | `http://localhost:3000/api/health` | Grafana API ヘルスエンドポイント |

```yaml
# 全 observability サービス共通の healthcheck 設定パターン
healthcheck:
  test: ["CMD-SHELL", "wget -qO- http://localhost:<PORT>/<PATH> || exit 1"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 30s
```

**`start_period: 30s`** を設定することで、起動直後の false negative（起動中なのに unhealthy 判定）を防止する。

---

## Grafana データベース本番移行方針（M-11 対応）

ローカル開発環境では Grafana は SQLite（デフォルト）を使用する。本番環境では可用性・バックアップ・同時接続性を確保するために PostgreSQL への移行が必要。

### 本番環境への移行手順

1. **PostgreSQL に `grafana` データベースを作成する**

```sql
CREATE DATABASE grafana;
CREATE USER grafana WITH PASSWORD '<secret>';
GRANT ALL PRIVILEGES ON DATABASE grafana TO grafana;
```

2. **環境変数で Grafana の DB を設定する**（`docker-compose.prod.yaml` または Kubernetes Secret）

```yaml
environment:
  GF_DATABASE_TYPE: postgres
  GF_DATABASE_HOST: <postgres-host>:5432
  GF_DATABASE_NAME: grafana
  GF_DATABASE_USER: grafana
  GF_DATABASE_PASSWORD: <secret>
  GF_DATABASE_SSL_MODE: require
```

3. **既存 SQLite データを移行する場合**: [Grafana 公式ドキュメント](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-monitoring/) の DB migration 手順を参照。

> **注意**: ダッシュボードは `provisioning/` 経由でコード管理しているため、DB 移行不要でダッシュボードは再現できる。ユーザー・アラート設定のみ要移行。

---

## 軽量モード（ローカル PC 向け）

ローカル PC のリソースが限られている場合、OTel スタック（Jaeger・Prometheus・Grafana・Loki）を共用開発サーバーに寄せ、ローカルでは起動しない運用パターンを推奨する。

### 構成

```
[ローカル PC]                          [共用開発サーバー]
  docker compose --profile infra        docker compose --profile observability
  ├── postgres, kafka, redis            ├── jaeger (4317, 16686)
  ├── keycloak, vault                   ├── prometheus (9090)
  └── auth-rust, config-rust, ...       ├── loki (3100)
      │                                 └── grafana (3200)
      │
      └── OTEL_EXPORTER_OTLP_ENDPOINT=http://<shared-server>:4317
```

### アプリケーション設定

ローカルのサーバーから共用サーバーの Jaeger にトレースを送信するには、`config.dev.yaml` の OTel エンドポイントを変更する。

```yaml
# config.dev.yaml（ローカル PC）
otel:
  endpoint: "http://<shared-server-ip>:4317"   # 共用サーバーの Jaeger
```

### 使い分け

| 環境 | infra プロファイル | observability プロファイル | OTel エンドポイント |
| --- | --- | --- | --- |
| ローカル PC（フル） | ローカル | ローカル | `http://jaeger:4317` |
| ローカル PC（軽量） | ローカル | **起動しない** | `http://<shared-server>:4317` |
| 共用サーバー | サーバー | サーバー | `http://jaeger:4317` |

詳細は [共用開発サーバー設計](../devenv/共用開発サーバー設計.md) を参照。

---

---

## Grafana アクセスポートに関する注意（MED-002 対応）

Grafana のホストポートは **3200**（デフォルト）であり、一般的な 3000 ではない。

```
http://localhost:3200  ← 正しいアクセス先
http://localhost:3000  ← 誤り（フロントエンド開発サーバーと競合するため 3200 を使用）
```

**理由**: ポート 3000 は React / Next.js / Vite 等のフロントエンド開発サーバーが標準で使用する。同一マシンで両者を起動するとポート競合が発生するため、Grafana のホストポートを 3200 に設定している。

**変更方法**: `.env` に `GRAFANA_HOST_PORT=3000` を追記すればポート 3000 に変更できる（ただしフロントエンド開発サーバーとのポート競合に注意）。

```bash
# .env（任意）
GRAFANA_HOST_PORT=3000
```

ヘルスチェックコマンド（`http://localhost:3000/api/health`）はコンテナ**内部**のポート 3000 を参照するため変更不要。

---

## Alertmanager の未デプロイについて（LOW-005 対応）

現在のローカル開発環境では **Alertmanager はデプロイされていない**。

- `infra/docker/prometheus/alerting_rules.yaml` にアラートルールは定義されているが、発火しても通知先がない状態
- Prometheus コンソール（`http://localhost:9090/alerts`）でアラート状態の確認は可能
- 通知（メール・Slack・PagerDuty 等）は届かない

**本番環境（K8s）での対応**:
- Prometheus Operator を使用する場合: `AlertmanagerConfig` リソースで受信者を定義する
- 開発環境で通知テストが必要な場合: `docker compose` に Alertmanager を追加し `prometheus.yaml` の `alerting.alertmanagers` で接続設定を行う

```yaml
# prometheus.yaml への Alertmanager 接続追加例（必要な場合のみ）
alerting:
  alertmanagers:
    - static_configs:
        - targets: ["alertmanager:9093"]
      basic_auth:
        username: prometheus_admin
        password: prometheus_admin
```

---

## 関連ドキュメント

- [docker-compose設計.md](docker-compose設計.md) -- 基本方針・プロファイル設計
- [docker-compose-システムサービス設計.md](compose-システムサービス設計.md) -- auth-server・config-server・System プロファイル
- [docker-compose-インフラサービス設計.md](compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis・Kong の詳細設定
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [共用開発サーバー設計](../devenv/共用開発サーバー設計.md)
