# テンプレート仕様 — ServiceMesh

## 概要

k1s0 CLI ひな形生成のServiceMeshテンプレート仕様。VirtualService、DestinationRule、PeerAuthentication、AuthorizationPolicy、NetworkPolicy の5種類のリソースを、サービスの `tier` と `api_styles` に応じて自動生成する。

サービスメッシュ設計の全体像は [サービスメッシュ設計](../../infrastructure/service-mesh/サービスメッシュ設計.md) を参照。

## 生成対象

| kind       | VirtualService | DestinationRule | PeerAuthentication | AuthorizationPolicy | NetworkPolicy |
| ---------- | -------------- | --------------- | ------------------ | ------------------- | ------------- |
| `server`   | 生成する       | 生成する        | 生成する           | 生成する            | 生成する      |
| `bff`      | 生成する       | 生成する        | 生成する           | 生成する            | 生成する      |

- **VirtualService**: ルーティング、timeout、retry、カナリアリリースの設定
- **DestinationRule**: サブセット定義、ロードバランシング、Circuit Breaker の設定
- **PeerAuthentication**: mTLS の強制設定
- **AuthorizationPolicy**: Tier 間アクセス制御の設定
- **NetworkPolicy**: L3/L4 レベルの Namespace 間アクセス制御（Ingress/Egress）

## 配置パス

生成されるリソースファイルは `infra/service-mesh/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル              | 配置パス                                                          |
| --------------------- | ----------------------------------------------------------------- |
| VirtualService        | `infra/service-mesh/{{ service_name }}/virtual-service.yaml`      |
| DestinationRule       | `infra/service-mesh/{{ service_name }}/destination-rule.yaml`     |
| PeerAuthentication    | `infra/service-mesh/{{ service_name }}/peer-authentication.yaml`  |
| AuthorizationPolicy   | `infra/service-mesh/{{ service_name }}/authorization-policy.yaml` |
| NetworkPolicy         | `infra/service-mesh/{{ service_name }}/network-policy.yaml`       |

## テンプレートファイル一覧

テンプレートは `CLI/templates/service-mesh/` 配下に配置する。

| テンプレートファイル                | 生成先                                                          | 説明                                      |
| ----------------------------------- | --------------------------------------------------------------- | ----------------------------------------- |
| `virtual-service.yaml.tera`        | `infra/service-mesh/{{ service_name }}/virtual-service.yaml`      | VirtualService（ルーティング・retry）     |
| `destination-rule.yaml.tera`       | `infra/service-mesh/{{ service_name }}/destination-rule.yaml`     | DestinationRule（LB・Circuit Breaker）    |
| `peer-authentication.yaml.tera`    | `infra/service-mesh/{{ service_name }}/peer-authentication.yaml`  | PeerAuthentication（mTLS）                |
| `authorization-policy.yaml.tera`   | `infra/service-mesh/{{ service_name }}/authorization-policy.yaml` | AuthorizationPolicy（Tier 間アクセス制御）|
| `network-policy.yaml.tera`         | `infra/service-mesh/{{ service_name }}/network-policy.yaml`       | NetworkPolicy（L3/L4 アクセス制御）       |

### ディレクトリ構成

```
CLI/
└── templates/
    └── service-mesh/
        ├── virtual-service.yaml.tera
        ├── destination-rule.yaml.tera
        ├── peer-authentication.yaml.tera
        ├── authorization-policy.yaml.tera
        └── network-policy.yaml.tera
