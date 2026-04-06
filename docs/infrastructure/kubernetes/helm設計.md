# Helm 設計

k1s0 のアプリケーションデプロイに使用する Helm Chart の設計を定義する。

## 基本方針

- サービスごとに個別の Helm Chart を作成する
- 共通設定は Library Chart として切り出し、各 Chart で参照する
- 環境別の差分は `values-{env}.yaml` で管理する
- シークレットは Vault Agent Injector で Pod に注入する

### k1s0-common 依存の本番移行（H-007）

現状、全サービスの Chart.yaml で k1s0-common を `file://` パスで参照している。
この方式はローカル開発では便利だが、CI/CD パイプラインや外部環境では壊れる可能性がある。

**現状（開発環境用）:**
```yaml
repository: "file://../../../charts/k1s0-common"
```

**本番移行時の設定（TODO）:**
```yaml
repository: "oci://harbor.k1s0.io/helm-charts"
```

本番デプロイ前に Harbor 等の内部 Helm Registry に k1s0-common を publish し、参照を変更すること。

## Chart 構成

```
infra/helm/
├── charts/
│   └── k1s0-common/                 # Library Chart（共通テンプレート）
│       ├── Chart.yaml
│       └── templates/
│           ├── _deployment.tpl
│           ├── _service.tpl
│           ├── _hpa.tpl
│           ├── _pdb.tpl
│           ├── _configmap.tpl
│           ├── _ingress.tpl
│           ├── _vault-annotations.tpl
│           └── _helpers.tpl
└── services/
    ├── system/
    │   ├── auth/                    # 認証・認可サービス（Rust, gRPC+HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── config/                  # 構成管理サービス（Rust, gRPC+HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── saga/                    # Saga オーケストレータ（Rust, gRPC+HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── dlq-manager/             # DLQ 管理サービス（Rust, HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── bff-proxy/               # BFF プロキシ（Go, HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── graphql-gateway/         # GraphQL Gateway（Go, HTTP）
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   ├── kong/                    # Kong API Gateway
    │   │   ├── Chart.yaml
    │   │   ├── values.yaml
    │   │   ├── values-dev.yaml
    │   │   ├── values-staging.yaml
    │   │   ├── values-prod.yaml
    │   │   └── templates/
    │   └── app-registry/            # アプリ配布メタデータ管理（Rust, HTTP）
    │       ├── Chart.yaml
    │       ├── values.yaml
    │       ├── values-dev.yaml
    │       ├── values-staging.yaml
    │       ├── values-prod.yaml
    │       └── templates/
    ├── business/
    │   └── taskmanagement/
    │       └── project-master/
    │           ├── Chart.yaml
    │           ├── values.yaml
    │           ├── values-dev.yaml
    │           ├── values-staging.yaml
    │           ├── values-prod.yaml
    │           └── templates/
    └── service/
        └── task/
            ├── Chart.yaml
            ├── values.yaml
            ├── values-dev.yaml
            ├── values-staging.yaml
            ├── values-prod.yaml
            └── templates/
```

## System Tier Chart 一覧

system tier には以下の 14 つの Chart が存在する。全て `k1s0-common` Library Chart に依存し、`labels.tier: system` を設定する。

| Chart | 説明 | 言語 | gRPC | Kafka | Redis | Vault secrets |
| --- | --- | --- | --- | --- | --- | --- |
| auth | 認証・認可（JWT 検証、ユーザー管理） | Rust | 50051 | ✓ | - | DB パスワード |
| config | 構成管理（サービス設定の集中管理） | Rust | 50051 | - | - | DB パスワード |
| saga | Saga オーケストレータ（分散トランザクション） | Rust | 50051 | ✓ | - | DB パスワード |
| dlq-manager | Dead Letter Queue 管理（失敗メッセージの再処理） | Rust | - | ✓ | - | DB パスワード |
| bff-proxy | BFF プロキシ（OIDC 認証、セッション管理、リバースプロキシ） | Go | - | - | ✓ | OIDC client secret, Redis パスワード |
| graphql-gateway | GraphQL Gateway（フェデレーション、クエリルーティング） | Rust | - | - | - | JWKS 署名鍵 |
| kong | API Gateway（DB-backed PostgreSQL モード） | - | - | - | - | DB パスワード（SecretKeyRef） |
| app-registry | アプリバイナリメタデータ管理・直接配信（[アプリ配布基盤設計](../distribution/アプリ配布基盤設計.md)） | Rust | - | - | - | DB パスワード |
| featureflag | 動的フィーチャーフラグ管理（フラグ評価・ルール配信） | Go | 50056 | ✓ | - | DB パスワード |
| ratelimit | レート制限（ルールベースのスロットリング） | Go | 50057 | ✓ | - | DB パスワード |
| tenant | テナント管理（マルチテナント設定・プロビジョニング） | Go | 50058 | ✓ | - | DB パスワード |
| vault | シークレット管理サービス（Vault ポリシー・ロール管理） | Go | 50059 | - | - | DB パスワード |
| ai-gateway | LLM ルーティング・プロキシ（マルチプロバイダー対応） | Go | 50061 | - | - | LLM API キー |
| ai-agent | AI エージェント実行基盤（ツール呼び出し・ワークフロー） | Go | 50062 | ✓ | - | DB パスワード |

