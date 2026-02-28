# API ゲートウェイ設計

> **ガイド**: 設計背景・選定理由は [APIゲートウェイ設計.guide.md](./APIゲートウェイ設計.guide.md) を参照。

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
| データベース     | PostgreSQL 17（Kong 3.7 は PostgreSQL 15+ 対応） |
| 接続先           | `postgres.k1s0-system.svc.cluster.local:5432` |
| データベース名   | `kong`                                        |
| レプリカ構成     | 読み取りレプリカなし（Admin API 経由の管理のみ） |

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