```

## 使用するテンプレート変数

ServiceMesh テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名           | 型       | VirtualService | DestinationRule | PeerAuthentication | AuthorizationPolicy | NetworkPolicy | 用途                                          |
| ---------------- | -------- | -------------- | --------------- | ------------------ | ------------------- | ------------- | --------------------------------------------- |
| `service_name`   | String   | 用             | 用              | 用                 | 用                  | 用            | リソース名、host 名                           |
| `tier`           | String   | 用             | 用              | 用                 | 用                  | 用            | Namespace 導出、Tier 別デフォルト値の決定     |
| `namespace`      | String   | 用             | 用              | 用                 | 用                  | 用            | リソースの配置先 Namespace                    |
| `api_styles`     | [String] | 用             | 用              | -                  | -                   | 用            | gRPC 時の H2 アップグレード / ポート開放      |
| `server_port`    | Number   | 用             | -               | -                  | -                   | 用            | HTTP ポート番号                               |
| `grpc_port`      | Number   | 用             | -               | -                  | -                   | 用            | gRPC ポート番号（gRPC 使用時）                |
| `kind`           | String   | -              | -               | -                  | 用                  | -             | BFF 判定（deny-bff-to-bff ポリシー適用）      |

### Namespace の導出

`namespace` は `tier` から以下のルールで導出する。

| tier       | namespace        |
| ---------- | ---------------- |
| `system`   | `k1s0-system`    |
| `business` | `k1s0-business`  |
| `service`  | `k1s0-service`   |

### Tier 別デフォルト値

テンプレート内で `tier` に応じて以下のデフォルト値を適用する。

#### Timeout / Retry

| Tier       | timeout | retry attempts | perTryTimeout | retryOn                                    |
| ---------- | ------- | -------------- | ------------- | ------------------------------------------ |
| `system`   | 5s      | 3              | 2s            | `5xx,reset,connect-failure`                |
| `business` | 10s     | 3              | 3s            | `5xx,reset,connect-failure`                |
| `service`  | 15s     | 2              | 5s            | `5xx,reset,connect-failure,retriable-4xx`  |

#### Circuit Breaker（outlierDetection）

| 設定                   | system | business | service |
| ---------------------- | ------ | -------- | ------- |
| consecutive5xxErrors   | 3      | 5        | 5       |
| interval               | 10s    | 30s      | 30s     |
| baseEjectionTime       | 30s    | 30s      | 60s     |
| maxEjectionPercent     | 30     | 50       | 50      |

#### connectionPool

| 設定                       | system | business | service |
| -------------------------- | ------ | -------- | ------- |
| tcp.maxConnections         | 200    | 100      | 100     |
| http.http1MaxPendingRequests | 200  | 100      | 100     |
| http.http2MaxRequests      | 2000   | 1000     | 1000    |
| http.maxRequestsPerConnection | 20  | 10       | 10      |

---

## VirtualService テンプレート（virtual-service.yaml.tera）

基本ルーティングとカナリアリリース用のサブセットルーティングを定義する。timeout と retry は Tier 別デフォルト値を適用する。

```tera
apiVersion: networking.istio.io/v1
kind: VirtualService
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
spec:
  hosts:
    - {{ service_name }}
  http:
    - route:
        - destination:
            host: {{ service_name }}
            port:
              number: {{ server_port }}
            subset: stable
          weight: 100
        - destination:
            host: {{ service_name }}
            port:
              number: {{ server_port }}
            subset: canary
          weight: 0
{% if tier == "system" %}
      timeout: 5s
      retries:
        attempts: 3
        perTryTimeout: 2s
        retryOn: "5xx,reset,connect-failure"
{% elif tier == "business" %}
      timeout: 10s
      retries:
        attempts: 3
        perTryTimeout: 3s
        retryOn: "5xx,reset,connect-failure"
{% elif tier == "service" %}
      timeout: 15s
      retries:
        attempts: 2
        perTryTimeout: 5s
        retryOn: "5xx,reset,connect-failure,retriable-4xx"
{% endif %}
{% if api_styles is containing("grpc") %}
    - match:
        - port: {{ grpc_port }}
      route:
        - destination:
            host: {{ service_name }}
            port:
              number: {{ grpc_port }}
            subset: stable
          weight: 100
        - destination:
            host: {{ service_name }}
            port:
              number: {{ grpc_port }}
            subset: canary
          weight: 0
{% if tier == "system" %}
      timeout: 5s
      retries:
        attempts: 3
        perTryTimeout: 2s
        retryOn: "cancelled,deadline-exceeded,internal,resource-exhausted,unavailable"
{% elif tier == "business" %}
      timeout: 10s
      retries:
        attempts: 3
        perTryTimeout: 3s
        retryOn: "cancelled,deadline-exceeded,internal,resource-exhausted,unavailable"
{% elif tier == "service" %}
      timeout: 15s
      retries:
        attempts: 2
        perTryTimeout: 5s
        retryOn: "cancelled,deadline-exceeded,internal,resource-exhausted,unavailable"
{% endif %}
{% endif %}
```

### ポイント

- カナリアリリースのウェイトは初期状態で `stable: 100` / `canary: 0` とし、Flagger が段階的に変更する
- gRPC 使用時は gRPC 固有の `retryOn` 条件（`cancelled,deadline-exceeded,internal,resource-exhausted,unavailable`）を適用する
- timeout / retry の値はサービス固有の要件に応じて生成後に上書き可能

---

## DestinationRule テンプレート（destination-rule.yaml.tera）

サブセット定義、ロードバランシングポリシー、connectionPool、Circuit Breaker（outlierDetection）を設定する。

```tera
apiVersion: networking.istio.io/v1
kind: DestinationRule
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
spec:
  host: {{ service_name }}
  trafficPolicy:
    connectionPool:
      tcp:
{% if tier == "system" %}
        maxConnections: 200
{% else %}
        maxConnections: 100
{% endif %}
      http:
{% if api_styles is containing("grpc") %}
        h2UpgradePolicy: UPGRADE
{% endif %}
{% if tier == "system" %}
        http1MaxPendingRequests: 200
        http2MaxRequests: 2000
        maxRequestsPerConnection: 20
{% else %}
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
{% endif %}
    loadBalancer:
      simple: LEAST_REQUEST
    outlierDetection:
{% if tier == "system" %}
      consecutive5xxErrors: 3
      interval: 10s
      baseEjectionTime: 30s
      maxEjectionPercent: 30
{% elif tier == "business" %}
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
{% elif tier == "service" %}
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 60s
      maxEjectionPercent: 50
{% endif %}
    tls:
      mode: ISTIO_MUTUAL
  subsets:
    - name: stable
      labels:
        version: stable
    - name: canary
      labels:
        version: canary
