# API ゲートウェイ設計

Kong API Gateway の構成管理を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](../../architecture/overview/tier-architecture.md) を参照。

> **注記: ドメイン名について**
> 本ドキュメント内で使用している `example.com`（例: `*.k1s0.internal.example.com`）はプレースホルダーであり、実際のドメインではない。本番環境へのデプロイ時には、以下のフローで実ドメインへ置換すること。
>
> 1. `infra/kong/` 配下の設定ファイルでは、環境変数またはHelm values の環境別オーバーライドでドメインを注入する
> 2. CI/CD パイプラインの各環境ステージ（dev / staging / prod）で、環境固有のドメイン値を `KONG_CORS_ORIGINS` 等の変数から設定する
> 3. decK の設定ファイルにはプレースホルダーを残し、`deck sync` 実行前に `envsubst` 等で置換する

---

## D-117: Kong 構成管理

### アーキテクチャ

```
Client → Nginx Ingress Controller (TLS終端) → Kong Proxy → Istio Sidecar (mTLS) → Backend Services
                                ↕
                          Kong Admin API
                                ↕
                          PostgreSQL (kong-db)
```

#### BFF Proxy 経由のトラフィックフロー

SPA（React）からのアクセスは BFF Proxy を経由し、HttpOnly Cookie と Bearer Token を変換する（詳細は [認証認可設計](../auth/認証認可設計.md) の「SPA トークン保存方式」参照）。

```
Browser → [HttpOnly Cookie] → Nginx Ingress Controller → Kong → BFF Proxy → [Bearer Token] → Istio Sidecar (mTLS) → Backend Services
```

### DB-backed モード

| 項目             | 設定                                          |
| ---------------- | --------------------------------------------- |
| 動作モード       | DB-backed（PostgreSQL）                       |
| データベース     | PostgreSQL 17（Kong 3.8 は PostgreSQL 15+ 対応） |
| 接続先           | `postgres.k1s0-system.svc.cluster.local:5432` |
| データベース名   | `kong`                                        |
| レプリカ構成     | 読み取りレプリカなし（Admin API 経由の管理のみ） |

### Helm デプロイ

Kong は公式 Helm Chart（`kong/kong`）でデプロイする。

```yaml
# infra/helm/services/system/kong/values.yaml
image:
  repository: kong
  tag: "3.8"

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
    enabled: false                  # TLS は Nginx Ingress Controller で終端するため Kong 側では無効化
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

replicaCount: 2                   # ベース値（staging デフォルト。dev/prod は環境別 values で上書き）

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
| post-function        | グローバル | ユーザー情報（claims）をバックエンドへのリクエストヘッダーに転送（[認証認可設計](../auth/認証認可設計.md) の「ヘッダー転送」参照） |

#### JWT プラグイン

```yaml
plugins:
  - name: jwt
    config:
      uri_param_names: []
      cookie_names: []
      key_claim_name: kid
      claims_to_verify:
        - exp
      maximum_expiration: 900           # 15分（Access Token のライフタイム）
      header_names:
        - Authorization
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

> **環境別 CORS origins**: 上記は prod 環境用の設定。環境別 values ファイルで origins を上書きする。
> - dev: `["http://localhost:3000", "http://localhost:5173"]`
> - staging: `["https://*.staging.k1s0.internal.example.com"]`
> - prod: `["https://*.k1s0.internal.example.com"]`

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

### decK ディレクトリ構成

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

### 環境別構成

| 項目               | dev              | staging          | prod             |
| ------------------ | ---------------- | ---------------- | ---------------- |
| Kong レプリカ      | 1                | 2                | 3                |
| PostgreSQL         | シングルノード | 2ノード（Primary 1 + Replica 1） | 3ノード HA 構成（Bitnami PostgreSQL HA Chart: Primary 1 + Replica 2） |
| Rate Limiting 倍率 | x10              | x2               | x1               |
| Admin API アクセス | Basic認証 + 開発用トークン | IP制限 + mTLS（運用チーム） | IP制限 + mTLS + 監査ログ（インフラチーム個人証明書） |
| decK 自動 sync     | 自動             | 自動             | 手動承認         |

---

## 基本方針

