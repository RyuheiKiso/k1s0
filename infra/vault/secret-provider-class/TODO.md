# SecretProviderClass 未作成サービス一覧

M-22 監査対応: 残り16サービスの SecretProviderClass が未作成。
対応優先度は低いが、本番稼働前に全サービス分の作成が必要。

## 対応済み（10サービス）

| サービス | ファイル | 対応日 |
|---------|---------|--------|
| auth | auth-secrets.yaml | 2026-03-22 |
| config | config-secrets.yaml | 2026-03-22 |
| dlq-manager | dlq-manager-secrets.yaml | 2026-03-22 |
| saga | saga-secrets.yaml | 2026-03-22 |
| notification | notification-secrets.yaml | 2026-03-29 |
| tenant | tenant-secrets.yaml | 2026-03-29 |
| featureflag | featureflag-secrets.yaml | 2026-03-29 |
| session | session-secrets.yaml | 2026-03-29 |
| workflow | workflow-secrets.yaml | 2026-03-29 |
| scheduler | scheduler-secrets.yaml | 2026-03-29 |

## 未作成（16サービス）

各サービスは auth-secrets.yaml のパターンに従って作成すること。
必要なシークレットは `infra/vault/policies/{service}.hcl` を参照。

| サービス | 対応するポリシーファイル | 主なシークレット | 優先度 |
|---------|----------------------|----------------|--------|
| bff-proxy | bff-proxy.hcl | api-key, redis, keycloak-oidc | 高 |
| graphql-gateway | graphql-gateway.hcl | api-key, db-password, kafka-sasl | 高 |
| ai-agent | ai-agent.hcl | api-key, db-password, kafka-sasl | 中 |
| ai-gateway | ai-gateway.hcl | api-key, db-password, kafka-sasl | 中 |
| api-registry | api-registry.hcl | api-key, db-password, kafka-sasl | 中 |
| app-registry | app-registry.hcl | api-key, db-password, kafka-sasl | 中 |
| event-monitor | event-monitor.hcl | api-key, db-password, kafka-sasl | 中 |
| event-store | event-store.hcl | api-key, db-password, kafka-sasl | 中 |
| file | file.hcl | api-key, db-password, kafka-sasl | 中 |
| master-maintenance | master-maintenance.hcl | api-key, db-password, kafka-sasl | 中 |
| navigation | navigation.hcl | api-key, db-password, kafka-sasl | 中 |
| policy | policy.hcl | api-key, db-password, kafka-sasl | 中 |
| quota | quota.hcl | api-key, db-password, kafka-sasl | 中 |
| ratelimit | ratelimit.hcl | api-key, db-password, kafka-sasl | 中 |
| rule-engine | rule-engine.hcl | api-key, db-password, kafka-sasl | 中 |
| search | search.hcl | api-key, db-password, kafka-sasl | 中 |
| service-catalog | service-catalog.hcl | api-key, db-password, kafka-sasl | 中 |

## 作成テンプレート

```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  # {service} サーバーが Vault から取得するシークレット定義。
  name: {service}-server-vault-secrets
  namespace: k1s0-system
  labels:
    app.kubernetes.io/name: {service}-server
    tier: system
spec:
  provider: vault
  parameters:
    vaultAddress: "https://vault.vault.svc.cluster.local:8200"
    roleName: "{service}-server"
    objects: |
      - objectName: "api-key"
        secretPath: "secret/data/k1s0/system/{service}/api-key"
        secretKey: "key"
      - objectName: "db-password"
        secretPath: "secret/data/k1s0/system/{service}/database"
        secretKey: "password"
      - objectName: "kafka-sasl-username"
        secretPath: "secret/data/k1s0/system/kafka/sasl"
        secretKey: "username"
      - objectName: "kafka-sasl-password"
        secretPath: "secret/data/k1s0/system/kafka/sasl"
        secretKey: "password"
```
