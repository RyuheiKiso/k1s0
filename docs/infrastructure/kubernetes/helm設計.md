# Helm 設計

k1s0 のアプリケーションデプロイに使用する Helm Chart の設計を定義する。

## 基本方針

- サービスごとに個別の Helm Chart を作成する
- 共通設定は Library Chart として切り出し、各 Chart で参照する
- 環境別の差分は `values-{env}.yaml` で管理する
- シークレットは Vault Agent Injector で Pod に注入する

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
    │   └── kong/                    # Kong API Gateway
    │       ├── Chart.yaml
    │       ├── values.yaml
    │       ├── values-dev.yaml
    │       ├── values-staging.yaml
    │       ├── values-prod.yaml
    │       └── templates/
    ├── business/
    │   └── accounting/
    │       └── ledger/
    │           ├── Chart.yaml
    │           ├── values.yaml
    │           ├── values-dev.yaml
    │           ├── values-staging.yaml
    │           ├── values-prod.yaml
    │           └── templates/
    └── service/
        └── order/
            ├── Chart.yaml
            ├── values.yaml
            ├── values-dev.yaml
            ├── values-staging.yaml
            ├── values-prod.yaml
            └── templates/
```

## System Tier Chart 一覧

system tier には以下の 6 つの Chart が存在する。全て `k1s0-common` Library Chart に依存し、`labels.tier: system` を設定する。

| Chart | 説明 | 言語 | gRPC | Kafka | Redis | Vault secrets |
| --- | --- | --- | --- | --- | --- | --- |
| auth | 認証・認可（JWT 検証、ユーザー管理） | Rust | 50051 | ✓ | - | DB パスワード |
| config | 構成管理（サービス設定の集中管理） | Rust | 50051 | - | - | DB パスワード |
| saga | Saga オーケストレータ（分散トランザクション） | Rust | 50051 | 有効 | - | DB パスワード |
| dlq-manager | Dead Letter Queue 管理（失敗メッセージの再処理） | Rust | - | 有効 | - | DB パスワード |
| bff-proxy | BFF プロキシ（OIDC 認証、セッション管理、リバースプロキシ） | Go | - | - | 有効 | OIDC client secret, Redis パスワード |
| kong | API Gateway（DB-backed PostgreSQL モード） | - | - | - | - | DB パスワード（SecretKeyRef） |

### 実ファイル配置

各 Chart は `infra/helm/services/system/` 配下に配置されている。

| Chart | Chart.yaml パス | values.yaml パス |
|-------|----------------|-----------------|
| auth | `infra/helm/services/system/auth/Chart.yaml` | `infra/helm/services/system/auth/values.yaml` |
| config | `infra/helm/services/system/config/Chart.yaml` | `infra/helm/services/system/config/values.yaml` |
| saga | `infra/helm/services/system/saga/Chart.yaml` | `infra/helm/services/system/saga/values.yaml` |
| dlq-manager | `infra/helm/services/system/dlq-manager/Chart.yaml` | `infra/helm/services/system/dlq-manager/values.yaml` |
| bff-proxy | `infra/helm/services/system/bff-proxy/Chart.yaml` | `infra/helm/services/system/bff-proxy/values.yaml` |
| kong | `infra/helm/services/system/kong/Chart.yaml` | `infra/helm/services/system/kong/values.yaml` |

全 Chart は `k1s0-common` Library Chart に依存し、`appVersion: "0.1.0"`（kong は `"3.8.0"`）。

### 各 Chart の values.yaml 重要フィールド差分

| フィールド | auth | config | saga | dlq-manager | bff-proxy | kong |
|-----------|------|--------|------|-------------|-----------|------|
| `container.grpcPort` | 50051 | 50051 | 50051 | null | - | - |
| `kafka.enabled` | true | false | true | true | - | - |
| `redis.enabled` | false | false | false | false | true | - |
| `autoscaling.maxReplicas` | 5 | 5 | 5 | 5 | 10 | - |
| `resources.requests.cpu` | 250m | 250m | 250m | 250m | 100m | 500m |
| `resources.requests.memory` | 256Mi | 256Mi | 256Mi | 256Mi | 128Mi | 512Mi |
| `vault.secrets` | DB | DB | DB | DB | OIDC + Redis | SecretKeyRef |

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
  interval: 15s
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
helm upgrade --install order ./infra/helm/services/service/order \
  -n k1s0-service \
  -f ./infra/helm/services/service/order/values-dev.yaml \
  --set image.tag=1.0.0-a1b2c3d

# staging 環境
helm upgrade --install order ./infra/helm/services/service/order \
  -n k1s0-service \
  -f ./infra/helm/services/service/order/values-staging.yaml \
  --set image.tag=1.0.0-a1b2c3d

# prod 環境
helm upgrade --install order ./infra/helm/services/service/order \
  -n k1s0-service \
  -f ./infra/helm/services/service/order/values-prod.yaml \
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