## Business Tier Chart 一覧

business tier には以下の Chart が存在する。全て `k1s0-common` Library Chart に依存し、`labels.tier: business` を設定する。

| Chart | 説明 | 言語 | gRPC | Kafka | Vault secrets |
| --- | --- | --- | --- | --- | --- |
| project-master | プロジェクトマスタ管理（プロジェクト・種別・コードマスタ） | Go | 9210 | ✓ | DB パスワード |

## Service Tier Chart 一覧

service tier には以下の Chart が存在する。全て `k1s0-common` Library Chart に依存し、`labels.tier: service` を設定する。

| Chart | 説明 | 言語 | gRPC | Kafka | Vault secrets |
| --- | --- | --- | --- | --- | --- |
| board | ボード管理（ボード照会・レーン・カード管理） | Go | - | ✓ | DB パスワード |
| activity | アクティビティ処理（操作ログ・通知・状態管理） | Go | - | ✓ | DB パスワード |
| service-catalog | サービスカタログ（提供サービス定義・価格管理） | Go | - | - | DB パスワード |

### 実ファイル配置

各 Chart は `infra/helm/services/{tier}/` 配下に配置されている。

| Tier | Chart | Chart.yaml パス | values.yaml パス |
|------|-------|----------------|-----------------|
| system | auth | `infra/helm/services/system/auth/Chart.yaml` | `infra/helm/services/system/auth/values.yaml` |
| system | config | `infra/helm/services/system/config/Chart.yaml` | `infra/helm/services/system/config/values.yaml` |
| system | saga | `infra/helm/services/system/saga/Chart.yaml` | `infra/helm/services/system/saga/values.yaml` |
| system | dlq-manager | `infra/helm/services/system/dlq-manager/Chart.yaml` | `infra/helm/services/system/dlq-manager/values.yaml` |
| system | bff-proxy | `infra/helm/services/system/bff-proxy/Chart.yaml` | `infra/helm/services/system/bff-proxy/values.yaml` |
| system | graphql-gateway | `infra/helm/services/system/graphql-gateway/Chart.yaml` | `infra/helm/services/system/graphql-gateway/values.yaml` |
| system | kong | `infra/helm/services/system/kong/Chart.yaml` | `infra/helm/services/system/kong/values.yaml` |
| system | app-registry | `infra/helm/services/system/app-registry/Chart.yaml` | `infra/helm/services/system/app-registry/values.yaml` |
| system | featureflag | `infra/helm/services/system/featureflag/Chart.yaml` | `infra/helm/services/system/featureflag/values.yaml` |
| system | ratelimit | `infra/helm/services/system/ratelimit/Chart.yaml` | `infra/helm/services/system/ratelimit/values.yaml` |
| system | tenant | `infra/helm/services/system/tenant/Chart.yaml` | `infra/helm/services/system/tenant/values.yaml` |
| system | vault | `infra/helm/services/system/vault/Chart.yaml` | `infra/helm/services/system/vault/values.yaml` |
| system | ai-gateway | `infra/helm/services/system/ai-gateway/Chart.yaml` | `infra/helm/services/system/ai-gateway/values.yaml` |
| system | ai-agent | `infra/helm/services/system/ai-agent/Chart.yaml` | `infra/helm/services/system/ai-agent/values.yaml` |
| business | project-master | `infra/helm/services/business/project-master/Chart.yaml` | `infra/helm/services/business/project-master/values.yaml` |
| service | board | `infra/helm/services/service/board/Chart.yaml` | `infra/helm/services/service/board/values.yaml` |
| service | activity | `infra/helm/services/service/activity/Chart.yaml` | `infra/helm/services/service/activity/values.yaml` |
| service | service-catalog | `infra/helm/services/service/service-catalog/Chart.yaml` | `infra/helm/services/service/service-catalog/values.yaml` |

全 Chart は `k1s0-common` Library Chart に依存し、`appVersion: "0.1.0"`（kong は `"3.8.0"`）。

### 各 Chart の values.yaml 重要フィールド差分（System Tier）

