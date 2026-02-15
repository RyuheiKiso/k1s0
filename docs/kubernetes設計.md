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
| `k1s0-system`     | system 層のサーバー・DB             | system   |
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

```yaml
# k1s0-system: business からのインバウンドのみ許可
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-business
  namespace: k1s0-system
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
# k1s0-business: service からのインバウンドのみ許可
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
              tier: service

---
# k1s0-service: Ingress からのインバウンドのみ許可（他 Tier からの直接アクセス禁止）
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

```yaml
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
| cluster-admin         | クラスタ全体の管理                            | インフラチーム    |
| namespace-admin       | 特定 Namespace 内の全リソース管理             | チームリーダー    |
| developer             | Deployment, Pod, Service の参照・ログ閲覧     | 開発者            |
| readonly              | 全リソースの参照のみ                          | 運用監視          |

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
