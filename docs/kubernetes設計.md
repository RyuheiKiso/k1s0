# Kubernetes 設計

オンプレミス Kubernetes クラスタのリソース設計を定義する。

## 基本方針

- Namespace を Tier アーキテクチャの階層に対応させる
- NetworkPolicy で階層間の依存方向を強制する
- リソースリミットを全 Pod に設定し、ノイジーネイバーを防止する
- HPA でトラフィックに応じたオートスケールを行う

## Namespace 設計

| Namespace         | 対象                                | Tier     |
| ----------------- | ----------------------------------- | -------- |
| `k1s0-system`     | system 層のサーバー・DB・Schema Registry | system   |
| `k1s0-business`   | business 層のサーバー・クライアント・DB | business |
| `k1s0-service`    | service 層のサーバー・クライアント・DB | service  |
| `observability`   | Prometheus, Grafana, Jaeger, Loki   | infra    |
| `messaging`       | Kafka クラスタ                      | infra    |
| `ingress`         | Nginx Ingress Controller            | infra    |
| `service-mesh`    | Istio Control Plane                 | infra    |
| `cert-manager`    | 証明書管理                          | infra    |
| `harbor`          | コンテナレジストリ（同一クラスタの場合） | infra |

## NetworkPolicy

Tier アーキテクチャの依存ルール（下位 → 上位の一方向のみ）を NetworkPolicy で強制する。

**通信方針:**
- 下位 Tier から上位 Tier（system）への依存は全 Tier で許可する（認証・config 取得等の共通基盤へのアクセスは全 Tier から必要なため）
- 同一 Namespace 内の通信は各 Tier で許可する（tier-architecture.md の同一 Tier 例外規定に基づく）
- 異なる Tier 間の同階層直接依存は禁止（tier-architecture.md の例外規定を除く）
- k1s0-system の NetworkPolicy では system 自身・business・service の全 Tier からのインバウンドを許可している

```yaml
# k1s0-system: business および service からのインバウンドを許可
# service Tier から system Tier への直接通信を許可する
# （認証・config取得等の共通基盤へのアクセスは全 Tier から必要）
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-business-and-service
  namespace: k1s0-system
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              tier: system
        - namespaceSelector:
            matchLabels:
              tier: business
        - namespaceSelector:
            matchLabels:
              tier: service

---
# k1s0-business: service および同一 Namespace からのインバウンドを許可
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-service
  namespace: k1s0-business
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              tier: business
        - namespaceSelector:
            matchLabels:
              tier: service

---
# k1s0-service: Ingress および同一 Namespace からのインバウンドを許可
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-ingress
  namespace: k1s0-service
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              tier: service
        - namespaceSelector:
            matchLabels:
              app.kubernetes.io/name: ingress-nginx
```

## リソースリミット

### サーバー Pod

| 環境    | requests CPU | requests Memory | limits CPU | limits Memory |
| ------- | ------------ | --------------- | ---------- | ------------- |
| dev     | 100m         | 128Mi           | 500m       | 512Mi         |
| staging | 250m         | 256Mi           | 1000m      | 1Gi           |
| prod    | 500m         | 512Mi           | 2000m      | 2Gi           |

### クライアント Pod（Nginx）

| 環境    | requests CPU | requests Memory | limits CPU | limits Memory |
| ------- | ------------ | --------------- | ---------- | ------------- |
| dev     | 50m          | 64Mi            | 200m       | 256Mi         |
| staging | 100m         | 128Mi           | 500m       | 512Mi         |
| prod    | 200m         | 256Mi           | 1000m      | 1Gi           |

### データベース Pod

| 環境    | requests CPU | requests Memory | limits CPU | limits Memory |
| ------- | ------------ | --------------- | ---------- | ------------- |
| dev     | 250m         | 256Mi           | 1000m      | 1Gi           |
| staging | 500m         | 1Gi             | 2000m      | 4Gi           |
| prod    | 1000m        | 2Gi             | 4000m      | 8Gi           |

### LimitRange

各 Namespace にデフォルトのリソースリミットを設定する。

```yaml
apiVersion: v1
kind: LimitRange
metadata:
  name: default-limits
  namespace: k1s0-service
spec:
  limits:
    - default:
        cpu: "1"
        memory: 1Gi
      defaultRequest:
        cpu: 250m
        memory: 256Mi
      type: Container
```

### ResourceQuota

Namespace 単位でリソースの上限を設定する。

| Namespace       | requests.cpu | requests.memory | limits.cpu | limits.memory | pods  |
| --------------- | ------------ | --------------- | ---------- | ------------- | ----- |
| k1s0-system     | 8            | 16Gi            | 16         | 32Gi          | 50    |
| k1s0-business   | 16           | 32Gi            | 32         | 64Gi          | 100   |
| k1s0-service    | 8            | 16Gi            | 16         | 32Gi          | 50    |

