# SecretProviderClass 未作成サービス一覧

INFRA-003 監査対応（2026-03-31）: ai-agent-secrets.yaml / ai-gateway-secrets.yaml を作成。
全27サービスの SecretProviderClass が揃った。

## 対応済み（27サービス）

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
| bff-proxy | bff-proxy-secrets.yaml | 2026-03-29 |
| graphql-gateway | graphql-gateway-secrets.yaml | 2026-03-29 |
| api-registry | api-registry-secrets.yaml | 2026-03-29 |
| app-registry | app-registry-secrets.yaml | 2026-03-29 |
| event-monitor | event-monitor-secrets.yaml | 2026-03-29 |
| event-store | event-store-secrets.yaml | 2026-03-29 |
| file | file-secrets.yaml | 2026-03-29 |
| master-maintenance | master-maintenance-secrets.yaml | 2026-03-29 |
| navigation | navigation-secrets.yaml | 2026-03-29 |
| policy | policy-secrets.yaml | 2026-03-29 |
| quota | quota-secrets.yaml | 2026-03-29 |
| ratelimit | ratelimit-secrets.yaml | 2026-03-29 |
| rule-engine | rule-engine-secrets.yaml | 2026-03-29 |
| search | search-secrets.yaml | 2026-03-29 |
| service-catalog | service-catalog-secrets.yaml | 2026-03-29 |
| ai-agent | ai-agent-secrets.yaml | 2026-03-31 |
| ai-gateway | ai-gateway-secrets.yaml | 2026-03-31 |

## 未作成（0サービス）

全サービスの SecretProviderClass 作成完了。

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
