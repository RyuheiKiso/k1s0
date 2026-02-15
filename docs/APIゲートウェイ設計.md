# API ゲートウェイ設計

Kong API Gateway の構成管理を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](tier-architecture.md) を参照。

## 基本方針

- API ゲートウェイは **Kong** を採用し、**DB-backed モード**（PostgreSQL）で運用する
- 管理は **Admin API** 経由で行い、decK で設定を宣言的に管理する
- CI/CD パイプラインから decK を実行し、設定変更をコードレビュー可能にする
- 認証・レート制限・ログ等の横断的関心事は Kong プラグインで一元管理する

---

## D-117: Kong 構成管理

### アーキテクチャ

```
Client → Ingress (Nginx) → Kong Proxy → Istio Sidecar → Backend Services
                                ↕
                          Kong Admin API
                                ↕
                          PostgreSQL (kong-db)
```

### DB-backed モード

| 項目             | 設定                                          |
| ---------------- | --------------------------------------------- |
| 動作モード       | DB-backed（PostgreSQL）                       |
| データベース     | PostgreSQL 15+                                |
| 接続先           | `postgres.k1s0-system.svc.cluster.local:5432` |
| データベース名   | `kong`                                        |
| レプリカ構成     | 読み取りレプリカなし（Admin API 経由の管理のみ） |

DB-backed モードの利点:
- Admin API による動的な設定変更が可能
- 複数 Kong インスタンス間で設定を自動共有
- decK によるバージョン管理・CI/CD 連携が容易

### Helm デプロイ

Kong は公式 Helm Chart（`kong/kong`）でデプロイする。

```yaml
# infra/helm/services/system/kong/values.yaml
image:
  repository: kong
  tag: "3.7"

env:
  database: postgres
  pg_host: postgres.k1s0-system.svc.cluster.local
  pg_port: "5432"
  pg_user: kong
  pg_database: kong
  pg_password:
    valueFrom:
      secretKeyRef:
        name: kong-db-secret
        key: password

proxy:
  enabled: true
  type: ClusterIP
  http:
    enabled: true
    containerPort: 8000
  tls:
    enabled: true
    containerPort: 8443

admin:
  enabled: true
  type: ClusterIP
  http:
    enabled: true
    containerPort: 8001
  tls:
    enabled: false

ingressController:
  enabled: false    # Ingress Controller は使わず Admin API で管理

postgresql:
  enabled: false    # 外部 PostgreSQL を使用

replicaCount: 2

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi
```

#### 環境別オーバーライド

```yaml
# values-dev.yaml
replicaCount: 1
resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi

# values-prod.yaml
replicaCount: 3
resources:
  requests:
    cpu: 1000m
    memory: 1Gi
  limits:
    cpu: 4000m
    memory: 4Gi
```

### Admin API による管理

Kong の設定は Admin API を通じて管理する。直接の Admin API 呼び出しは運用時のデバッグに限定し、通常の設定変更は decK 経由で行う。

```bash
# Service の作成例
curl -X POST http://kong-admin:8001/services \
  -d name=order-v1 \
  -d url=http://order-server.k1s0-service.svc.cluster.local:80

# Route の作成例
curl -X POST http://kong-admin:8001/services/order-v1/routes \
  -d name=order-v1-route \
  -d 'paths[]=/api/v1/orders' \
  -d strip_path=false
```

### プラグイン一覧と設定

| プラグイン           | 適用範囲   | 目的                          |
| -------------------- | ---------- | ----------------------------- |
| rate-limiting        | グローバル | レート制限                    |
| jwt                  | グローバル | JWT 認証                      |
| cors                 | グローバル | CORS 制御                     |
| request-transformer  | サービス別 | リクエストヘッダー変換        |
| response-transformer | サービス別 | レスポンスヘッダー付与        |
| prometheus           | グローバル | メトリクス収集                |
| file-log             | グローバル | アクセスログ出力              |
| ip-restriction       | サービス別 | IP 制限（Admin API 保護等）   |

#### JWT プラグイン

```yaml
plugins:
  - name: jwt
    config:
      uri_param_names: []
      cookie_names: []
      key_claim_name: iss
      claims_to_verify:
        - exp
```

#### CORS プラグイン

