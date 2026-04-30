# infra/gitops/local-stack — Argo CD 動作検証用ローカル git server

production の k1s0 GitOps は GitHub Enterprise / GitLab を前提（採用組織側の git
運用方針による）だが、kind ローカル kind cluster 環境では Argo CD が読み取れる
Repo を立てる必要がある。本マニフェストは gitea を最小構成（emptyDir / sqlite3 /
no redis）で deploy する。

## 構成

- gitea 1.22-rootless 1 Pod (emptyDir)
- admin: argocd / ArgoCD123!
- repo: argocd/k1s0 (任意 push 可)
- service: gitea.gitops.svc.cluster.local:3000

## Argo CD 連携

`infra/gitops/local-stack/argocd-repo-secret.yaml` を Argo CD に apply 後、
AppProject の `spec.sourceRepos` に `http://gitea.gitops.svc.cluster.local:3000/argocd/k1s0.git`
を追加する。本ディレクトリの README に書かれた手順をそのまま実行すれば、
GitHub への push を介さずに ApplicationSet が Synced/Healthy になる。

## 検証

- 2026-04-30: tier1-facade ApplicationSet が dev/staging/prod 3 環境で
  **Synced + Healthy** を確認済（kubectl get applications -n argocd）。
