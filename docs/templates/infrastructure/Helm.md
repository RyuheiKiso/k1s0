# テンプレート仕様 — Helm Chart

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で **server** 種別を選択した際に、サーバーコードと同時に生成される **Helm Chart テンプレート** を定義する。Helm Chart は Kubernetes 上へのデプロイに必要なマニフェスト一式（Deployment, Service, ConfigMap, HPA, PDB）と環境別 values ファイルを自動生成する。

### 対象

- **kind = server のみ** — client, library, database では Helm Chart を生成しない
- サーバーの言語（Go / Rust）に依存しない共通テンプレート
- **GraphQL BFF**: service Tier で BFF を生成する場合も、BFF は通常のサーバーと同じ扱いで個別の Helm Chart が生成される。BFF 用の Chart は `infra/helm/services/service/{service_name}-bff/` に配置される

### 生成条件

| kind     | 生成 |
| -------- | ---- |
| server   | ✓    |
| client   | ---  |
| library  | ---  |
| database | ---  |

## 配置パス

生成された Helm Chart は `infra/helm/services/` 配下に Tier 別のパスで配置される。

| Tier     | 配置パス                                                     |
| -------- | ------------------------------------------------------------ |
| system   | `infra/helm/services/system/{service_name}/`                 |
| business | `infra/helm/services/business/{domain}/{service_name}/`      |
| service  | `infra/helm/services/service/{service_name}/`                |

例:

| tier       | domain       | service_name | 配置パス                                                 |
| ---------- | ------------ | ------------ | -------------------------------------------------------- |
| `service`  | ---          | `order`      | `infra/helm/services/service/order/`                     |
| `system`   | ---          | `auth`       | `infra/helm/services/system/auth/`                       |
| `business` | `accounting` | `ledger`     | `infra/helm/services/business/accounting/ledger/`        |

## テンプレートファイル一覧

テンプレートファイルは `CLI/templates/helm/` 配下に配置する。

```
CLI/
└── templates/
    └── helm/
        ├── Chart.yaml.tera
        ├── values.yaml.tera
        ├── values-dev.yaml.tera
        ├── values-staging.yaml.tera
        ├── values-prod.yaml.tera
        └── templates/
            ├── deployment.yaml.tera
            ├── service.yaml.tera
            ├── configmap.yaml.tera
            ├── hpa.yaml.tera
            ├── pdb.yaml.tera
            └── ingress.yaml.tera
```

| ファイル                       | 説明               | 条件       |
| ------------------------------ | ------------------ | ---------- |
| `Chart.yaml.tera`             | Chart 定義         | 常に生成   |
| `values.yaml.tera`            | デフォルト値       | 常に生成   |
| `values-dev.yaml.tera`        | 開発環境オーバーライド | 常に生成   |
| `values-staging.yaml.tera`    | ステージング環境オーバーライド | 常に生成   |
| `values-prod.yaml.tera`       | 本番環境オーバーライド | 常に生成   |
| `templates/deployment.yaml.tera` | Deployment マニフェスト | 常に生成 |
| `templates/service.yaml.tera`    | Service マニフェスト   | 常に生成 |
| `templates/configmap.yaml.tera`  | ConfigMap マニフェスト | 常に生成 |
| `templates/hpa.yaml.tera`       | HPA マニフェスト      | 常に生成 |
| `templates/pdb.yaml.tera`       | PDB マニフェスト      | 常に生成 |
| `templates/ingress.yaml.tera`   | Ingress マニフェスト  | 常に生成 |

> Helm Chart テンプレートは全ファイル常に生成される。条件分岐（gRPC / DB / Kafka / Redis）は values.yaml 内の値と Helm テンプレート構文（`{{ if }}`）で制御する。

## 条件付き生成

Helm Chart のテンプレートファイル自体は全て生成されるが、**values.yaml に注入される設定値** がテンプレート変数の条件に応じて変化する。