- API ゲートウェイは **Kong** を採用し、**DB-backed モード**（PostgreSQL）で運用する
- 管理は **Admin API** 経由で行い、decK で設定を宣言的に管理する
- CI/CD パイプラインから decK を実行し、設定変更をコードレビュー可能にする
- 認証・レート制限・ログ等の横断的関心事は Kong プラグインで一元管理する

### DB-backed モードの利点

- Admin API による動的な設定変更が可能
- 複数 Kong インスタンス間で設定を自動共有
- decK によるバージョン管理・CI/CD 連携が容易

---

## Admin API による管理

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

---

## decK による宣言的設定管理

Kong の設定を YAML ファイルで宣言的に管理し、Git でバージョン管理する。

### kong.yaml の例

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
      - name: auth-v1-login
        paths:
          - /api/v1/auth/login
        strip_path: false
        plugins:
          - name: rate-limiting
            config:
              minute: 30                  # ブルートフォース防止（API設計.md 参照）
              policy: redis
              redis_host: redis.k1s0-system.svc.cluster.local

  - name: saga-v1
    url: http://saga-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: saga-v1-sagas
        paths:
          - /api/v1/sagas
        strip_path: false
        methods: [GET, POST]
      - name: saga-v1-workflows
        paths:
          - /api/v1/workflows
        strip_path: false
        methods: [GET, POST]
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Service-Name:saga-server

  - name: dlq-manager-v1
    url: http://dlq-manager.k1s0-system.svc.cluster.local:80
    routes:
      - name: dlq-manager-v1-route
        paths:
          - /api/v1/dlq
        strip_path: false
        methods: [GET, POST, DELETE]
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Service-Name:dlq-manager

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
  # グローバルレート制限: service Tier のデフォルト値。
  # system Tier および business Tier のサービスには、個別のルート/サービスレベルで上書き設定する。
  - name: rate-limiting
    config:
      minute: 500
      policy: redis
      redis_host: redis.k1s0-system.svc.cluster.local
      redis_port: 6379
      redis_database: 1
      fault_tolerant: true
      hide_client_headers: false

  - name: jwt
    config:
      key_claim_name: kid
      claims_to_verify:
        - exp
      maximum_expiration: 900     # 15分（Access Token のライフタイム）

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

### Tier 別レート制限オーバーライド例

グローバルレート制限（`minute: 500`）は service Tier のデフォルト値である。system Tier および business Tier のサービスには、ルート/サービスレベルで個別に上書き設定する。Tier 別デフォルト値の詳細は [REST-API設計.md](REST-API設計.md) の「Tier 別デフォルト値」を参照。

| Tier     | minute | second | 説明                               |
| -------- | ------ | ------ | ---------------------------------- |
| system   | 3000   | 100    | 内部基盤サービス（高頻度呼び出し） |
| business | 1000   | 40     | 領域共通サービス                   |
| service  | 500    | 20     | 個別業務サービス（グローバルデフォルト） |

```yaml
# system Tier のオーバーライド例（auth-server）
services:
  - name: auth-v1
    plugins:
      - name: rate-limiting
        config:
          minute: 3000            # system Tier: 高スループット要件（REST-API設計.md 参照）
          second: 100             # 秒あたりの上限（バースト制御）
          policy: redis
          redis_host: redis.k1s0-system.svc.cluster.local

# business Tier のオーバーライド例（accounting-ledger）
services:
  - name: accounting-ledger-v1
    plugins:
      - name: rate-limiting
        config:
          minute: 1000            # business Tier: 中程度のスループット（REST-API設計.md 参照）
          second: 40              # 秒あたりの上限（バースト制御）
          policy: redis
          redis_host: redis.k1s0-system.svc.cluster.local
```

---

## CI/CD 連携（decK）

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

  # NOTE: 各環境の CI/CD ランナーはそれぞれのクラスタ内で動作する。
  # そのため Kong Admin API のサービス名（kong-admin.k1s0-system.svc.cluster.local:8001）は
  # 全環境で同一だが、実際の接続先はランナーが属するクラスタコンテキストによって異なる。
  # dev クラスタのランナー → dev の Kong、staging クラスタのランナー → staging の Kong、
  # prod クラスタのランナー → prod の Kong にそれぞれ接続される。

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
    runs-on: [self-hosted, dev]
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - name: Sync to dev
        run: |
          # dev クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-staging:
    needs: sync-dev
    runs-on: [self-hosted, staging]
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - name: Sync to staging
        run: |
          # staging クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-prod:
    needs: sync-staging
    runs-on: [self-hosted, prod]
    environment:
      name: prod
    steps:
      - uses: actions/checkout@v4
      - name: Sync to prod
        run: |
          # prod クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001