```

### ポイント

- gRPC 使用時は `h2UpgradePolicy: UPGRADE` を設定し、HTTP/2 アップグレードを有効化する
- ロードバランサーは全 Tier 共通で `LEAST_REQUEST` を採用する
- Circuit Breaker（outlierDetection）は Tier 別のデフォルト値を適用する
- TLS は `ISTIO_MUTUAL` で mTLS を強制する
- サブセットは `stable` / `canary` の2つを定義し、カナリアリリースに対応する

---

## PeerAuthentication テンプレート（peer-authentication.yaml.tera）

サービス単位で mTLS を STRICT モードで強制する。メッシュワイドデフォルト（`service-mesh` Namespace）に加えて、サービス個別の PeerAuthentication を定義することで防御を多層化する。

```tera
apiVersion: security.istio.io/v1
kind: PeerAuthentication
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ service_name }}
  mtls:
    mode: STRICT
{% if api_styles is containing("grpc") %}
  portLevelMtls:
    {{ grpc_port }}:
      mode: STRICT
{% endif %}
```

### ポイント

- `selector` でサービス固有の Pod に適用する
- メッシュワイドデフォルト（`service-mesh` Namespace の `PeerAuthentication`）が誤って変更された場合でも、サービス単位の設定が防御層として機能する
- gRPC 使用時は gRPC ポートに対しても明示的に STRICT モードを設定する

---

## AuthorizationPolicy テンプレート（authorization-policy.yaml.tera）

Tier 間のアクセス制御を定義する。`tier` に応じて許可するソース Namespace が異なる。

```tera
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: {{ service_name }}-allow
  namespace: {{ namespace }}
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ service_name }}
  action: ALLOW
  rules:
{% if tier == "system" %}
    # system 層: business・service・同一 Tier からのアクセスを許可
    - from:
        - source:
            namespaces: ["k1s0-business", "k1s0-service"]
    - from:
        - source:
            namespaces: ["k1s0-system"]
{% elif tier == "business" %}
    # business 層: service・同一 Tier からのアクセスを許可
    - from:
        - source:
            namespaces: ["k1s0-service"]
    - from:
        - source:
            namespaces: ["k1s0-business"]
{% elif tier == "service" %}
    # service 層: ingress・同一 Tier からのアクセスを許可
    - from:
        - source:
            namespaces: ["ingress"]
    - from:
        - source:
            namespaces: ["k1s0-service"]
{% endif %}
{% if kind == "bff" %}
---
# BFF 間の直接通信を禁止
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: {{ service_name }}-deny-bff
  namespace: {{ namespace }}
