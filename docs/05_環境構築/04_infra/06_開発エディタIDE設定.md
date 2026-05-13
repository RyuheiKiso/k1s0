---
id: env.infra.infra_editor_ide
axis: infra
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.infra.infra_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code に Kubernetes / YAML / GitLens の拡張を導入し、Lens Desktop で cluster リソースを可視化できる状態を infra 軸の IDE 検収条件とする。

## VS Code 拡張のインストール

```bash
code --install-extension ms-kubernetes-tools.vscode-kubernetes-tools
code --install-extension redhat.vscode-yaml
code --install-extension eamodio.gitlens
code --install-extension ms-azuretools.vscode-docker
```

| 拡張 | 用途 |
|---|---|
| vscode-kubernetes-tools | kubectl / helm / kustomize 操作 / manifest オートコンプリート |
| vscode-yaml | YAML バリデーション・スキーマ補完（k8s CRD 対応） |
| GitLens | Git blame / diff / history 強化 |
| vscode-docker | Dockerfile / docker-compose 補完 |

## VS Code の settings.json（infra 推奨設定）

`.vscode/settings.json` に以下を追加する（リポジトリには追加しない、手元の User settings に追加すること）。

```json
{
  "yaml.schemas": {
    "kubernetes": "**/*.yaml"
  },
  "editor.formatOnSave": true,
  "[yaml]": {
    "editor.insertSpaces": true,
    "editor.tabSize": 2
  }
}
```

## Lens Desktop のインストール

Lens Desktop（https://k8slens.dev/）は k8s cluster を GUI で管理するツール。Windows 側にインストールし、`~/.kube/config` を読み込ませる。

```bash
# kind cluster 起動後に kubeconfig をマージ
kind get kubeconfig --name k1s0-local > ~/.kube/config
# Lens から localhost:6443 への接続を確認
```

## Argo CD UI へのアクセス

kind cluster に Argo CD をインストール後、port-forward でローカルアクセスする。

```bash
kubectl port-forward svc/argocd-server -n argocd 8080:443
# ブラウザで https://localhost:8080 にアクセス
```

## 検収コマンド

```bash
code --list-extensions | grep -E "kubernetes|yaml|gitlens"
# 3 拡張が一覧に含まれること
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