| フィールド | auth | config | saga | dlq-manager | bff-proxy | graphql-gateway | kong |
|-----------|------|--------|------|-------------|-----------|-----------------|------|
| `container.grpcPort` | 50051 | 50051 | 50051 | null | - | null | - |
| `kafka.enabled` | true | false | true | true | - | false | - |
| `redis.enabled` | false | false | false | false | true | false | - |
| `autoscaling.maxReplicas` | 5 | 5 | 5 | 5 | 10 | 5 | - |
| `resources.requests.cpu` | 250m | 250m | 250m | 250m | 100m | 250m | 500m |
| `resources.requests.memory` | 256Mi | 256Mi | 256Mi | 256Mi | 128Mi | 256Mi | 512Mi |
| `vault.secrets` | DB | DB | DB | DB | OIDC + Redis | JWKS 署名鍵 | SecretKeyRef |

### 新規 System Tier Chart の values.yaml 重要フィールド差分

| フィールド | featureflag | ratelimit | tenant | vault | ai-gateway | ai-agent |
|-----------|-------------|-----------|--------|-------|------------|----------|
| `container.port` | 8087 | 8088 | 8089 | 8091 | 8120 | 8121 |
| `container.grpcPort` | 50056 | 50057 | 50058 | 50059 | 50061 | 50062 |
| `kafka.enabled` | true | true | true | false | false | true |
| `vault.role` | system | system | system | system | system | system |
| `vault.secrets` | DB | DB | DB | DB | API キー | DB |

### Business / Service Tier Chart の values.yaml 重要フィールド差分

| フィールド | project-master | board | activity | service-catalog |
|-----------|----------------|-------|----------|-----------------|
| `container.port` | 8210 | 8311 | 8312 | 8313 |
| `container.grpcPort` | 9210 | 0 | 0 | 0 |
| `kafka.enabled` | true | true | true | false |
| `labels.tier` | business | service | service | service |
| `vault.role` | business | service | service | service |

### 共通設定（auth / config / saga / dlq-manager）

Rust 製サービスの共通パラメータ:

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/{service-name}
container:
  port: 8080
  grpcPort: 50051        # dlq-manager は null
service:
  type: ClusterIP
  port: 80
  grpcPort: 50051        # dlq-manager は null
vault:
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/{service-name}/database"
labels:
  tier: system
```

### bff-proxy 固有の設定

bff-proxy は Go 製の BFF（Backend for Frontend）で、OIDC 認証フローとセッション管理を担当する。他の Rust サービスとは以下の点が異なる:

```yaml
container:
  port: 8080
  # grpcPort なし（HTTP のみ）
resources:
  requests:
    cpu: 100m              # 軽量プロキシのため低リソース
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi
autoscaling:
  maxReplicas: 10          # フロントエンドトラフィック対応のため高めに設定
vault:
  secrets:
    - path: "secret/data/k1s0/system/bff-proxy/oidc"
      key: "client_secret"
    - path: "secret/data/k1s0/system/bff-proxy/redis"
      key: "password"
redis:
  enabled: true
  host: "redis.k1s0-system.svc.cluster.local"
metrics:
  enabled: true
  port: 8080
  path: /metrics
serviceMonitor:
  enabled: true
  interval: 30s
```

### graphql-gateway 固有の設定

graphql-gateway は Go 製の GraphQL Federation Gateway で、複数のサブグラフをフェデレーションし、クエリをルーティングする。

```yaml
container:
  port: 8080
  grpcPort: null               # HTTP のみ
resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi
autoscaling:
  maxReplicas: 5
vault:
  secrets:
    - path: "secret/data/k1s0/system/graphql-gateway/auth"
      key: "jwks-signing-key"
      mountPath: "/vault/secrets/jwks-key"
kafka:
  enabled: false
redis:
  enabled: false
```

### Kong 固有の設定

Kong は公式 Helm Chart のカスタム values を使用する。詳細は [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) を参照。

```yaml
image:
  tag: "3.8"
env:
  database: postgres
proxy:
  type: ClusterIP
ingressController:
  enabled: false           # Admin API で管理
postgresql:
  enabled: false           # 外部 PostgreSQL を使用
replicaCount: 2
```

#### Kong Admin API TLS 設定方針（C-04 監査対応）

Kong Admin API はクラスタ全体のルーティング・プラグイン設定を管理する高権限エンドポイントである。
以下のセキュリティポリシーを必須とする。

| 項目 | 設定値 | 理由 |
|------|--------|------|
| `admin.http.enabled` | `false` | HTTP平文アクセスを禁止。盗聴・改ざんリスクを排除する |
| `admin.tls.enabled` | `true` | TLS暗号化通信のみを許可する |
| `admin.type` | `ClusterIP` | クラスタ内部にのみ公開。外部からのアクセスを遮断する |

**開発環境（docker-compose）での注意事項:**

- Admin API のホストポートバインドは `0.0.0.0` ではなく `127.0.0.1` に制限すること
- `KONG_ADMIN_HOST_PORT` 環境変数を使用する場合も `127.0.0.1:${KONG_ADMIN_HOST_PORT:-8001}:8001` 形式で指定すること
- `0.0.0.0:8001` バインドは開発機が同一ネットワーク上の別端末から不正操作されるリスクがある

```yaml
# infra/helm/services/system/kong/values.yaml — Admin API セキュリティ設定
admin:
  enabled: true
  type: ClusterIP
  http:
    enabled: false    # HTTP平文アクセス禁止（C-04監査対応）
  tls:
    enabled: true     # TLSのみ許可
