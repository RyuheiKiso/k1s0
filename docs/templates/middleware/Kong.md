# テンプレート仕様 — Kong

## 概要

k1s0 CLI ひな形生成のKongテンプレート仕様。サービス単位の Kong Service/Route 定義およびプラグイン設定（レート制限、認証、ロギング）を、サービスの `tier` と `api_styles` に応じて自動生成する。

API ゲートウェイ設計の全体像は [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) を参照。

## 生成対象

| kind       | kong-service | kong-plugins |
| ---------- | ------------ | ------------ |
| `server`   | 生成する     | 生成する     |
| `bff`      | 生成する     | 生成する     |

## 配置パス

生成されるリソースファイルは `infra/kong/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル       | 配置パス                                           |
| -------------- | -------------------------------------------------- |
| Kong Service   | `infra/kong/{{ service_name }}/kong-service.yaml`   |
| Kong Plugins   | `infra/kong/{{ service_name }}/kong-plugins.yaml`   |

## テンプレートファイル一覧

テンプレートは `CLI/templates/kong/` 配下に配置する。

| テンプレートファイル           | 生成先                                             | 説明                                 |
| ------------------------------ | -------------------------------------------------- | ------------------------------------ |
| `kong-service.yaml.tera`       | `infra/kong/{{ service_name }}/kong-service.yaml`   | Kong Service/Route 定義              |
| `kong-plugins.yaml.tera`       | `infra/kong/{{ service_name }}/kong-plugins.yaml`   | Kong プラグイン設定                  |

### ディレクトリ構成

```
CLI/
└── templates/
    └── kong/
        ├── kong-service.yaml.tera
        └── kong-plugins.yaml.tera
```

## 使用するテンプレート変数

Kong テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型       | kong-service | kong-plugins | 用途                                         |
| -------------------- | -------- | ------------ | ------------ | -------------------------------------------- |
| `service_name`       | String   | 用           | 用           | Service 名、Route 名、ホスト名              |
| `service_name_snake` | String   | 用           | 用           | 設定キーのプレフィクス                       |
| `namespace`          | String   | 用           | -            | Kubernetes Service のホスト解決              |
| `tier`               | String   | -            | 用           | Tier 別レート制限値の決定                    |
| `server_port`        | Number   | 用           | -            | HTTP ポート番号                              |
| `grpc_port`          | Number   | 用           | -            | gRPC ポート番号（gRPC 使用時）               |
| `api_styles`         | [String] | 用           | 用           | REST/gRPC/GraphQL に応じたプロトコル設定     |

### Tier 別レート制限

[APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) および [REST-API設計](../../architecture/api/REST-API設計.md) のTier 別デフォルト値に準拠する。

| Tier       | minute | second |
| ---------- | ------ | ------ |
| `system`   | 3000   | 100    |
| `business` | 1000   | 40     |
| `service`  | 500    | 20     |

---

## Kong Service/Route テンプレート（kong-service.yaml.tera）

Kong の Service と Route を定義する。API 方式に応じて HTTP/gRPC の両方のルートを生成する。

```tera
apiVersion: configuration.konghq.com/v1
kind: KongIngress
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
  labels:
    app: {{ service_name }}
    tier: {{ tier }}
proxy:
  protocol: http
  path: /
  connect_timeout: 10000
  write_timeout: 60000
  read_timeout: 60000
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
  annotations:
    konghq.com/override: {{ service_name }}
    konghq.com/strip-path: "false"
  labels:
    app: {{ service_name }}
    tier: {{ tier }}
{% if api_styles is containing("grpc") %}
    konghq.com/protocols: "http,https,grpc,grpcs"
{% else %}
    konghq.com/protocols: "http,https"
{% endif %}
spec:
  ingressClassName: kong
  rules:
{% if api_styles is containing("rest") or api_styles is containing("graphql") %}
    - host: {{ service_name }}.{{ namespace }}.svc.cluster.local
      http:
        paths:
          - path: /api/{{ service_name_snake }}
            pathType: Prefix
            backend:
              service:
                name: {{ service_name }}
                port:
                  number: {{ server_port }}
{% endif %}
{% if api_styles is containing("grpc") %}
    - host: {{ service_name }}.{{ namespace }}.svc.cluster.local
      http:
        paths:
          - path: /{{ service_name_snake }}.
            pathType: Prefix
            backend:
              service:
                name: {{ service_name }}
                port:
                  number: {{ grpc_port }}
{% endif %}
```

### ポイント

- `KongIngress` でプロキシのタイムアウト設定を定義する
- REST/GraphQL 使用時は `/api/{{ service_name_snake }}` パスでルーティングする
- gRPC 使用時は `/{{ service_name_snake }}.` パスプレフィクスで gRPC サービスにルーティングする
- `konghq.com/strip-path: "false"` でバックエンドにバージョン付きパスをそのまま転送する（[REST-API設計](../../architecture/api/REST-API設計.md) の Kong ルーティング連携参照）
- gRPC 使用時は `konghq.com/protocols` に `grpc,grpcs` を追加する

---

## Kong Plugins テンプレート（kong-plugins.yaml.tera）

レート制限、CORS、JWT 認証の3種類のプラグインを定義する。設定値は [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) のプラグイン一覧と整合する。

```tera
# Rate Limiting Plugin
apiVersion: configuration.konghq.com/v1
kind: KongPlugin
metadata:
  name: {{ service_name }}-rate-limit
  namespace: {{ namespace }}
  labels:
    app: {{ service_name }}
    tier: {{ tier }}