```yaml
# k1s0-system
apiVersion: v1
kind: ResourceQuota
metadata:
  name: namespace-quota
  namespace: k1s0-system
spec:
  hard:
    requests.cpu: "8"
    requests.memory: 16Gi
    limits.cpu: "16"
    limits.memory: 32Gi
    pods: "50"
    persistentvolumeclaims: "20"
---
# k1s0-business
apiVersion: v1
kind: ResourceQuota
metadata:
  name: namespace-quota
  namespace: k1s0-business
spec:
  hard:
    requests.cpu: "16"
    requests.memory: 32Gi
    limits.cpu: "32"
    limits.memory: 64Gi
    pods: "100"
    persistentvolumeclaims: "40"
---
# k1s0-service
apiVersion: v1
kind: ResourceQuota
metadata:
  name: namespace-quota
  namespace: k1s0-service
spec:
  hard:
    requests.cpu: "8"
    requests.memory: 16Gi
    limits.cpu: "16"
    limits.memory: 32Gi
    pods: "50"
    persistentvolumeclaims: "20"
```

## HPA（Horizontal Pod Autoscaler）

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: order-server
  namespace: k1s0-service
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: order-server
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Pods
          value: 2
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Pods
          value: 1
          periodSeconds: 120
```

### 環境別 HPA 設定

| 環境    | minReplicas | maxReplicas | 備考               |
| ------- | ----------- | ----------- | ------------------ |
| dev     | 1           | 2           | コスト最小化       |
| staging | 2           | 5           | スケール検証       |
| prod    | 2           | 10          | トラフィックに追従 |

## PodDisruptionBudget

prod 環境ではメンテナンス時の可用性を保証する。

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: order-server
  namespace: k1s0-service
spec:
  minAvailable: 1
  selector:
    matchLabels:
      app: order-server
```

## RBAC

| Role                  | 権限                                          | 対象ユーザー      |
| --------------------- | --------------------------------------------- | ----------------- |
| k1s0-admin            | クラスタ全体の管理                            | インフラチーム    |
| k1s0-operator         | リソースの作成・更新・削除を含む運用操作      | 運用チーム        |
| k1s0-developer        | Deployment, Pod, Service の参照・ログ閲覧     | 開発者            |
| readonly              | 全リソースの参照のみ                          | 運用監視          |

### ClusterRole 定義

```yaml
# 開発者用 Role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: k1s0-developer
rules:
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["get", "list", "watch"]
---
# 運用者用 Role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: k1s0-operator
rules:
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps", "secrets"]
    verbs: ["get", "list", "watch", "create", "update", "delete"]
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
# インフラ管理者用 Role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: k1s0-admin
rules:
  - apiGroups: ["*"]
    resources: ["*"]
    verbs: ["*"]
```

## Ingress 設計

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: k1s0-ingress
  namespace: ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "internal-ca"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - "*.k1s0.internal.example.com"
      secretName: k1s0-tls
  rules:
    - host: api.k1s0.internal.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: kong-proxy
                port:
                  number: 80
    - host: grafana.k1s0.internal.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: grafana
                port:
                  number: 3000
```

## StorageClass

StorageClass の定義は terraform設計.md の Ceph CSI モジュール（`modules/ceph/`）で管理する。
Kubernetes 上では以下の 3 種類の StorageClass を使用する。

| StorageClass 名     | アクセスモード | 用途                             |
| ------------------- | -------------- | -------------------------------- |
| `ceph-block`        | RWO            | 一般用途（ログ、キャッシュ等）   |
| `ceph-filesystem`   | RWX            | 共有ストレージ（複数 Pod 間共有）|
| `ceph-block-fast`   | RWO（SSD）     | データベース用（高 IOPS 要求）   |

- PVC を作成する際は用途に応じて適切な StorageClass を指定する
- データベース Pod には `ceph-block-fast` を使用し、SSD による低レイテンシを確保する
- 詳細な設定パラメータは terraform設計.md の `modules/ceph/` セクションを参照

## ラベル規約

すべての Kubernetes リソースに以下のラベルを付与する。

| ラベル                         | 値の例         | 用途                 |
| ------------------------------ | -------------- | -------------------- |
| `app.kubernetes.io/name`       | order-server   | アプリケーション名   |
| `app.kubernetes.io/version`    | 1.2.3          | バージョン           |
| `app.kubernetes.io/component`  | server         | コンポーネント種別   |
| `app.kubernetes.io/part-of`    | k1s0           | プロジェクト名       |
| `app.kubernetes.io/managed-by` | helm           | 管理ツール           |
| `tier`                         | service        | Tier 階層            |
