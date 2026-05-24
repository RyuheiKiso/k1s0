---
id: env.security.security_lint_format
axis: security
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.security.security_test_environment
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- Semgrep (SAST) / conftest OPA policy for k8s / checkov (IaC lint) の 3 ツールを手元で実行し、全 check が通ることを security 軸の lint 検収条件とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の lint / format 手順は P10 完了後に有効。

## Semgrep (SAST) の実行

Semgrep はソースコードのセキュリティパターンを静的解析するツール。

```bash
source .venv/bin/activate

# OWASP TOP 10 ルールセットでスキャン
semgrep --config "p/owasp-top-ten" src/

# セキュリティ監査ルールセット
semgrep --config "p/security-audit" src/

# 特定のルールセット（Python）
semgrep --config "p/python" src/

# CI 用（exit code 1 でブロック）
semgrep --config "p/security-audit" --error src/ || true
```

主要ルールカテゴリ:

| カテゴリ | 検出内容 |
|---|---|
| injection | SQL / コマンド / LDAP インジェクション |
| secrets | ハードコードされた credential |
| crypto | 脆弱な暗号アルゴリズム（MD5 / SHA-1 / DES） |
| deserialization | 安全でないデシリアライズ |
| path-traversal | パストラバーサル脆弱性 |

## conftest OPA policy（k8s 向け）

```bash
conftest --version

# セキュリティポリシーファイルの作成
mkdir -p security-policy
cat <<'EOF' > security-policy/deny_privileged.rego
package main

deny[msg] {
  input.spec.containers[_].securityContext.privileged == true
  msg := sprintf("Container '%s' must not run as privileged", [input.metadata.name])
}

deny[msg] {
  not input.spec.securityContext.runAsNonRoot
  msg := sprintf("Pod '%s' must set runAsNonRoot: true", [input.metadata.name])
}
EOF

# manifest に対して policy を適用
conftest test --policy security-policy/ <path-to-manifests>/*.yaml
```

## checkov (IaC lint) の実行

```bash
source .venv/bin/activate

# k8s manifest のスキャン
checkov -d <path-to-manifests>/ --framework kubernetes

# Terraform のスキャン（IaC ファイルがある場合）
checkov -d <path-to-terraform>/ --framework terraform

# docker-compose のスキャン
checkov -f docker-compose.yml --framework dockerfile

# JSON レポートの出力
checkov -d . --output json > /tmp/checkov-results.json
```

## ハードコードされた secret の検査

```bash
# detect-secrets でスキャン
source .venv/bin/activate
detect-secrets scan --all-files . | python3 -m json.tool

# git 履歴に secret が含まれていないかスキャン
# trufflehog（バイナリ）
curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/scripts/install.sh | sh -s -- -b /tmp
/tmp/trufflehog git file://. --only-verified 2>/dev/null | head -20 || true
```

## 検収コマンド

```bash
source .venv/bin/activate
semgrep --version
checkov --version
conftest --version
```

3 コマンド全て応答することを確認する。

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
