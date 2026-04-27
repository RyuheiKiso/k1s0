# deploy/rollouts — Argo Rollouts canary / analysis

ADR-CICD-002 に従い、Progressive Delivery を Argo Rollouts で実現する。

## ファイル

| ディレクトリ | 役割 |
|---|---|
| `canary-strategies/` | canary 戦略テンプレート（25 → 50 → 100% の 3 段階） |
| `analysis-templates/` | Mimir（Prometheus 互換）クエリベースの自動評価（error rate / latency p99） |
| `experiments/` | 採用後の運用拡大時に追加（A/B 比較実験定義） |

## デプロイ

```sh
# Argo Rollouts controller インストール（必要に応じて helm chart に切替）
kubectl create namespace argo-rollouts
kubectl apply -n argo-rollouts -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml

# AnalysisTemplate / canary strategy を適用
kubectl apply -f deploy/rollouts/canary-strategies/canary-25-50-100.yaml
kubectl apply -f deploy/rollouts/analysis-templates/error-rate.yaml
kubectl apply -f deploy/rollouts/analysis-templates/latency-p99.yaml
```

## Rollout 適用例

各 chart の Deployment を Rollout に置換える際は、以下を参考に:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
spec:
  strategy:
    canary:
      steps:
        - setWeight: 25
        - pause: { duration: 5m }
        - analysis:
            templates:
              - templateName: error-rate
              - templateName: latency-p99
            args:
              - name: service-name
                value: tier1-state
        - setWeight: 50
        - pause: { duration: 5m }
        # ...
```

## 関連設計

- [ADR-CICD-002](../../docs/02_構想設計/adr/ADR-CICD-002-argo-rollouts.md)
- IMP-REL-* — リリース設計