| テンプレート変数       | 条件                    | 影響範囲                                                     |
| ---------------------- | ----------------------- | ------------------------------------------------------------ |
| `api_styles`           | `"grpc"` を含む         | `container.grpcPort` / `service.grpcPort` に `50051` を設定  |
| `has_database`         | `true`                  | `vault.secrets` に DB パスワードパスを追加、config.data に database セクション追加 |
| `has_kafka`            | `true`                  | `vault.secrets` に Kafka シークレットパスを追加、`kafka.enabled: true` |
| `has_redis`            | `true`                  | `vault.secrets` に Redis シークレットパスを追加、`redis.enabled: true` |

### 複数 API 方式選択時のポートマッピング

REST と gRPC を同時に選択した場合（`api_styles` に `"rest"` と `"grpc"` の両方が含まれる）、Deployment と Service で以下のポート構成となる。

| ポート名 | ポート番号 | 条件 | 用途 |
|----------|-----------|------|------|
| `http` | 8080 (container) / 80 (service) | 常に設定 | REST API / ヘルスチェック |
| `grpc` | 50051 (container) / 50051 (service) | `api_styles` に `"grpc"` を含む場合 | gRPC API |

#### values.yaml での設定

REST のみの場合:
```yaml
container:
  port: 8080
  grpcPort: null

service:
  type: ClusterIP
  port: 80
  grpcPort: null
```

REST + gRPC の場合:
```yaml
container:
  port: 8080
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051
```

#### Library Chart での処理

`k1s0-common` Library Chart の `_deployment.tpl` と `_service.tpl` は、`grpcPort` が `null` でない場合にのみ gRPC ポートを追加する。

Deployment の ports セクション（`_deployment.tpl`）:
```yaml
ports:
  - name: http
    containerPort: {{ .Values.container.port }}
    protocol: TCP
  {{- if .Values.container.grpcPort }}
  - name: grpc
    containerPort: {{ .Values.container.grpcPort }}
    protocol: TCP
  {{- end }}
```

Service の ports セクション（`_service.tpl`）:
```yaml
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
```

