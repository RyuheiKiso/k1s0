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
    │   └── auth/                    # system 層のサービス
    │       ├── Chart.yaml
    │       ├── values.yaml          # デフォルト値
    │       ├── values-dev.yaml
    │       ├── values-staging.yaml
    │       ├── values-prod.yaml
    │       └── templates/
    │           ├── deployment.yaml
    │           ├── service.yaml
    │           ├── configmap.yaml
    │           ├── hpa.yaml
    │           └── pdb.yaml
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
    repository: "file://../../charts/k1s0-common"
```

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
  grpcPort: null                       # gRPC 無効（有効時は 50051 を設定）
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
  maxReplicas: 10
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
  runAsUser: 1000
  fsGroup: 1000
containerSecurityContext:
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop: ["ALL"]

# config.yaml のマウント
config:
  mountPath: /etc/app
  data: {}                             # ConfigMap として作成

# Vault Agent Injector
vault:
  enabled: true
  role: "order-service"
  secrets:
    - path: "secret/data/k1s0/order/database"
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
        host: postgres.k1s0-system.svc.cluster.local
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
        host: postgres.k1s0-system.svc.cluster.local
        ssl_mode: require
      observability:
        log:
          level: info
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
        host: postgres.k1s0-system.svc.cluster.local
        ssl_mode: verify-full
        max_open_conns: 50
      observability:
        log:
          level: warn
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
  --set image.tag=1.0.0

# prod 環境
helm upgrade --install order ./infra/helm/services/service/order \
  -n k1s0-service \
  -f ./infra/helm/services/service/order/values-prod.yaml \
  --set image.tag=1.0.0
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
        vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/order/database"
        vault.hashicorp.com/agent-inject-template-db-password: |
          {{`{{ with secret "secret/data/k1s0/order/database" }}{{ .Data.data.password }}{{ end }}`}}
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
# 各サービスの Chart.yaml での依存指定例
dependencies:
  - name: k1s0-common
    version: "~1.2.0"    # 1.2.x の PATCH アップデートを自動追従
    repository: "file://../../charts/k1s0-common"
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
