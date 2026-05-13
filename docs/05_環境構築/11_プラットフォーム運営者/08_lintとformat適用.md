---
id: env.ops.platform_operator_lint_format
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.platform_operator_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- プラットフォーム運営者は OPA conftest による RBAC policy lint と cosign verify の自動化スクリプトを手元で実行し、policy 変更が green であることを push 前に確認する。

## OPA conftest による RBAC policy lint

`conftest` は Open Policy Agent（OPA）を使った policy-as-code テストツール。RBAC マニフェストの policy 検査に使用する。

```bash
# conftest のインストール
curl -L https://github.com/open-policy-agent/conftest/releases/latest/download/conftest_linux_amd64.tar.gz \
  | tar xz -C /usr/local/bin/
conftest --version

# RBAC policy の lint 実行
conftest test src/ops/rbac/ --policy src/ops/policy/
```

policy ファイルは `src/ops/policy/` 配下の `.rego` ファイルとして管理する。

## cosign verify の自動化

artifact の署名検証を自動化する。

```bash
# cosign verify スクリプトの実行
bash src/ops/verify-artifacts.sh
# 期待: "All artifacts verified" または同等のメッセージ
```

verify スクリプトが存在しない場合は個別に実行する。

```bash
cosign verify --key cosign.pub <image-reference>
```

## docs lint との統合

RBAC マニフェスト / policy ファイルは `docs/` 配下には置かない。`src/ops/` に配置し、docs lint の対象外とする。

```bash
bash tools/docs_lint/run_lint.sh
python3 tools/docs_lint/run_lint.py
```

## 検収コマンド

```bash
conftest --version
cosign version
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