gRPC を含む場合のヘルスチェックは、`probes.grpcHealthCheck.enabled` の値に応じて切り替わる。デフォルトは HTTP ポート (`/healthz`) で行い、gRPC ヘルスチェックを有効にした場合は gRPC ネイティブプローブを使用する。詳細は [gRPC ヘルスチェック仕様](#grpc-ヘルスチェック仕様) を参照。

> **注記**: Helm の Go テンプレートでは YAML の `null` 値に対して `if` 条件は `false` と評価される。したがって `grpcPort: null` の場合 `{{- if .Values.container.grpcPort }}` は偽となり、gRPC ポート定義は出力されない。

## Tera / Helm 構文衝突の回避

Helm テンプレートは `{{ .Values.xxx }}` という Go テンプレート構文を使用する。Tera も `{{ }}` を変数展開に使用するため、構文が衝突する。この問題は Tera の `{% raw %}` / `{% endraw %}` ブロックで解決する。

### 方針

- **Tera が処理する部分**（テンプレート変数の展開）は通常の `{{ }}` で記述する
- **Helm が処理する部分**（Kubernetes マニフェスト内の Go テンプレート構文）は `{% raw %}` ... `{% endraw %}` で囲む

### 構文の使い分け

```tera
{# Tera 変数展開（CLI 実行時に解決） #}
name: {{ service_name }}
description: {{ service_name_pascal }} service

{# Helm テンプレート構文（helm install 実行時に解決） #}
{% raw %}
metadata:
  labels:
    app.kubernetes.io/name: {{ include "k1s0-common.name" . }}
    app.kubernetes.io/instance: {{ .Release.Name }}
spec:
  replicas: {{ .Values.replicaCount }}
{% endraw %}
```

### 混在する場合の記述例

Tera 変数と Helm 構文が同一ファイル内で交互に出現する場合は、セクションごとに `{% raw %}` / `{% endraw %}` を配置する。

```tera
apiVersion: v2
name: {{ service_name }}
description: {{ service_name_pascal }} service
type: application
version: 0.1.0
appVersion: "1.0.0"

{% raw %}
dependencies:
  - name: k1s0-common
    version: "~0.1.0"
{% endraw %}
    repository: "file://{{ common_chart_relative_path }}"
```

## テンプレート変数マッピング

Helm Chart テンプレートで使用するテンプレート変数と、生成される values.yaml フィールドの対応を示す。

### Chart.yaml への変数マッピング

| Tera 変数              | Chart.yaml フィールド   | 例                       |
| ---------------------- | ----------------------- | ------------------------ |
| `service_name`         | `name`                  | `order`                  |
| `service_name_pascal`  | `description` の一部    | `Order service`          |

### values.yaml への変数マッピング

| Tera 変数              | values.yaml フィールド            | 導出ルール                                            | 例                             |
| ---------------------- | --------------------------------- | ----------------------------------------------------- | ------------------------------ |
| `docker_registry`      | `image.registry`                  | 固定値                                                | `harbor.internal.example.com`  |
| `docker_project`       | `image.repository` のプレフィクス | `k1s0-{tier}`                                         | `k1s0-service`                 |
| `service_name`         | `image.repository` のサフィクス   | ---                                                   | `order`                        |
| ---                    | `image.repository`（結合値）      | `{docker_project}/{service_name}`                     | `k1s0-service/order`           |
| `api_style` / `api_styles` | `container.grpcPort`         | grpc を含む場合 `50051`、それ以外 `null`              | `50051` or `null`              |
| `api_style` / `api_styles` | `service.grpcPort`           | grpc を含む場合 `50051`、それ以外 `null`              | `50051` or `null`              |
| `tier`                 | `labels.tier`                     | そのまま                                              | `service`                      |
| `tier`                 | `vault.role`                      | そのまま                                              | `service`                      |
| `tier`                 | `vault.secrets[].path` の一部     | ---                                                   | `secret/data/k1s0/service/...` |
| `service_name`         | `vault.secrets[].path` の一部     | ---                                                   | `.../order/database`           |
| `has_database`         | `vault.secrets` の有無            | `true` 時のみ DB シークレットパスを追加               | ---                            |
| `has_kafka`            | `vault.secrets` の有無            | `true` 時に Kafka シークレットパスを追加              | ---                            |
| `has_redis`            | `vault.secrets` の有無            | `true` 時に Redis シークレットパスを追加              | ---                            |
| `has_kafka`            | `kafka.enabled`                   | そのまま                                              | `true` / `false`               |
| `has_redis`            | `redis.enabled`                   | そのまま                                              | `true` / `false`               |

### Library Chart 相対パスの導出

`dependencies[].repository` の相対パスは Tier に応じて変わる。

| Tier     | Chart.yaml からの相対パス                       |
| -------- | ----------------------------------------------- |
| system   | `file://../../../charts/k1s0-common`            |
| service  | `file://../../../charts/k1s0-common`            |
| business | `file://../../../../charts/k1s0-common`         |

## テンプレート詳細

### Chart.yaml.tera

```tera
apiVersion: v2
name: {{ service_name }}
description: {{ service_name_pascal }} service
type: application
version: 0.1.0
appVersion: "1.0.0"

dependencies:
  - name: k1s0-common
    version: "~0.1.0"
{% if tier == "business" %}
    repository: "file://../../../../charts/k1s0-common"
{% else %}
    repository: "file://../../../charts/k1s0-common"
{% endif %}
```

### values.yaml.tera

```tera
nameOverride: ""
fullnameOverride: ""

image:
  registry: {{ docker_registry }}
  repository: {{ docker_project }}/{{ service_name }}
  tag: ""
  pullPolicy: IfNotPresent

imagePullSecrets:
  - name: harbor-pull-secret

replicaCount: 2

container:
  port: 8080
{% if api_styles is containing("grpc") %}
  grpcPort: 50051
{% else %}
  grpcPort: null
{% endif %}
  command: []
  args: []

resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi

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

service:
  type: ClusterIP
  port: 80
{% if api_styles is containing("grpc") %}
  grpcPort: 50051
{% else %}
  grpcPort: null
{% endif %}

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

pdb:
  enabled: true
  minAvailable: 1

ingress:
  enabled: false
  ingressClassName: nginx

podSecurityContext:
  runAsNonRoot: true
  runAsUser: 65532
  fsGroup: 65532
containerSecurityContext:
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop: ["ALL"]

config:
  mountPath: /etc/app
  data: {}

vault:
  enabled: true
  role: "{{ tier }}"
  secrets:
{% if has_database %}
    - path: "secret/data/k1s0/{{ tier }}/{{ service_name }}/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
{% endif %}
{% if has_kafka %}
    - path: "secret/data/k1s0/{{ tier }}/{{ service_name }}/kafka"
      key: "password"
      mountPath: "/vault/secrets/kafka-password"
{% endif %}
{% if has_redis %}
    - path: "secret/data/k1s0/{{ tier }}/{{ service_name }}/redis"
      key: "password"
      mountPath: "/vault/secrets/redis-password"
{% endif %}
{% if not has_database and not has_kafka and not has_redis %}
    []
{% endif %}

nodeSelector: {}
tolerations: []
affinity: {}

serviceAccount:
  create: true
  name: ""
  annotations: {}

labels:
  tier: {{ tier }}

kafka:
  enabled: {{ has_kafka }}
  brokers: []

redis:
  enabled: {{ has_redis }}
  host: ""
```

### values-dev.yaml.tera

```tera
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
  enabled: false

config:
  data:
    config.yaml: |
      app:
        environment: dev
{% if has_database %}
      database:
        host: postgres.k1s0-{{ tier }}.svc.cluster.local
        ssl_mode: disable
{% endif %}
      observability:
        log:
          level: debug
          format: text
        trace:
          sample_rate: 1.0
```

### values-staging.yaml.tera

```tera
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
{% if has_database %}
      database:
        host: postgres.k1s0-{{ tier }}.svc.cluster.local
        ssl_mode: require
{% endif %}
      observability:
        log:
          level: info
        trace:
          sample_rate: 0.5
```

### values-prod.yaml.tera

```tera
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

{% raw %}
affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
            - key: app.kubernetes.io/name
              operator: In
              values:
{% endraw %}
                - {{ service_name }}
{% raw %}
        topologyKey: kubernetes.io/hostname
{% endraw %}

config:
  data:
    config.yaml: |
      app:
        environment: prod
{% if has_database %}
      database:
        host: postgres.k1s0-{{ tier }}.svc.cluster.local
        ssl_mode: verify-full
        max_open_conns: 50
{% endif %}
      observability:
        log:
          level: warn
        trace:
          sample_rate: 0.1
```

### templates/deployment.yaml.tera

Deployment マニフェストは Library Chart（k1s0-common）の `_deployment.tpl` を呼び出す形式で生成する。

```tera
{% raw %}
{{- include "k1s0-common.deployment" . }}
{% endraw %}
```

> Library Chart が Deployment の全構成（レプリカ数、コンテナ設定、ヘルスチェック、ボリュームマウント、セキュリティコンテキスト、gRPC ポート条件分岐）を提供するため、各サービスの deployment.yaml は Library Chart の呼び出しのみとなる。

### templates/service.yaml.tera

```tera
{% raw %}
{{- include "k1s0-common.service" . }}
{% endraw %}
```

### templates/configmap.yaml.tera

```tera
{% raw %}
{{- include "k1s0-common.configmap" . }}
{% endraw %}
```

### templates/hpa.yaml.tera

```tera
{% raw %}
{{- include "k1s0-common.hpa" . }}
{% endraw %}
```

### templates/pdb.yaml.tera

```tera
{% raw %}
{{- include "k1s0-common.pdb" . }}
{% endraw %}
```

### templates/ingress.yaml.tera

Ingress マニフェストは Library Chart（k1s0-common）の `_ingress.tpl` を呼び出す形式で生成する。`ingress.enabled: true` の場合にのみ Kubernetes Ingress リソースが出力される。

```tera
{% raw %}
{{- include "k1s0-common.ingress" . }}
{% endraw %}
```

> Library Chart が Ingress の全構成（metadata、annotations、TLS、ルーティングルール）を提供するため、各サービスの ingress.yaml は Library Chart の呼び出しのみとなる。

## Ingress 詳細仕様

### 概要

Ingress は Kong API Gateway 経由でない管理系 UI や内部ダッシュボードなどに対して、Kubernetes Ingress リソースを通じた直接的なルーティングを提供する。通常の API サービスは Kong 経由でルーティングされるため、デフォルトでは `ingress.enabled: false` となる。

### values.yaml の Ingress 設定

| フィールド | 型 | デフォルト | 説明 |
|---|---|---|---|
| `ingress.enabled` | boolean | `false` | Ingress リソースの生成を制御 |
| `ingress.ingressClassName` | string | `"nginx"` | 使用する Ingress Controller のクラス名（`nginx` または `kong`） |
| `ingress.annotations` | map | `{}` | Ingress リソースに付与するアノテーション |
| `ingress.hosts` | list | `[]` | ホストベースのルーティング定義 |
| `ingress.hosts[].host` | string | --- | ホスト名（例: `admin.example.com`） |
| `ingress.hosts[].paths` | list | --- | パスベースのルーティング定義 |
| `ingress.hosts[].paths[].path` | string | --- | URL パス（例: `/`） |
| `ingress.hosts[].paths[].pathType` | string | `"Prefix"` | パスマッチタイプ（`Prefix` / `Exact` / `ImplementationSpecific`） |
| `ingress.hosts[].paths[].backend.serviceName` | string | --- | バックエンドサービス名 |
| `ingress.hosts[].paths[].backend.servicePort` | integer | --- | バックエンドサービスポート番号 |
| `ingress.tls` | list | `[]` | TLS 設定 |
| `ingress.tls[].secretName` | string | --- | TLS 証明書の Secret 名 |
| `ingress.tls[].hosts` | list | --- | TLS を適用するホスト名リスト |

### YAML 設定例

デフォルト（Ingress 無効）:

```yaml
ingress:
  enabled: false
  ingressClassName: nginx
```

Ingress 有効時の設定例:

```yaml
ingress:
  enabled: true
  ingressClassName: nginx
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
  hosts:
    - host: admin.example.com
      paths:
        - path: /
          pathType: Prefix
          backend:
            serviceName: admin-ui
            servicePort: 80
  tls:
    - secretName: admin-tls
      hosts:
        - admin.example.com
```

### Library Chart での処理

`k1s0-common` Library Chart の `_ingress.tpl` は、`ingress.enabled` が `true` の場合にのみ Ingress リソースを出力する。

```yaml
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
```

## gRPC ヘルスチェック仕様

### 概要

gRPC サービスに対して、HTTP エンドポイント (`/healthz`) ではなく gRPC ネイティブのヘルスチェックプロトコル（[gRPC Health Checking Protocol](https://github.com/grpc/grpc/blob/master/doc/health-checking.md)）を使用できる。Kubernetes 1.24 以降で利用可能な `grpc` プローブを使用する。

### values.yaml の gRPC ヘルスチェック設定

| フィールド | 型 | デフォルト | 説明 |
|---|---|---|---|
| `probes.grpcHealthCheck.enabled` | boolean | `false` | gRPC ネイティブヘルスチェックの有効化 |
| `probes.grpcHealthCheck.port` | string | `"grpc"` | gRPC ヘルスチェックで使用するポート名 |

### YAML 設定例

デフォルト（HTTP ヘルスチェック）:

```yaml
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
  grpcHealthCheck:
    enabled: false
    port: "grpc"
```

gRPC ヘルスチェック有効時:

```yaml
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
  grpcHealthCheck:
    enabled: true
    port: "grpc"
```

> `probes.liveness` / `probes.readiness` のHTTP定義は values.yaml 上に残るが、`grpcHealthCheck.enabled: true` の場合は Library Chart の `_deployment.tpl` が gRPC プローブで上書きする。

### Library Chart での条件分岐ロジック

`_deployment.tpl` は `probes.grpcHealthCheck.enabled` の値に応じてヘルスチェック方式を切り替える。

#### grpcHealthCheck.enabled が false の場合（デフォルト）

既存の HTTP ベースのプローブをそのまま使用する:

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: http
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3
readinessProbe:
  httpGet:
    path: /readyz
    port: http
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

#### grpcHealthCheck.enabled が true の場合

gRPC ネイティブプローブを使用する:

```yaml
livenessProbe:
  grpc:
    port: 50051
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3
readinessProbe:
  grpc:
    port: 50051
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

#### `_deployment.tpl` の条件分岐

```yaml
{{- if and .Values.probes .Values.probes.grpcHealthCheck .Values.probes.grpcHealthCheck.enabled }}
          livenessProbe:
            grpc:
              port: {{ .Values.container.grpcPort }}
            initialDelaySeconds: {{ .Values.probes.liveness.initialDelaySeconds }}
            periodSeconds: {{ .Values.probes.liveness.periodSeconds }}
            failureThreshold: {{ .Values.probes.liveness.failureThreshold }}
          readinessProbe:
            grpc:
              port: {{ .Values.container.grpcPort }}
            initialDelaySeconds: {{ .Values.probes.readiness.initialDelaySeconds }}
            periodSeconds: {{ .Values.probes.readiness.periodSeconds }}
            failureThreshold: {{ .Values.probes.readiness.failureThreshold }}
{{- else }}
          {{- with .Values.probes.liveness }}
          livenessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.probes.readiness }}
          readinessProbe:
            {{- toYaml . | nindent 12 }}
          {{- end }}
{{- end }}
```

### サーバーテンプレートの gRPC Health スタブ

gRPC ヘルスチェックを利用するには、サーバー側で `grpc.health.v1.Health` サービスを実装する必要がある。CLI でサーバーテンプレートを生成する際、`api_styles` に `"grpc"` が含まれる場合は、gRPC Health Checking Protocol のスタブが自動的に含まれる。

- **Go サーバー**: `google.golang.org/grpc/health` パッケージの `health.NewServer()` を gRPC サーバーに登録
- **Rust サーバー**: `tonic-health` クレートの `health_reporter()` を gRPC サーバーに追加

これにより、`grpcHealthCheck.enabled: true` に切り替えるだけで gRPC ネイティブヘルスチェックが機能する。

> **サーバーテンプレートとの相互参照**: gRPC Health Check のサーバー側スケルトンコード（`grpc_health.go.tera` / `grpc_health.rs.tera`）は [テンプレート仕様-サーバー](../server/サーバー.md) の「gRPC Health Check テンプレート」セクションで定義されている。values.yaml の `probes.grpcHealthCheck.enabled` と `probes.grpcHealthCheck.port` の設定値がサーバー側の gRPC Health サービス実装と対応する。

## テンプレート変数一覧

Helm Chart テンプレートで使用する変数の一覧。全変数の定義は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型           | Helm Chart での用途                              |
| -------------------- | ------------ | ------------------------------------------------ |
| `service_name`       | String       | Chart 名、image.repository、Vault パス、affinity |
| `service_name_pascal`| String       | Chart description                                |
| `tier`               | String       | Vault ロール、ラベル、Vault シークレットパス     |
| `domain`             | String       | business Tier の配置パス                         |
| `api_style`          | String       | gRPC ポート設定（後方互換）                      |
| `api_styles`         | Vec\<String> | gRPC ポート設定（複数 API 方式対応）             |
| `has_database`       | bool         | DB シークレット・config セクションの生成制御     |
| `has_kafka`          | bool         | Kafka 設定の有効化                               |
| `has_redis`          | bool         | Redis 設定の有効化                               |
| `docker_registry`    | String       | image.registry                                   |
| `docker_project`     | String       | image.repository のプレフィクス                  |

## 生成後の配置先構造

テンプレートから生成された Helm Chart は、以下のディレクトリ構造で配置される。

```
infra/helm/services/{tier}/{service_name}/          # system / service
infra/helm/services/business/{domain}/{service_name}/  # business
├── Chart.yaml
├── values.yaml
├── values-dev.yaml
├── values-staging.yaml
├── values-prod.yaml
└── templates/
    ├── deployment.yaml
    ├── service.yaml
    ├── configmap.yaml
    ├── hpa.yaml
    ├── pdb.yaml
    └── ingress.yaml