plugin: rate-limiting
config:
{% if tier == "system" %}
  minute: 3000
  second: 100
{% elif tier == "business" %}
  minute: 1000
  second: 40
{% else %}
  minute: 500
  second: 20
{% endif %}
  policy: redis
  redis_host: redis.k1s0-system.svc.cluster.local
  redis_port: 6379
  redis_database: 1
  fault_tolerant: true
  hide_client_headers: false
---
# CORS Plugin
apiVersion: configuration.konghq.com/v1
kind: KongPlugin
metadata:
  name: {{ service_name }}-cors
  namespace: {{ namespace }}
  labels:
    app: {{ service_name }}
    tier: {{ tier }}
plugin: cors
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
  credentials: true
  max_age: 3600
---
# JWT Plugin
apiVersion: configuration.konghq.com/v1
kind: KongPlugin
metadata:
  name: {{ service_name }}-jwt
  namespace: {{ namespace }}
  labels:
    app: {{ service_name }}
    tier: {{ tier }}
plugin: jwt
config:
  uri_param_names: []
  cookie_names: []
  key_claim_name: kid
  claims_to_verify:
    - exp
  maximum_expiration: 900
  header_names:
    - Authorization
```

### ポイント

- **Rate Limiting**: Tier 別のレート制限（minute + second）を Redis ポリシーで適用する。`fault_tolerant: true` で Redis 障害時もリクエストを許可する。制限値は [REST-API設計](../../architecture/api/REST-API設計.md) の Tier 別デフォルト値に準拠する
- **CORS**: ワイルドカードオリジン `*.k1s0.internal.example.com` で全サブドメインを許可する。`exposed_headers` でレート制限ヘッダーをクライアントに公開する
- **JWT**: `key_claim_name: kid`（Key ID）で JWT 署名鍵を特定する。`maximum_expiration: 900`（15分）で Access Token のライフタイムを制限する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                    | 選択肢                              | 生成への影響                                              |
| ----------------------- | ----------------------------------- | --------------------------------------------------------- |
| Tier (`tier`)           | `system` / `business` / `service`   | Rate Limiting のレート制限値                               |
| API 方式 (`api_styles`) | `rest` / `graphql` を含む           | HTTP ルートを生成                                         |
| API 方式 (`api_styles`) | `grpc` を含む                       | gRPC ルートを追加、プロトコルに grpc/grpcs を追加         |
| kind (`kind`)           | `server` / `bff` 以外              | Kong リソースを生成しない                                 |

---

## 生成例

### system Tier の REST + gRPC サーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "service_name_snake": "auth_service",
  "tier": "system",
  "namespace": "k1s0-system",
  "api_styles": ["rest", "grpc"],
  "server_port": 80,
  "grpc_port": 50051
}
```

生成されるファイル:
- `infra/kong/auth-service/kong-service.yaml` -- REST + gRPC ルート、grpc/grpcs プロトコル
- `infra/kong/auth-service/kong-plugins.yaml` -- rate=3000/min + 100/sec、CORS、JWT

### service Tier の REST サーバーの場合

入力:
```json
{
  "service_name": "order-server",
  "service_name_snake": "order_server",
  "tier": "service",
  "namespace": "k1s0-service",
  "api_styles": ["rest"],
  "server_port": 80
}
```

生成されるファイル:
- `infra/kong/order-server/kong-service.yaml` -- REST ルートのみ
- `infra/kong/order-server/kong-plugins.yaml` -- rate=500/min + 20/sec、CORS、JWT

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) -- Kong API Gateway の全体設計
- [認証認可設計](../../architecture/auth/認証認可設計.md) -- 認証・認可の詳細設計
- [テンプレート仕様-Keycloak](Keycloak.md) -- Keycloak テンプレート仕様
- [テンプレート仕様-ServiceMesh](ServiceMesh.md) -- ServiceMesh テンプレート仕様
- [テンプレート仕様-Helm](../infrastructure/Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-Observability](../observability/Observability.md) -- Observability テンプレート仕様
