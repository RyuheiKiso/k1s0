---
id: env.ops.platform_operator_editor_ide
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.platform_operator_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- プラットフォーム運営者の作業は CLI 中心であり、GUI IDE は必須ではない。runbook 編集には VS Code（任意）を使い、Yubikey manager GUI（ykman-gui）は PIV スロット設定の補助として任意で導入する。

## CLI 中心の作業環境

プラットフォーム運営者の主要作業はすべて terminal で完結する。

```bash
# OpenBao 操作
bao operator init -key-shares=5 -key-threshold=3

# kubectl 操作
kubectl get nodes -A
kubectl --as=system:masters get pods -A  # break-glass 例（dry-run のみ）

# audit log 確認
bash src/ops/audit-trail/replay-audit.sh
```

terminal の設定（~/.bashrc 追加例）:

```bash
export KUBECONFIG=~/.kube/config
alias k='kubectl'
alias bao-status='bao status'
```

## VS Code（任意）

runbook の Markdown ファイルや YAML ファイルの編集には VS Code を使うことができる。

```bash
code --install-extension redhat.vscode-yaml       # YAML サポート
code --install-extension ms-kubernetes-tools.vscode-kubernetes-tools  # kubectl 補助
```

VS Code の Remote - WSL 拡張を使うと Windows 上の VS Code から WSL2 ファイルシステムを直接編集できる。

## Yubikey manager GUI（任意）

Yubikey の PIV スロットを GUI で管理したい場合に任意で導入する。

```bash
# WSL2 上での CLI ツールで代替可能
ykman piv info
ykman piv certificates list
```

Windows 側で Yubikey manager GUI をインストールする場合は公式サイト（developers.yubico.com）からダウンロードする。

## escalation engine クライアントの設定

on-call アラートの受信には escalation engine クライアントを設定する。具体的なクライアントは組織の escalation 基盤に依存する（例: PagerDuty CLI / OpsGenie CLI）。

```bash
# PagerDuty CLI 例（組織設定に依存）
# pd --version
```

## 検収コマンド

```bash
# CLI 環境の確認
bao --version && kubectl version --client && ykman --version
echo "CLI 環境: OK"
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
