# Kafka SASL_SSL 設定手順書

本ドキュメントは k1s0 プロジェクトにおける Kafka の SASL_SSL 設定について説明する。
開発環境（PLAINTEXT）と本番環境（SASL_SSL）の違いを整理し、
Strimzi Operator を使用した本番環境向け設定手順を提供する。

## 環境別プロトコル対応表

| 環境 | プロトコル | 設定方法 | 備考 |
|------|----------|---------|------|
| ローカル開発 | PLAINTEXT | `docker-compose.yaml` | 開発効率優先。証明書管理不要 |
| dev（Kubernetes） | SASL_SSL | Strimzi Operator | 本番同等の設定を早期検証 |
| staging（Kubernetes） | SASL_SSL | Strimzi Operator | 本番前の最終検証環境 |
| prod（Kubernetes） | SASL_SSL | Strimzi Operator | 通信の暗号化・認証を強制 |

---

## ローカル開発環境（PLAINTEXT）

`docker-compose.yaml` で起動するローカル Kafka は **開発専用** の PLAINTEXT 設定である。
本番環境への適用は禁止する。

```yaml
# docker-compose.yaml（抜粋）
# NOTE: ローカル開発では PLAINTEXT を使用（開発効率優先）。
# staging/prod では SASL_SSL を使用し、Strimzi Operator が証明書管理を行う。
kafka:
  image: apache/kafka:3.8.0
  environment:
    KAFKA_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
    KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
    KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
```

ローカル開発時は各サービスの config.yaml の `security_protocol` を
`PLAINTEXT` に上書きするか、環境変数 `KAFKA_SECURITY_PROTOCOL=PLAINTEXT` で
オーバーライドすること。

---

## 本番環境（SASL_SSL）

### Strimzi Operator を使用した SASL_SSL 設定

本番環境の Kafka は Strimzi Operator（`kafka.strimzi.io`）で管理する。
Strimzi は TLS 証明書の自動生成・ローテーション、SASL 認証の設定を Kubernetes ネイティブに提供する。

#### 1. Kafka クラスター定義（`infra/kubernetes/system/kafka-cluster.yaml`）

```yaml
apiVersion: kafka.strimzi.io/v1beta2
kind: Kafka
metadata:
  # k1s0 システム tier の Kafka クラスター定義
  name: k1s0-kafka
  namespace: k1s0-system
spec:
  kafka:
    replicas: 3
    listeners:
      # 内部通信（Pod 間）: TLS + SCRAM-SHA-512 認証
      - name: tls
        port: 9093
        type: internal
        tls: true
        authentication:
          type: scram-sha-512
      # 外部公開なし（ClusterIP のみ）
    config:
      # 通信の暗号化を強制（平文接続を拒否する）
      inter.broker.protocol: TLS
      ssl.protocol: TLSv1.3
    storage:
      type: jbod
      volumes:
        - id: 0
          type: persistent-claim
          size: 100Gi
          deleteClaim: false
  zookeeper:
    replicas: 3
    storage:
      type: persistent-claim
      size: 10Gi
      deleteClaim: false
```

#### 2. KafkaUser 定義（サービスごとにユーザーを作成する）

```yaml
apiVersion: kafka.strimzi.io/v1beta2
kind: KafkaUser
metadata:
  # auth サービス用 Kafka ユーザー（SCRAM-SHA-512 認証）
  name: auth-kafka-user
  namespace: k1s0-system
  labels:
    strimzi.io/cluster: k1s0-kafka
spec:
  authentication:
    type: scram-sha-512
  authorization:
    type: simple
    acls:
      # auth サービスが publish するトピックへの書き込み権限
      - resource:
          type: topic
          name: k1s0.system.auth.login.v1
          patternType: literal
        operations:
          - Write
          - Describe
      # auth サービスが subscribe するコンシューマーグループ権限
      - resource:
          type: group
          name: auth-server.default
          patternType: literal
        operations:
          - Read
```

Strimzi は `KafkaUser` リソースを作成すると、認証情報（ユーザー名・パスワード）を
Kubernetes Secret として自動生成する。Secret 名は `{KafkaUser.metadata.name}` となる。

#### 3. TLS 証明書の取得

Strimzi が管理する TLS 証明書は以下の Secret に格納される:

```bash
# Kafka クラスターの CA 証明書を取得する
kubectl get secret k1s0-kafka-cluster-ca-cert -n k1s0-system \
  -o jsonpath='{.data.ca\.crt}' | base64 -d > /tmp/kafka-ca.crt
```

---

## 各サービスの config.yaml での切り替え方法

各サービスは `config/config.yaml` の `kafka.security_protocol` フィールドで
接続プロトコルを設定する。

### SASL_SSL 設定例（本番・staging・dev 共通）

```yaml
# config.yaml（本番環境向け設定）
kafka:
  brokers:
    - "k1s0-kafka-kafka-bootstrap.k1s0-system.svc.cluster.local:9093"
  consumer_group: "auth-server.default"
  # セキュリティ: 本番環境では SASL_SSL で暗号化通信を強制する
  security_protocol: "SASL_SSL"
  # SASL 認証方式: Strimzi は SCRAM-SHA-512 を使用する
  sasl_mechanism: "SCRAM-SHA-512"
  # TLS 証明書・SASL 認証情報は Kubernetes Secret 経由で注入する
  # 以下は環境変数またはファイルマウントで上書きすること
  sasl_username: "${KAFKA_SASL_USERNAME}"
  sasl_password: "${KAFKA_SASL_PASSWORD}"
  ssl_ca_location: "/etc/kafka/certs/ca.crt"
```

### PLAINTEXT 設定例（ローカル開発専用）

```yaml
# config.yaml（ローカル開発環境向け設定）
# 警告: この設定は開発専用であり、本番環境への適用は禁止する
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "auth-server.default"
  security_protocol: "PLAINTEXT"
```

### Helm values による環境別切り替え

`infra/helm/services/{tier}/{service}/values-{env}.yaml` で環境ごとの接続先を管理する:

```yaml
# values-prod.yaml（抜粋）
kafka:
  bootstrapServers: "k1s0-kafka-kafka-bootstrap.k1s0-system.svc.cluster.local:9093"
  securityProtocol: SASL_SSL
  saslMechanism: SCRAM-SHA-512
  # Strimzi が生成した Secret を参照する
  saslSecretName: "auth-kafka-user"
  tlsSecretName: "k1s0-kafka-cluster-ca-cert"
```

---

## Kubernetes Deployment への証明書・認証情報マウント

Strimzi が生成した Secret を Pod にマウントする例:

```yaml
# Deployment（抜粋）
spec:
  template:
    spec:
      containers:
        - name: auth
          env:
            # SCRAM-SHA-512 認証情報を Secret から注入する
            - name: KAFKA_SASL_USERNAME
              valueFrom:
                secretKeyRef:
                  name: auth-kafka-user
                  key: sasl.jaas.config
            - name: KAFKA_SASL_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: auth-kafka-user
                  key: password
          volumeMounts:
            # Kafka CA 証明書を読み取り専用でマウントする
            - name: kafka-ca-cert
              mountPath: /etc/kafka/certs
              readOnly: true
      volumes:
        - name: kafka-ca-cert
          secret:
            secretName: k1s0-kafka-cluster-ca-cert
            items:
              - key: ca.crt
                path: ca.crt
```

---

## docker-compose.yaml の PLAINTEXT 設定について

`docker-compose.yaml` に記述されている Kafka 設定は **ローカル開発専用** である。

- `KAFKA_LISTENERS: PLAINTEXT://:9092` — 平文接続（開発のみ）
- `KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092` — コンテナ内部向けアドレス

本番・staging・dev の Kubernetes 環境には適用されない。
Kubernetes 環境では Strimzi Operator が管理する SASL_SSL 設定が強制的に適用される。

---

## トラブルシューティング

### 証明書エラーが発生する場合

```bash
# Strimzi が生成した CA 証明書の有効期限を確認する
kubectl get secret k1s0-kafka-cluster-ca-cert -n k1s0-system \
  -o jsonpath='{.data.ca\.crt}' | base64 -d | openssl x509 -noout -dates
```

### SASL 認証エラーが発生する場合

```bash
# KafkaUser の状態を確認する（Ready になっているか確認する）
kubectl get kafkauser auth-kafka-user -n k1s0-system -o yaml
```

---

## 関連ドキュメント

- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) — Kafka トピック設計・スキーマ管理
- [kubernetes設計.md](kubernetes/kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) — 認証・認可設計全体