```

## デプロイコマンド

生成された Helm Chart は以下のコマンドでデプロイする。

```bash
# dev 環境
helm upgrade --install {service_name} ./infra/helm/services/{tier}/{service_name} \
  -n k1s0-{tier} \
  -f ./infra/helm/services/{tier}/{service_name}/values-dev.yaml \
  --set image.tag={commit_hash}

# staging 環境
helm upgrade --install {service_name} ./infra/helm/services/{tier}/{service_name} \
  -n k1s0-{tier} \
  -f ./infra/helm/services/{tier}/{service_name}/values-staging.yaml \
  --set image.tag={commit_hash}

# prod 環境
helm upgrade --install {service_name} ./infra/helm/services/{tier}/{service_name} \
  -n k1s0-{tier} \
  -f ./infra/helm/services/{tier}/{service_name}/values-prod.yaml \
  --set image.tag={commit_hash}
```

## GraphQL API パス設定

GraphQL エンドポイントを Ingress 経由で公開する場合、`/query` パスへのルーティングを設定する。

### Ingress ルーティング設定

BFF の GraphQL エンドポイント（`/query`）を Ingress 経由で公開する values.yaml の設定例:

```yaml
ingress:
  enabled: true
  ingressClassName: nginx
  annotations:
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
  hosts:
    - host: api.example.com
      paths:
        - path: /query
          pathType: Exact
          backend:
            serviceName: {{ service_name }}-bff
            servicePort: 80
        - path: /healthz
          pathType: Exact
          backend:
            serviceName: {{ service_name }}-bff
            servicePort: 80
