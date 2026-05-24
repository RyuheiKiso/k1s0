---
id: env.infra.infra_lint_format
axis: infra
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.infra.infra_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- kubeconform（manifest バリデーション）/ conftest（OPA policy）/ helm lint / kustomize build の 4 ツールを手元で実行し、全 check が通ることを infra 軸の lint 検収条件とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の lint / format 手順は P10 完了後に有効。

## kubeconform のインストールと実行

kubeconform は k8s manifest を JSON Schema で検証するツール。

```bash
curl -LO https://github.com/yannh/kubeconform/releases/latest/download/kubeconform-linux-amd64.tar.gz
tar xf kubeconform-linux-amd64.tar.gz
sudo mv kubeconform /usr/local/bin/kubeconform
kubeconform --version

# manifest の検証
kubeconform -strict -summary <path-to-manifests>/*.yaml
```

## conftest のインストールと実行

conftest は OPA (Open Policy Agent) を使って k8s manifest を検証するツール。

```bash
curl -LO https://github.com/open-policy-agent/conftest/releases/latest/download/conftest_Linux_x86_64.tar.gz
tar xf conftest_Linux_x86_64.tar.gz
sudo mv conftest /usr/local/bin/conftest
conftest --version

# OPA policy ファイルを作成してテスト（例: privileged container 禁止）
mkdir -p policy
cat <<'EOF' > policy/deny_privileged.rego
package main

deny[msg] {
  input.spec.containers[_].securityContext.privileged
  msg := "Privileged containers are not allowed"
}
EOF

conftest test <path-to-manifests>/*.yaml
```

## helm lint の実行

```bash
# chart ディレクトリに対して lint を実行
helm lint <path-to-chart>/
# --strict フラグで警告もエラー扱い
helm lint --strict <path-to-chart>/
```

## kustomize build の確認

```bash
# kustomize overlay を build してエラーがないことを確認
kustomize build <path-to-overlay>/ | kubeconform -strict -summary -
```

kustomize build の出力をパイプで kubeconform に渡すことで、overlay 適用後の manifest を一括検証できる。

## よくある lint エラーと対処

| エラー | 原因 | 対処 |
|---|---|---|
| `unknown field: spec.templateXxx` | CRD フィールドの typo | フィールド名を正確に確認 |
| `error: missing required field: metadata.name` | name がない | metadata.name を追加 |
| `FAILED: Privileged containers are not allowed` | 特権 container | securityContext.privileged を除去 |
| `Error: chart requires kubeVersion: >=1.25.0` | kubectl version 不一致 | cluster バージョンを確認 |

## 検収コマンド

```bash
kubeconform --version
conftest --version
helm version --short
kustomize version
```

4 コマンド全て応答することを確認する。

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