```

## Chart.yaml

```yaml
# infra/helm/services/service/order/Chart.yaml
apiVersion: v2
name: order
description: Order service
type: application
version: 0.1.0        # Chart バージョン
appVersion: "1.0.0"   # アプリケーションバージョン

dependencies:
  - name: k1s0-common
    version: "0.1.0"
    repository: "file://../../../charts/k1s0-common"   # services/service/order/ → helm/charts/ への相対パス
```

> **注**: `repository` の相対パスは Chart.yaml の配置場所に応じて変わる。
> - system / service tier（`services/{tier}/{service}/`）: `file://../../../charts/k1s0-common`
> - business tier（`services/business/{domain}/{service}/`）: `file://../../../../charts/k1s0-common`

## values.yaml 設計

### デフォルト値（values.yaml）

```yaml
# 共通設定
nameOverride: ""
fullnameOverride: ""

# イメージ
image:
  registry: harbor.internal.example.com
  repository: k1s0-service/order
  tag: ""                              # CI/CD で上書き
  pullPolicy: IfNotPresent

imagePullSecrets:
  - name: harbor-pull-secret

# レプリカ
replicaCount: 2

# コンテナ設定
container:
  port: 8080
  grpcPort: null                       # gRPC 無効。gRPC 有効時は 50051 を設定（config.md の grpc.port デフォルト値と一致）
  command: []
  args: []

# リソース
resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi

# ヘルスチェック
probes:
  liveness:
    httpGet:
      path: /healthz
      port: http
    initialDelaySeconds: 10
    periodSeconds: 15
    failureThreshold: 3
  readiness:
    httpGet:
      path: /readyz
      port: http
    initialDelaySeconds: 5
    periodSeconds: 5
    failureThreshold: 3

# Service
service:
  type: ClusterIP
  port: 80
  grpcPort: null                       # gRPC 無効（有効時は 50051 を設定）

# HPA
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5                # kubernetes設計.md の staging 設定と同じ値を採用
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60    # スパイク時の過剰スケールアップを抑制
      policies:
        - type: Pods
          value: 2
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300   # トラフィック減少後の急なスケールダウンを防止
      policies:
        - type: Pods
          value: 1
          periodSeconds: 120

# PodDisruptionBudget
pdb:
  enabled: true
  minAvailable: 1

# Ingress（Kong 経由のため通常は無効）
ingress:
  enabled: false
  ingressClassName: nginx       # nginx or kong（管理系UIは nginx を指定）

# セキュリティコンテキスト
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 65532              # distroless nonroot ユーザー（Dockerイメージ戦略.md と同期）
  fsGroup: 65532
containerSecurityContext:
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop: ["ALL"]

# config.yaml のマウント
# 本番（Kubernetes）: /etc/app に ConfigMap をマウント
# 開発（Docker Compose）: /app/config にボリュームマウント（compose-システムサービス設計.md 参照）
config:
  mountPath: /etc/app
  data: {}                             # ConfigMap として作成

# Vault Agent Injector
vault:
  enabled: true
  role: "service"                        # Tier 名を指定（認証認可設計.md の Vault Kubernetes Auth ロールに対応）
  secrets:
    - path: "secret/data/k1s0/service/order/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"

# Pod 設定
nodeSelector: {}
tolerations: []
affinity: {}

# サービスアカウント
serviceAccount:
  create: true
  name: ""
  annotations: {}

# ラベル（Kubernetes 設計のラベル規約に準拠）
labels:
  tier: service

# Kafka（有効時のみ）
kafka:
  enabled: false
  brokers: []

# Redis（有効時のみ）
redis:
  enabled: false
  host: ""
```

### 環境別オーバーライド

#### values-dev.yaml

```yaml
replicaCount: 1

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: false

pdb:
  enabled: false

vault:
  enabled: false              # dev は Vault なしで動作

config:
  data:
    config.yaml: |
      app:
        environment: dev
      database:
        host: postgres.k1s0-service.svc.cluster.local
        ssl_mode: disable
      observability:
        log:
          level: debug
          format: text
        trace:
          sample_rate: 1.0
```

#### values-staging.yaml

```yaml
replicaCount: 2

resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5

config:
  data:
    config.yaml: |
      app:
        environment: staging
      database:
        host: postgres.k1s0-service.svc.cluster.local
        ssl_mode: require
      observability:
        log:
          level: info
          format: json
        trace:
          sample_rate: 0.5
```

#### values-prod.yaml