```

GraphQL エンドポイントは通常 `POST /query` のみであるため、`pathType: Exact` を使用する。ヘルスチェックエンドポイント（`/healthz`）も外部からアクセス可能にする場合は同様に追加する。

> **注記**: 通常の API サービスは Kong API Gateway 経由でルーティングされるため Ingress は不要だが、BFF は管理 UI やダッシュボードから直接アクセスされるケースがあるため、Ingress を有効化する場合がある。

---

## BFF 用 Helm Chart 差分

BFF は通常のサーバーと同じ Helm Chart テンプレートを使用するが、以下の点が異なる。

### BFF Chart の特徴

| 項目 | 通常サーバー | BFF |
|---|---|---|
| 配置パス | `infra/helm/services/{tier}/{service_name}/` | `infra/helm/services/service/{service_name}-bff/` |
| Vault シークレット | DB/Kafka/Redis パスを含む | 空配列（`[]`） |
| upstream 設定 | なし | `config.data` に upstream URL を含む |
| gRPC ポート | `api_styles` による | `null`（GraphQL over HTTP のみ） |
| DB 関連設定 | `has_database` による | 常になし |

### BFF 用 values.yaml の差分

通常の values.yaml.tera との差分:

```yaml
# BFF 固有の設定
container:
  port: 8080
  grpcPort: null          # BFF は HTTP のみ