```yaml
plugins:
  - name: cors
    config:
      origins:
        - "https://*.k1s0.internal.example.com"
      methods:
        - GET
        - POST
        - PUT
        - PATCH
        - DELETE
        - OPTIONS
      headers:
        - Authorization
        - Content-Type
        - X-Request-ID
      exposed_headers:
        - X-RateLimit-Limit
        - X-RateLimit-Remaining
        - X-RateLimit-Reset
      max_age: 3600
      credentials: true
```

#### Prometheus プラグイン

```yaml
plugins:
  - name: prometheus
    config:
      per_consumer: true
      status_code_metrics: true
      latency_metrics: true
      bandwidth_metrics: true
```

### decK による宣言的設定管理

Kong の設定を YAML ファイルで宣言的に管理し、Git でバージョン管理する。

#### ディレクトリ構成

```
infra/kong/
├── kong.yaml          # メイン設定ファイル（decK）
├── plugins/
│   ├── global.yaml    # グローバルプラグイン
│   └── auth.yaml      # 認証プラグイン
└── services/
    ├── system.yaml    # system Tier のサービス定義
    ├── business.yaml  # business Tier のサービス定義
    └── service.yaml   # service Tier のサービス定義
```

#### kong.yaml の例

```yaml
# infra/kong/kong.yaml
_format_version: "3.0"

services:
  # system Tier
  - name: auth-v1
    url: http://auth-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: auth-v1-route
        paths:
          - /api/v1/auth
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 30
          policy: redis
          redis_host: redis.messaging.svc.cluster.local

  # service Tier
  - name: order-v1
    url: http://order-server.k1s0-service.svc.cluster.local:80
    routes:
      - name: order-v1-route
        paths:
          - /api/v1/orders
        strip_path: false

plugins:
  # グローバルプラグイン
  - name: rate-limiting
    config:
      minute: 500
      policy: redis
      redis_host: redis.messaging.svc.cluster.local
      redis_port: 6379
      redis_database: 1
      fault_tolerant: true
      hide_client_headers: false

  - name: jwt
    config:
      key_claim_name: iss
      claims_to_verify:
        - exp

  - name: cors
    config:
      origins:
        - "https://*.k1s0.internal.example.com"
      credentials: true

  - name: prometheus
    config:
      per_consumer: true
      status_code_metrics: true
```

### CI/CD 連携（decK）

```yaml
# .github/workflows/kong-sync.yaml
name: Kong Config Sync

on:
  push:
    branches: [main]
    paths:
      - 'infra/kong/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install decK
        run: |
          curl -sL https://github.com/Kong/deck/releases/latest/download/deck_linux_amd64.tar.gz | tar xz
          sudo mv deck /usr/local/bin/
      - name: Validate config
        run: deck validate -s infra/kong/kong.yaml

  diff:
    needs: validate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Show diff
        run: |
          deck diff -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-dev:
    needs: diff
    runs-on: ubuntu-latest
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - name: Sync to dev
        run: |
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-staging:
    needs: sync-dev
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - name: Sync to staging
        run: |
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-prod:
    needs: sync-staging
    runs-on: ubuntu-latest
    environment:
      name: prod
    steps:
      - uses: actions/checkout@v4
      - name: Sync to prod
        run: |
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001
```

### 環境別構成

| 項目               | dev              | staging          | prod             |
| ------------------ | ---------------- | ---------------- | ---------------- |
| Kong レプリカ      | 1                | 2                | 3                |
| PostgreSQL         | 単一インスタンス | 単一インスタンス | HA 構成          |
| Rate Limiting 倍率 | x10              | x2               | x1               |
| Admin API アクセス | 開発者全員       | 運用チーム       | インフラチームのみ |
| decK 自動 sync     | 自動             | 自動             | 手動承認         |

---

## 関連ドキュメント

- [tier-architecture.md](tier-architecture.md) — Tier アーキテクチャの詳細
- [API設計.md](API設計.md) — REST API・gRPC・GraphQL・レート制限設計
- [認証認可設計.md](認証認可設計.md) — 認証・認可・Kong 認証フロー
- [kubernetes設計.md](kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [サービスメッシュ設計.md](サービスメッシュ設計.md) — Istio 設計・耐障害性
- [CI-CD設計.md](CI-CD設計.md) — CI/CD パイプライン設計
- [可観測性設計.md](可観測性設計.md) — 監視・ログ・トレース設計
