# 03. Helm charts 配置

本ファイルは `deploy/charts/` 配下の共通 Helm chart 配置を確定する。tier1 / tier2 / tier3 の類似アプリで再利用可能な chart を集約する。

## 共通 chart 化の目的

各サービスが個別に Deployment / Service / HPA / ServiceMonitor / PodDisruptionBudget を記述すると、同型の YAML が 50 個超に膨張する。共通 chart で template 化することで以下を実現する。

- ベストプラクティス（securityContext / resources / probes）の強制
- 監視（ServiceMonitor）、PDB、HPA の自動付与
- 新サービス追加時のコスト最小化（values.yaml 1 個で起動）

## レイアウト

```
deploy/charts/
├── README.md
├── tier1-facade/               # Go Dapr facade 用 chart
│   ├── Chart.yaml
│   ├── values.yaml             # デフォルト values（prod ベース）
│   ├── values-dev.yaml         # dev 環境 override
│   ├── values-staging.yaml     # staging 環境 override
│   ├── values-prod.yaml        # prod 明示 override（通常は values.yaml と同等、空でも可）
│   ├── templates/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── servicemonitor.yaml
│   │   ├── hpa.yaml
│   │   ├── pdb.yaml
│   │   ├── networkpolicy.yaml
│   │   └── _helpers.tpl
│   └── tests/
├── tier1-rust-service/         # Rust 自作領域用 chart
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── values-dev.yaml
│   ├── values-staging.yaml
│   ├── values-prod.yaml
│   └── templates/              # tier1-facade と同様
├── tier2-dotnet-service/
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── values-dev.yaml
│   ├── values-staging.yaml
│   ├── values-prod.yaml
│   └── templates/
├── tier2-go-service/
│   └── ...
├── tier3-web-app/              # React (Vite) 汎用 chart
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── values-dev.yaml
│   ├── values-staging.yaml
│   ├── values-prod.yaml
│   └── templates/
└── tier3-bff/
    └── ...
```

## 環境別 values の運用

各 chart 配下に `values.yaml`（共通基準）と `values-<env>.yaml`（環境固有 override）を同梱する。`infra/environments/<env>/` の overlay は infra 専用で、deploy 側の chart は chart 配下に環境ファイルを寄せることで以下を得る。

- ArgoCD ApplicationSet の `valueFiles:` が相対パス `values-{{env}}.yaml` のみで完結し、他ディレクトリへの `../` traversal が発生しない
- CODEOWNERS が chart 単位で `tier1-rust` / `tier2-dev` / `tier3-web` に紐付き、環境別の override 変更も正しくレビュー担当に届く
- `helm lint` / `helm template --values=values.yaml --values=values-<env>.yaml` を CI に組み込むときの参照パスが一意

## chart の共通構造

各 chart は同じ templates/ 構造を持ち、技術スタック固有の微調整のみ異なる。

### Deployment

```yaml
# templates/deployment.yaml（tier1-facade の例）
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "tier1-facade.fullname" . }}
  labels:
    {{- include "tier1-facade.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "tier1-facade.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      annotations:
        dapr.io/enabled: "true"
        dapr.io/app-id: "{{ .Values.daprAppId }}"
        dapr.io/app-port: "{{ .Values.service.port }}"
      labels:
        {{- include "tier1-facade.selectorLabels" . | nindent 8 }}
    spec:
      serviceAccountName: {{ include "tier1-facade.serviceAccountName" . }}
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        fsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: http
              containerPort: {{ .Values.service.port }}
            - name: grpc
              containerPort: {{ .Values.service.grpcPort }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          livenessProbe:
            httpGet:
              path: /healthz
              port: http
          readinessProbe:
            httpGet:
              path: /ready
              port: http
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
```

### HPA / PDB / ServiceMonitor / NetworkPolicy

HPA は CPU / memory 基準＋KEDA の event-driven（Kafka lag 等）。PDB は最小 2 Pod（HA）。ServiceMonitor は Prometheus / Mimir が scrape。NetworkPolicy は default-deny からアプリ単位で allow-list。

## values.yaml のデフォルト

```yaml
# tier1-facade/values.yaml
replicaCount: 3

image:
  repository: harbor.k1s0.internal/tier1/facade
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  grpcPort: 9090

daprAppId: tier1-facade

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

pdb:
  enabled: true
  minAvailable: 2

serviceMonitor:
  enabled: true
  interval: 15s

networkPolicy:
  enabled: true
  allowIngress:
    - podSelector:
        matchLabels:
          app.kubernetes.io/part-of: k1s0-tier1
```

各サービス（例: tier1-facade-service-invoke）は `deploy/apps/application-sets/tier1.yaml` が values.yaml を override する。

## tier1 / tier2 / tier3 の chart 差異

- **tier1-facade**: Dapr annotation 自動注入
- **tier1-rust-service**: Dapr 不使用（grpc 直接）、scratch image 前提（securityContext 厳格）
- **tier2-dotnet-service**: .NET ヘルスチェック endpoint（/health/live、/health/ready）
- **tier2-go-service**: Go 標準 /healthz
- **tier3-web-app**: React (Vite) ビルド成果物を nginx:alpine で静的配信、PORT: 8080 デフォルト、ingress Gateway HTTPRoute 同梱
- **tier3-bff**: GraphQL /graphql + REST エンドポイント、認可 middleware 前提

## chart の test

各 chart には `templates/tests/` で Helm test を定義。chart 変更時に kind cluster で helm install → helm test で検証する CI を `.github/workflows/ci-helm-charts.yml` に設置する（運用蓄積後）。

## 対応 IMP-DIR ID

- IMP-DIR-OPS-093（Helm charts 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-004（Helm 採用）
- DX-CICD-\* / NFR-C-NOP-\*