```yaml
replicaCount: 3

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10

affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
            - key: app.kubernetes.io/name
              operator: In
              values:
                - order
        topologyKey: kubernetes.io/hostname

config:
  data:
    config.yaml: |
      app:
        environment: prod
      database:
        host: postgres.k1s0-service.svc.cluster.local
        ssl_mode: verify-full
        max_open_conns: 50
      observability:
        log:
          level: warn
          format: json
        trace:
          sample_rate: 0.1
```

## デプロイコマンド

```bash
# dev 環境
helm upgrade --install task ./infra/helm/services/service/task \
  -n k1s0-service \
  -f ./infra/helm/services/service/task/values-dev.yaml \
  --set image.tag=1.0.0-a1b2c3d

# staging 環境
helm upgrade --install task ./infra/helm/services/service/task \
  -n k1s0-service \
  -f ./infra/helm/services/service/task/values-staging.yaml \
  --set image.tag=1.0.0-a1b2c3d

# prod 環境
helm upgrade --install task ./infra/helm/services/service/task \
  -n k1s0-service \
  -f ./infra/helm/services/service/task/values-prod.yaml \
  --set image.tag=1.0.0-a1b2c3d
```

## シークレット注入（Vault Agent Injector）

Pod の annotations で Vault Agent Injector にシークレットの注入を指示する。

```yaml
# templates/deployment.yaml（抜粋）
spec:
  template:
    metadata:
      annotations:
        vault.hashicorp.com/agent-inject: "true"
        vault.hashicorp.com/role: "{{ .Values.vault.role }}"
        vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/service/order/database"
        vault.hashicorp.com/agent-inject-template-db-password: |
          {{`{{ with secret "secret/data/k1s0/service/order/database" }}{{ .Data.data.password }}{{ end }}`}}
```

## Library Chart（k1s0-common）

各サービスの Chart で共通するテンプレートを Library Chart として提供する。

### バージョニング方針

Library Chart（k1s0-common）はセマンティックバージョニング（SemVer）を採用する。

| バージョン種別 | 変更内容                                                                 |
| -------------- | ------------------------------------------------------------------------ |
| MAJOR          | テンプレートの破壊的変更（values.yaml のキー名変更・削除等）             |
| MINOR          | 新しいテンプレート・パラメータの追加（後方互換あり）                     |
| PATCH          | バグ修正、ドキュメント更新                                               |

- バージョンは `Chart.yaml` の `version` フィールドで管理する
- 各サービスの `Chart.yaml` では `dependencies[].version` にチルダ範囲指定（`~1.x.x`）を推奨する

```yaml
# 各サービスの Chart.yaml での依存指定例（system / service tier の場合）
dependencies:
  - name: k1s0-common
    version: "~1.2.0"    # 1.2.x の PATCH アップデートを自動追従
    repository: "file://../../../charts/k1s0-common"
```

### _deployment.tpl（抜粋）

```yaml
{{- define "k1s0-common.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "k1s0-common.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "k1s0-common.labels" . | nindent 8 }}
    spec:
      {{- with .Values.podSecurityContext }}
      securityContext:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          {{- with .Values.containerSecurityContext }}
          securityContext:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          ports:
            - name: http
              containerPort: {{ .Values.container.port }}
            {{- if .Values.container.grpcPort }}
            - name: grpc
              containerPort: {{ .Values.container.grpcPort }}
              protocol: TCP
            {{- end }}
          {{- with .Values.probes.liveness }}
          livenessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.probes.readiness }}
          readinessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          volumeMounts:
            - name: config
              mountPath: {{ .Values.config.mountPath }}
              readOnly: true
      volumes:
        - name: config
          configMap:
            name: {{ include "k1s0-common.fullname" . }}-config
{{- end }}
```

### _service.tpl（抜粋）

```yaml
{{- define "k1s0-common.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - name: http
      port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
    {{- if .Values.service.grpcPort }}
    - name: grpc
      port: {{ .Values.service.grpcPort }}
      targetPort: grpc
      protocol: TCP
    {{- end }}
  selector:
    {{- include "k1s0-common.selectorLabels" . | nindent 4 }}
{{- end }}
```

### _ingress.tpl（抜粋）

管理系UI（Grafana, Prometheus, Jaeger 等）は Kong を経由せず、Nginx Ingress から直接ルーティングする。
`ingressClassName` パラメータで `nginx` を指定し、管理系サービス専用の Ingress リソースを個別に定義する。

- ホスト名パターン: `{service}.k1s0.internal.example.com`（例: `grafana.k1s0.internal.example.com`）
- API サービス: Kong 経由（`api.k1s0.internal.example.com` → Kong Proxy）
- 管理系サービス: Nginx Ingress から直接ルーティング

