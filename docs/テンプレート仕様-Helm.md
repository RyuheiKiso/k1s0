# テンプレート仕様 — Helm Chart

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で **server** 種別を選択した際に、サーバーコードと同時に生成される **Helm Chart テンプレート** を定義する。Helm Chart は Kubernetes 上へのデプロイに必要なマニフェスト一式（Deployment, Service, ConfigMap, HPA, PDB）と環境別 values ファイルを自動生成する。

### 対象

- **kind = server のみ** — client, library, database では Helm Chart を生成しない
- サーバーの言語（Go / Rust）に依存しない共通テンプレート

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
            └── pdb.yaml.tera
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

> Helm Chart テンプレートは全ファイル常に生成される。条件分岐（gRPC / DB / Kafka / Redis）は values.yaml 内の値と Helm テンプレート構文（`{{ if }}`）で制御する。

## 条件付き生成

Helm Chart のテンプレートファイル自体は全て生成されるが、**values.yaml に注入される設定値** がテンプレート変数の条件に応じて変化する。

| テンプレート変数       | 条件                    | 影響範囲                                                     |
| ---------------------- | ----------------------- | ------------------------------------------------------------ |
| `api_style`            | `grpc` を含む           | `container.grpcPort` / `service.grpcPort` に `50051` を設定  |
| `has_database`         | `true`                  | `vault.secrets` に DB パスワードパスを追加、config.data に database セクション追加 |
| `has_kafka`            | `true`                  | `kafka.enabled: true`、`kafka.brokers` にデフォルト値を設定  |
| `has_redis`            | `true`                  | `redis.enabled: true`、`redis.host` にデフォルト値を設定     |

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
{% else %}
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
{% if has_kafka %}
  brokers: []
{% else %}
  brokers: []
{% endif %}

redis:
  enabled: {{ has_redis }}
{% if has_redis %}
  host: ""
{% else %}
  host: ""
{% endif %}
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

## テンプレート変数一覧

Helm Chart テンプレートで使用する変数の一覧。全変数の定義は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

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
    └── pdb.yaml
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

## 関連ドキュメント

- [helm設計](helm設計.md) --- Helm Chart の設計・Library Chart・values 設計の詳細
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) --- テンプレート変数の定義・Tera 構文リファレンス
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) --- サーバーテンプレートの詳細
- [kubernetes設計](kubernetes設計.md) --- Kubernetes クラスタ設計・ラベル規約
- [config設計](config設計.md) --- アプリケーション設定ファイルの設計
- [Dockerイメージ戦略](Dockerイメージ戦略.md) --- Docker イメージのビルド・レジストリ戦略
- [認証認可設計](認証認可設計.md) --- Vault Agent Injector・シークレット管理
- [CI-CD設計](CI-CD設計.md) --- デプロイパイプラインとの連携