```

---

## PostgreSQL HA 構成詳細

### prod 環境（3ノード構成）

- Primary 1 ノード + Replica 2 ノードの合計 3 ノード構成
- **Bitnami PostgreSQL HA Chart** によるストリーミングレプリケーションとフェイルオーバー管理
  - Primary 障害時に Replica の中から自動的に新しい Primary を選出
  - フェイルオーバー時間目標: 30 秒以内
- **同期レプリケーション**を採用し、データ損失を防止
  - `synchronous_commit = on` により、少なくとも 1 つの Replica への書き込み完了を保証
  - `synchronous_standby_names = 'ANY 1 (*)'` で任意の 1 Replica を同期対象とする
- Kong からの接続は Kubernetes Service 経由でルーティング
- PostgreSQL のデプロイは [terraform設計.md](../../infrastructure/terraform/terraform設計.md) の `modules/database/` で管理する

### staging 環境（2ノード構成）

- Primary 1 ノード + Replica 1 ノードの合計 2 ノード構成
- **非同期レプリケーション**を採用（パフォーマンス優先）
  - `synchronous_commit = off`
- フェイルオーバーのテスト用途を兼ねる

### dev 環境（シングルノード）

- PostgreSQL シングルノード構成
- レプリケーションなし
- 開発・テスト用途のため可用性要件は設けない

---

## Admin API アクセス制御

環境ごとに異なるアクセス制御を適用し、セキュリティレベルを段階的に強化する。

### dev 環境

- **Basic 認証** + 開発用トークンによるアクセス制御
- 開発者全員がアクセス可能
- 開発用トークンは `kong-admin-dev-token` Secret で管理

### staging 環境

- **IP 制限**: 管理ネットワーク（`10.0.0.0/8` 等の社内ネットワーク）からのアクセスのみ許可
- **mTLS**: クライアント証明書による相互認証を必須とする
  - 運用チーム用のクライアント証明書を発行し、Kong Admin API への接続時に提示
- Kong の `ip-restriction` プラグインと Istio の PeerAuthentication を組み合わせて適用

### prod 環境

- **IP 制限**: 管理ネットワークからのアクセスのみ許可（staging と同様）
- **mTLS**: インフラチームメンバー個人に発行されたクライアント証明書による認証
  - 個人証明書は社内 CA から発行し、有効期限 1 年、失効管理は CRL で実施
  - 証明書の CN にはメンバーの識別子を含め、誰がアクセスしたかを特定可能にする
- **監査ログ記録**: Admin API への全リクエストを監査ログとして記録
  - 記録項目: タイムスタンプ、操作者（証明書 CN）、HTTPメソッド、エンドポイント、リクエストボディ、レスポンスコード
  - ログは Loki に送信し、1 年間保持（`retention_period: 8760h`、監査ログ保持ポリシーに準拠）
  - 設定変更操作（POST / PUT / PATCH / DELETE）は Microsoft Teams の `infra-audit` チャンネルにもリアルタイム通知

---

## 関連ドキュメント

- [tier-architecture.md](../../architecture/overview/tier-architecture.md) — Tier アーキテクチャの詳細
- [API設計.md](API設計.md) — REST API・gRPC・GraphQL・レート制限設計
- [認証認可設計.md](../auth/認証認可設計.md) — 認証・認可・Kong 認証フロー
- [kubernetes設計.md](../../infrastructure/kubernetes/kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [サービスメッシュ設計.md](../../infrastructure/service-mesh/サービスメッシュ設計.md) — Istio 設計・耐障害性
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md) — CI/CD パイプライン設計
- [可観測性設計.md](../observability/可観測性設計.md) — 監視・ログ・トレース設計
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) — Helm Chart と values 設計
- [terraform設計.md](../../infrastructure/terraform/terraform設計.md) — Terraform モジュール設計
- [インフラ設計.md](../../infrastructure/overview/インフラ設計.md) — オンプレミスインフラ全体構成