spec:
  action: DENY
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ service_name }}
      app.kubernetes.io/component: bff
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/{{ namespace }}/sa/*-bff-sa"]
{% endif %}
```

### ポイント

- Tier 間のアクセス制御は [認証認可設計](../../architecture/auth/認証認可設計.md) の定義に基づく
- `system` 層は下位 Tier（business / service）と同一 Tier からのアクセスを許可する
- `business` 層は service Tier と同一 Tier からのアクセスを許可する
- `service` 層は ingress と同一 Tier からのアクセスを許可する
- `kind == "bff"` の場合、BFF 間の直接通信を禁止する DENY ポリシーを追加生成する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                    | 選択肢                              | 生成への影響                                              |
| ----------------------- | ----------------------------------- | --------------------------------------------------------- |
| Tier (`tier`)           | `system` / `business` / `service`   | timeout / retry / Circuit Breaker / connectionPool のデフォルト値 |
| Tier (`tier`)           | `system` / `business` / `service`   | AuthorizationPolicy の許可 Namespace                      |
| API 方式 (`api_styles`) | `grpc` を含む                       | VirtualService に gRPC ルート追加                         |
| API 方式 (`api_styles`) | `grpc` を含む                       | DestinationRule に `h2UpgradePolicy: UPGRADE` 追加        |
| API 方式 (`api_styles`) | `grpc` を含む                       | PeerAuthentication に gRPC ポートの portLevelMtls 追加    |
| API 方式 (`api_styles`) | `grpc` を含む                       | VirtualService の retryOn を gRPC 固有の条件に変更        |
| kind (`kind`)           | `bff`                               | AuthorizationPolicy に BFF 間通信禁止の DENY ポリシー追加 |
| kind (`kind`)           | `server` / `bff` 以外              | サービスメッシュリソースを生成しない                      |

---

## 生成例

### system Tier の Go gRPC サーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "tier": "system",
  "namespace": "k1s0-system",
  "kind": "server",
  "api_styles": ["grpc"],
  "server_port": 80,
  "grpc_port": 9090
}
```

生成されるファイル:
- `infra/service-mesh/auth-service/virtual-service.yaml` -- timeout=5s, retry=3, gRPC ルート付き
- `infra/service-mesh/auth-service/destination-rule.yaml` -- maxConnections=200, consecutive5xxErrors=3, h2UpgradePolicy=UPGRADE
- `infra/service-mesh/auth-service/peer-authentication.yaml` -- STRICT + gRPC ポート portLevelMtls
- `infra/service-mesh/auth-service/authorization-policy.yaml` -- business / service / system からのアクセス許可

### service Tier の REST サーバーの場合

入力:
```json
{
  "service_name": "order-server",
  "tier": "service",
  "namespace": "k1s0-service",
  "kind": "server",
  "api_styles": ["rest"],
  "server_port": 80
}
```

生成されるファイル:
- `infra/service-mesh/order-server/virtual-service.yaml` -- timeout=15s, retry=2, HTTP ルートのみ
- `infra/service-mesh/order-server/destination-rule.yaml` -- maxConnections=100, consecutive5xxErrors=5, baseEjectionTime=60s
- `infra/service-mesh/order-server/peer-authentication.yaml` -- STRICT
- `infra/service-mesh/order-server/authorization-policy.yaml` -- ingress / service からのアクセス許可

### service Tier の BFF（REST + gRPC）の場合

入力:
```json
{
  "service_name": "order-bff",
  "tier": "service",
  "namespace": "k1s0-service",
  "kind": "bff",
  "api_styles": ["rest", "grpc"],
  "server_port": 80,
  "grpc_port": 9090
}
```

生成されるファイル:
- `infra/service-mesh/order-bff/virtual-service.yaml` -- timeout=15s, retry=2, HTTP + gRPC ルート
- `infra/service-mesh/order-bff/destination-rule.yaml` -- maxConnections=100, h2UpgradePolicy=UPGRADE
- `infra/service-mesh/order-bff/peer-authentication.yaml` -- STRICT + gRPC ポート portLevelMtls
- `infra/service-mesh/order-bff/authorization-policy.yaml` -- ingress / service からのアクセス許可 + BFF 間通信禁止 DENY ポリシー

### business Tier の REST サーバーの場合

入力:
```json
{
  "service_name": "accounting-api",
  "tier": "business",
  "namespace": "k1s0-business",
  "kind": "server",
  "api_styles": ["rest"],
  "server_port": 80
}
```

生成されるファイル:
- `infra/service-mesh/accounting-api/virtual-service.yaml` -- timeout=10s, retry=3
- `infra/service-mesh/accounting-api/destination-rule.yaml` -- maxConnections=100, consecutive5xxErrors=5, baseEjectionTime=30s
- `infra/service-mesh/accounting-api/peer-authentication.yaml` -- STRICT
- `infra/service-mesh/accounting-api/authorization-policy.yaml` -- service / business からのアクセス許可

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [サービスメッシュ設計](../../infrastructure/service-mesh/サービスメッシュ設計.md) -- Istio 詳細設計・耐障害性設計
- [認証認可設計](../../architecture/auth/認証認可設計.md) -- mTLS・AuthorizationPolicy の詳細定義
- [テンプレート仕様-Helm](../infrastructure/Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-CICD](../infrastructure/CICD.md) -- CI/CD テンプレート仕様
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) -- Namespace・NetworkPolicy 設計
- [helm設計](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart と values 設計
