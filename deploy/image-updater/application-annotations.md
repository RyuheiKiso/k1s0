# Argo CD Application への annotation 設定例

`deploy/apps/application-sets/<chart>.yaml` の `template.metadata.annotations:` に
以下のような annotation を追加し、Image Updater が image を追跡 / 書換する範囲を指定する。

## tier1-facade（Go ファサード 3 Pod、prod は semver pin）

```yaml
metadata:
  annotations:
    # 追跡する image を一覧する（カンマ区切り）。エイリアス=registry/repo:tag-constraint。
    argocd-image-updater.argoproj.io/image-list: |
      tier1-state=ghcr.io/k1s0/k1s0/tier1-state,
      tier1-secret=ghcr.io/k1s0/k1s0/tier1-secret,
      tier1-workflow=ghcr.io/k1s0/k1s0/tier1-workflow

    # 各エイリアスの update strategy
    argocd-image-updater.argoproj.io/tier1-state.update-strategy: semver
    argocd-image-updater.argoproj.io/tier1-state.allow-tags: "regexp:^[0-9]+\\.[0-9]+\\.[0-9]+$"
    argocd-image-updater.argoproj.io/tier1-secret.update-strategy: semver
    argocd-image-updater.argoproj.io/tier1-secret.allow-tags: "regexp:^[0-9]+\\.[0-9]+\\.[0-9]+$"
    argocd-image-updater.argoproj.io/tier1-workflow.update-strategy: semver
    argocd-image-updater.argoproj.io/tier1-workflow.allow-tags: "regexp:^[0-9]+\\.[0-9]+\\.[0-9]+$"

    # Helm values への書き戻し（kustomize overlay の image.tag を更新する）
    argocd-image-updater.argoproj.io/write-back-method: git
    argocd-image-updater.argoproj.io/write-back-target: kustomization
    argocd-image-updater.argoproj.io/git-branch: main
```

## 環境別 strategy

| 環境 | annotation | 例 |
|---|---|---|
| dev | `update-strategy: digest` + `allow-tags: latest` | `latest` の digest 変化を毎分追従 |
| staging | `update-strategy: newest-build` + `allow-tags: rc-.*` | rc tag の最新を追従 |
| prod | `update-strategy: semver` + `allow-tags: ^[0-9]+\.[0-9]+\.[0-9]+$` | semver patch 追従 |

## Git write-back と Renovate の責任分界（ADR-CICD-003）

| 責務 | Image Updater | Renovate |
|---|---|---|
| Container image tag 追跡 | ✅ | ✗ |
| Helm chart version 追跡 | ✗ | ✅ |
| Go module / NPM / Cargo の依存 | ✗ | ✅ |
| GitHub Actions version 追跡 | ✗ | ✅ |
| Argo CD ApplicationSet 内の Helm chart targetRevision | ✗ | ✅ |

両者は重複せず、棲み分けで運用する。
