# deploy/image-updater — Argo CD Image Updater 設定

Argo CD Image Updater を導入し、GHCR（ghcr.io/k1s0/k1s0/...）へ push された
新規 image tag / digest を Application に自動反映する。

## 役割

GitOps の文脈で「image tag を Git 上の values.yaml に書き戻す」ことで、Argo CD の
desired state と一致させる。Image Updater が毎分 GHCR を polling し、policy に
合致した新 tag を検出したら、`deploy/kustomize/overlays/<env>/<chart>-values.yaml`
の `image.tag` を git commit + push する。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | argocd-image-updater Helm chart の values（registry / git write-back / log level） |
| `registries.conf.yaml` | registry 別 credentials / API endpoint 設定 |
| `application-annotations.md` | ApplicationSet への annotation 追加例（参考、実適用は ApplicationSet 側） |

## デプロイ

```sh
helm repo add argo https://argoproj.github.io/argo-helm
helm upgrade --install argocd-image-updater argo/argocd-image-updater \
  -n argocd -f deploy/image-updater/values.yaml \
  --version 0.13.0
```

## 環境別の更新ポリシー

| 環境 | 戦略 | 例 |
|---|---|---|
| dev | `latest` を毎 push 追従（digest pin） | `update-strategy: digest` |
| staging | `rc.*` の最新を追従 | `update-strategy: newest-build`、constraint: `^.*-rc\..*$` |
| prod | semver の patch 自動追従、major/minor 手動 | `update-strategy: semver`、constraint: `~0.1` |

実際の policy は `deploy/apps/application-sets/<chart>.yaml` の annotation で個別設定する。
詳細は `application-annotations.md`。

## Git write-back

Image Updater が values.yaml を更新する際の commit user / SSH key は OpenBao 経由で
注入する（plan 04-06 で動的シークレット化）。本リリース時点 では Kubernetes Secret に
SSH key を直接置く運用（dev のみ）。

## 関連設計

- ADR-CICD-001（Argo CD ApplicationSet）
- ADR-CICD-003（Renovate + Image Updater の責任分界、本リリース時点 暫定）
- IMP-REL-* — リリース設計
