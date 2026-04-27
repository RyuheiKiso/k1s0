# deploy/kustomize/base/

全環境共通の Kubernetes リソース（Namespace / 共通 label）を `infra/k8s/namespaces/`
から取り込み、共通 label `k1s0.io/managed-by=argo-cd` を全リソースに付与する。

overlays/<env>/ から bases として参照される。