```yaml
{{- define "k1s0-common.ingress" -}}
{{- if .Values.ingress.enabled }}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "k1s0-common.fullname" . }}
  labels:
    {{- include "k1s0-common.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  ingressClassName: {{ .Values.ingress.ingressClassName | default "nginx" }}
  {{- with .Values.ingress.tls }}
  tls:
    {{- toYaml . | nindent 4 }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            pathType: {{ .pathType | default "Prefix" }}
            backend:
              service:
                name: {{ .backend.serviceName }}
                port:
                  number: {{ .backend.servicePort }}
          {{- end }}
    {{- end }}
{{- end }}
{{- end }}
```

## リリース管理

| 操作             | コマンド                                                    |
| ---------------- | ----------------------------------------------------------- |
| インストール     | `helm upgrade --install {name} {chart} -n {ns} -f {values}` |
| ロールバック     | `helm rollback {name} {revision} -n {ns}`                   |
| 履歴確認         | `helm history {name} -n {ns}`                               |
| アンインストール | `helm uninstall {name} -n {ns}`                             |

- リリース履歴は直近 10 リビジョンを保持する（`--history-max 10`）
- prod のロールバックは即座に実行できるよう、前バージョンの動作確認を staging で事前に行う

## k1s0-common テンプレート nil-safe 規約（C-1 対応）

### 背景

Helm Library Chart（type: library）の `values.yaml` は、使用側チャートの `helm template` / `helm install` 時に**自動マージされない**。
そのため `k1s0-common/values.yaml` に記述されたデフォルト値は、サービス側の `values.yaml` で明示的に上書きされていない場合は nil になりうる。
この仕様により、サービス固有の `values.yaml` に特定のキーが存在しない場合、テンプレート内でそのキーにアクセスするとパニックが発生する。

### 修正済みの nil-safe パターン（2026-03-24: C-1 対応）

| テンプレートファイル | nil-safe 修正内容 |
|---|---|
| `_pdb.tpl` | `{{- if and .Values.pdb .Values.pdb.enabled }}` |
| `_hpa.tpl` | `{{- if and .Values.autoscaling .Values.autoscaling.enabled }}` |
| `_deployment.tpl` | `{{- if not (and .Values.autoscaling .Values.autoscaling.enabled) }}` |
| `_deployment.tpl` | `{{- if and .Values.container .Values.container.command }}` |
| `_deployment.tpl` | `{{- if and .Values.container .Values.container.grpcPort }}` |
| `_deployment.tpl` | `{{- with .Values.probes }}` で probes nil ガード追加 |
| `_deployment.tpl` | `mountPath: {{ if .Values.config }}...{{ else }}/etc/app{{ end }}` |
| `_virtualservice.tpl` | `{{- if and .Values.istio .Values.istio.enabled .Values.istio.virtualService .Values.istio.virtualService.enabled }}` |
| `_destinationrule.tpl` | `{{- if and .Values.istio .Values.istio.enabled .Values.istio.destinationRule .Values.istio.destinationRule.enabled }}` |
| `_service.tpl` | `type: {{ if .Values.service }}...{{ else }}ClusterIP{{ end }}` |
| `_service.tpl` | `{{- if and .Values.service .Values.service.grpcPort }}` |
| `_configmap.tpl` | `{{- range $key, $value := ((.Values.config).data) }}` |
| `_helpers.tpl` | `{{- if and .Values.serviceAccount .Values.serviceAccount.create }}` |

### K8S-CRIT-001 対応: vaultAnnotations の _deployment.tpl への統合（2026-04-04）

Vault Agent Injector がシークレットを Pod に注入するには、Pod の `metadata.annotations` に Vault 固有のアノテーション（`vault.hashicorp.com/agent-inject: "true"` 等）が必要。
以前の実装では `_vault-annotations.tpl` ヘルパーは定義されていたが、`_deployment.tpl` から呼び出されていなかったため、全サービスで Vault 注入が機能しない致命的欠陥があった（K8S-CRIT-001）。

**修正内容**:
- `_deployment.tpl` の `template.metadata.annotations` ブロックに `{{- include "k1s0-common.vaultAnnotations" . | nindent 8 }}` を追加
- `vaultAnnotations` を `istioAnnotations` より前に配置することで、Vault init コンテナが Istio サイドカーより先に起動し、アプリ起動前にシークレットが確実にマウントされる順序を保証する

**影響範囲**: k1s0-common を使用する全サービスチャート（31 サービス）

### K8S-CRIT-002 対応: service tier ServiceAccount マニフェスト追加（2026-04-04）

Vault の Kubernetes auth は `bound_service_account_names` でサービスアカウント名を検証する。
service tier の task / board / activity の各サービスに `templates/serviceaccount.yaml` が存在しなかったため、
`serviceAccount.create: true` を設定しても ServiceAccount が作成されず、Vault 認証に失敗する欠陥があった（K8S-CRIT-002）。

