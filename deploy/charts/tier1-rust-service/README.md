# tier1-rust-service Helm chart

tier1 Rust core 3 Pod（decision / audit / pii）を Kubernetes に配備する Helm chart。

## デプロイ

```sh
helm install tier1-rust deploy/charts/tier1-rust-service \
  --namespace tier1-facade \
  --create-namespace \
  --set image.tag=v0.1.0
```

prod では Argo CD ApplicationSet（plan 06-XX）から適用する。

## 構造

3 Pod を `pods.{decision,audit,pii}` で個別 enable / disable / replica 上書き可能。
`templates/deployment.yaml` が `range` で展開し、各 Pod に独立した Deployment / Service が生成される。

## 関連設計

- DS-SW-COMP-008（t1-decision）/ 007（t1-audit）/ 009（t1-pii）
- ADR-TIER1-001（Go + Rust hybrid）
- ADR-CICD-001 / ADR-CICD-002
