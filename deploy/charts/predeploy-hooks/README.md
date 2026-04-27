# deploy/charts/predeploy-hooks — Argo CD PreSync Hook Jobs

Argo CD の `PreSync` Hook で、Wave -5 で宣言された Dapr Component CR の backing store
（Postgres / Kafka / Valkey / MinIO）の Ready 状態を polling 検証する。
Wave 10 で tier1 Pod が起動する前に backing store の利用可能性を確認することで、
「CR は登録されたが実体が起動していない」「DNS は解決するが TCP/HTTP が通らない」状態でアプリ Pod が
起動して接続失敗が連鎖する事故を防ぐ。

詳細設計: [`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md`](../../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md) の
「backing store との接続タイミング」節。

## Job 一覧（4 種）

| ファイル | 検証対象 | image | 検証方法 |
|---|---|---|---|
| `templates/job-postgres-ready.yaml` | Postgres（CNPG） | `postgres:16-alpine` | `pg_isready` |
| `templates/job-kafka-ready.yaml` | Kafka（Strimzi KRaft） | `busybox:1.36` | TCP probe（`nc -z`） |
| `templates/job-valkey-ready.yaml` | Valkey | `valkey/valkey:7.2-alpine` | `valkey-cli ping` |
| `templates/job-minio-ready.yaml` | MinIO | `curlimages/curl:8.10.1` | HTTP `/minio/health/ready` |

各 Job は ServiceAccount `predeploy-hooks` で動作し、`securityContext` で nonroot / readOnlyRootFilesystem /
all capabilities drop / RuntimeDefault seccomp を強制する（Pod Security Standards `restricted` 準拠）。

## Argo CD Hook 動作

```yaml
metadata:
  annotations:
    argocd.argoproj.io/hook: PreSync                         # Sync 直前に実行
    argocd.argoproj.io/hook-delete-policy: HookSucceeded,BeforeHookCreation
    argocd.argoproj.io/sync-wave: "-1"                       # Wave 0 直前
```

`PreSync` Hook は Sync の各サイクルで Argo CD が自動的に再生成する。`HookSucceeded` で成功時削除、
`BeforeHookCreation` で前回 Job 残骸の事前削除を行うことで、再 Sync ごとにクリーンに動作する。

## 適用

```sh
# Argo CD Application から参照（推奨）
# deploy/apps/application-sets/infra.yaml が path: infra/environments/<env>/ で本 chart も含める

# 直接 helm install（手動デプロイ用）
helm install predeploy-hooks deploy/charts/predeploy-hooks \
  --namespace argocd --create-namespace \
  -f deploy/kustomize/overlays/<env>/predeploy-hooks-values.yaml
```

## 環境別 overlay（採用初期で配置）

採用組織が values を環境別に上書きする例:

```yaml
# deploy/kustomize/overlays/dev/predeploy-hooks-values.yaml
job:
  polling:
    intervalSeconds: 2          # dev は短く
    timeoutSeconds: 60          # dev は短時間で諦める
postgres:
  service: cnpg-cluster-rw.k1s0-data-dev.svc.cluster.local
kafka:
  enabled: false                # dev は Kafka 起動しない構成も許容
```

## 関連設計

- [docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md](../../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md) — backing store 接続タイミング
- [ADR-CICD-002](../../../docs/02_構想設計/adr/ADR-CICD-002-argo-rollouts.md) — Argo CD
- [IMP-DIR-OPS-092](../../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md) — ApplicationSet 配置