**修正内容**:
- `infra/helm/services/service/task/templates/serviceaccount.yaml` を新規作成
- `infra/helm/services/service/board/templates/serviceaccount.yaml` を新規作成
- `infra/helm/services/service/activity/templates/serviceaccount.yaml` を新規作成
- 各サービスの `values.yaml` に `serviceAccount.create: true` を追加

**ServiceAccount テンプレートの形式**: `k1s0-common.serviceAccountName` ヘルパーを使用し、`serviceAccount.create` が true の場合のみ作成する（auth サービスと同一パターン）。

### テンプレート修正時の規約

新しいテンプレートを追加・変更する場合は、以下の規約に従うこと:

1. **オブジェクト型の値へのアクセス**: `{{- if .Values.parent }}{{ .Values.parent.child }}{{ end }}` の形で親オブジェクトの nil チェックを先行させる
2. **条件分岐での比較**: `{{- if and .Values.parent .Values.parent.enabled }}` を使用する
3. **デフォルト値の提供**: `{{ if .Values.service }}{{ .Values.service.type | default "ClusterIP" }}{{ else }}ClusterIP{{ end }}`
4. **`range` での nil-safe**: `((.Values.config).data)` のように二重括弧を使用する（Go テンプレートの optional chaining）

### CI による継続検証

`helm lint` は `infra/helm/**` 変更時と定期実行時に全 31 サービスチャートに対して自動実行される（`.github/workflows/_validate.yaml` の `helm-lint` ジョブ）。

---

## ローカル開発での Helm Chart 検証手順（I-03 監査対応）

PR 前に以下の手順でローカル検証を行うこと。CI の `helm lint` が通ることは必要条件だが、実際のレンダリング結果は `helm template` で確認すること。

### 1. helm lint（構文チェック）

```bash
# 単一チャートの lint
helm lint infra/helm/services/service/task -f infra/helm/services/service/task/values-dev.yaml

# 全チャートの一括 lint（justfile 経由）
just helm-lint
```

### 2. helm template（レンダリング結果確認）

```bash
# dev 環境向けのマニフェストをレンダリングして確認する
helm template task infra/helm/services/service/task \
  -f infra/helm/services/service/task/values-dev.yaml \
  --set image.tag=local \
  --namespace k1s0-service \
  | less

# 特定のリソースのみ確認する（例: Deployment のみ）
helm template task infra/helm/services/service/task \
  -f infra/helm/services/service/task/values-dev.yaml \
  --set image.tag=local \
  | yq 'select(.kind == "Deployment")'
```

### 3. kind / minikube でのドライラン（推奨）

ローカル Kubernetes クラスタ（kind または minikube）で `--dry-run=server` を使用すると、サーバーサイドのバリデーションも確認できる。

#### 3-1. ローカル開発環境向け CRD インストール手順

cert-manager および Istio の CRD がインストールされていない環境では `kubectl apply --dry-run=client` が `unknown field` エラーで失敗する。
以下の手順で事前に CRD をインストールしてからドライランを実行すること。

```bash
# kind クラスタを作成する（初回のみ）
kind create cluster --name k1s0-local

# ---- cert-manager CRD のインストール ----
# cert-manager v1.x の CRD を一括インストールする（infra/kubernetes/cert-manager/ が依存）
# 本番環境と同じ cert-manager.io/v1 API バージョンを使用
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.4/cert-manager.crds.yaml

# CRD の登録が完了するまで待機する（established 状態になるまで）
kubectl wait --for=condition=established --timeout=60s \
  crd/certificates.cert-manager.io \
  crd/clusterissuers.cert-manager.io \
  crd/issuers.cert-manager.io

# ---- Istio CRD のインストール ----
# 本プロジェクトは Istio 1.20.0 を使用する（infra/terraform/environments/dev/variables.tf 参照）
helm repo add istio https://istio-release.storage.googleapis.com/charts
helm repo update

# istio-base チャートで Istio の CRD（VirtualService, DestinationRule, PeerAuthentication 等）を登録する
helm upgrade --install istio-base istio/base \
  --version 1.20.0 \
  --namespace service-mesh \
  --create-namespace \
  --dry-run=false

# CRD の登録を確認する
kubectl wait --for=condition=established --timeout=60s \
  crd/virtualservices.networking.istio.io \
  crd/destinationrules.networking.istio.io \
  crd/peerauthentications.security.istio.io \
  crd/authorizationpolicies.security.istio.io
```

#### 3-2. CRD インストール後のドライラン

```bash
# Namespace を作成する
kubectl create namespace k1s0-service --dry-run=client -o yaml | kubectl apply -f -

# サーバーサイドドライランでバリデーションする
helm upgrade --install task infra/helm/services/service/task \
  -n k1s0-service \
  -f infra/helm/services/service/task/values-dev.yaml \
  --set image.tag=local \
  --dry-run=server

# cert-manager マニフェストのドライラン（CRD インストール後に実行可能）
kubectl apply --dry-run=client -f infra/kubernetes/cert-manager/

# Istio マニフェストのドライラン（CRD インストール後に実行可能）
kubectl apply --dry-run=client -f infra/istio/

# 検証後にクラスタを削除する
kind delete cluster --name k1s0-local
```