service:
  type: ClusterIP
  port: 80
  grpcPort: null

# DB / Kafka / Redis は不使用
vault:
  enabled: false          # BFF は Vault 不要
  secrets: []

kafka:
  enabled: false
  brokers: []

redis:
  enabled: false
  host: ""

# upstream 設定を config に含める
config:
  data:
    config.yaml: |
      server:
        port: 8080
        name: {{ service_name }}-bff
      upstream:
        http_url: http://{{ service_name }}:8080
        grpc_address: {{ service_name }}:50051
```

### BFF Chart の生成条件

BFF の Helm Chart は、サーバー本体の Helm Chart と同時に生成される。CLI の `build_output_path()` において、BFF 用のパスは `{service_name}-bff` サフィックスで構築される。

---

## 関連ドキュメント

- [helm設計](../../infrastructure/kubernetes/helm設計.md) --- Helm Chart の設計・Library Chart・values 設計の詳細
- [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) --- テンプレート変数の定義・Tera 構文リファレンス
- [テンプレート仕様-サーバー](../server/サーバー.md) --- サーバーテンプレートの詳細
- [テンプレート仕様-BFF](../client/BFF.md) --- BFF テンプレートの詳細
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) --- Kubernetes クラスタ設計・ラベル規約
- [config設計](../../cli/config/config設計.md) --- アプリケーション設定ファイルの設計
- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) --- Docker イメージのビルド・レジストリ戦略
- [認証認可設計](../../auth/design/認証認可設計.md) --- Vault Agent Injector・シークレット管理
- [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md) --- デプロイパイプラインとの連携