### 4. よくある失敗パターンと対処

| エラー | 原因 | 対処 |
|--------|------|------|
| `Error: unable to build kubernetes objects` | values.yaml の型ミスマッチ | `helm template` でレンダリング結果を確認する |
| `required value "xxx" not set` | 必須 values の未設定 | `values-dev.yaml` に該当フィールドを追加する |
| `template: ... cannot use nil as string` | nil ガード不足 | `_deployment.tpl` の nil チェックパターンを参照する（テンプレート修正時の規約を参照） |
| `unknown field "xxx"` (cert-manager リソース) | cert-manager CRD の未インストール | 3-1 節の cert-manager CRD インストール手順を実行してから再試行する |
| `unknown field "xxx"` (Istio リソース) | Istio CRD の未インストール | 3-1 節の Istio CRD インストール手順を実行してから再試行する |

## ローカル開発・CI/CD での注意事項（HELM-01/HELM-02 監査対応）

### ローカル開発時の Helm 依存関係解決（HELM-02 対応）

`Chart.yaml` の依存クレート `k1s0-common` は本番では `oci://harbor.k1s0.io/helm-charts` から取得するが、
ローカル開発時は `oci://` 参照が外部からアクセスできないため、`file://` 参照に変更してから `helm dependency update` を実行すること。

```bash
# ローカル開発時のみ（コミット禁止）
sed -i 's|oci://harbor.k1s0.io/helm-charts|file://../../../../helm/charts|' \
  infra/helm/services/system/auth/Chart.yaml
helm dependency update infra/helm/services/system/auth/
```

> **注意**: この変更はコミットしないこと。本番 CI/CD では `oci://harbor.k1s0.io/helm-charts` を使用する。
> system / service tier と business tier ではパス深度が異なるため、相対パス数を合わせること。

### CI/CD でのイメージレジストリ・タグ上書き（HELM-01 対応）

`k1s0-common/values.yaml` の `image.registry` はプレースホルダー `harbor.internal.example.com`。
デプロイ時は必ず CI/CD パイプラインから `--set` フラグで上書きすること。

```bash
# CI/CD パイプラインでの Helm デプロイ例
helm upgrade --install auth-server infra/helm/services/system/auth/ \
  --set image.registry=YOUR_REGISTRY \
  --set image.tag=${GIT_SHA} \
  -f infra/helm/services/system/auth/values-prod.yaml
```

`image.tag` を省略すると空文字列になり、`latest` ではなくイメージプルエラーになる可能性がある。
必ず Git SHA またはセマンティックバージョンを指定すること。

## docker-compose との設定乖離に関する注意（LOW-004 対応）

開発環境の設定は `docker-compose.dev.yaml`（ローカル）と `infra/helm/services/*/values-dev.yaml`（K8s dev）の2箇所で管理されている。設定変更時は**両方を同時に更新する**こと。

| 設定項目 | docker-compose 側 | Helm 側 |
|---------|-------------------|----|
| ポート番号 | `docker-compose.yaml` の ports / `.env.dev` の HOST_PORT 変数 | `values-dev.yaml` の `service.port` / `container.port` |
| 環境変数 | `docker-compose.dev.yaml` の environment | `values-dev.yaml` の `env` セクション |
| リソース制限 | `deploy.resources` | `values.yaml` の `resources` |
| ConfigMap | ボリュームマウント（`config/config.dev.yaml`） | `values-dev.yaml` の config 内容 |

**乖離が発生しやすいシナリオ**:
1. `docker-compose.yaml` でポートを変更したが `values-dev.yaml` は旧ポートのまま → K8s dev 環境でルーティング失敗
2. Helm values に新しい設定キーを追加したが docker-compose の config.yaml に反映していない → 起動時のデフォルト値で動作する

**推奨**: docker-compose とHelmのいずれかに設定変更を加えた場合は、PR レビュー時に両方のファイルが更新されていることを確認すること。

## 関連ドキュメント

- [kubernetes設計](kubernetes設計.md)
- [config設計](../../cli/config/config設計.md)
- [Dockerイメージ戦略](../docker/Dockerイメージ戦略.md)
- [認証認可設計](../../architecture/auth/認証認可設計.md)
- [サービスメッシュ設計](../service-mesh/サービスメッシュ設計.md)
- [terraform設計](../terraform/terraform設計.md)
- [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md)
- [API設計](../../architecture/api/API設計.md)
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md)
- [インフラ設計](../overview/インフラ設計.md)
- [可観測性設計](../../architecture/observability/可観測性設計.md)
- [CI-CD設計](../cicd/CI-CD設計.md)
