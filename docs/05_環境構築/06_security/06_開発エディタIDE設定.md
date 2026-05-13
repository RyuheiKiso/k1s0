---
id: env.security.security_editor_ide
axis: security
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.security.security_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code に Trivy VS Code extension / Semgrep extension を導入し、コード編集時にリアルタイムで SAST スキャンが実行される状態を security 軸の IDE 検収条件とする。

## VS Code 拡張のインストール

```bash
code --install-extension AquaSecurityOfficial.trivy-vulnerability-scanner
code --install-extension Semgrep.semgrep
code --install-extension redhat.vscode-yaml
code --install-extension ms-vscode.vscode-github-issue-notebooks
```

| 拡張 | 用途 |
|---|---|
| trivy-vulnerability-scanner | ファイル保存時に Trivy で脆弱性スキャン |
| semgrep | SAST（静的解析）でセキュリティパターン検出 |
| vscode-yaml | k8s manifest / SPIRE 設定ファイルの補完 |
| vscode-github-issue-notebooks | セキュリティ Issue のトラッキング |

## Trivy VS Code extension の設定

```json
{
  "trivy.path": "/usr/local/bin/trivy",
  "trivy.onlyFixed": false,
  "trivy.severity": "MEDIUM,HIGH,CRITICAL"
}
```

拡張をインストール後、プロジェクトを開いて Trivy が自動実行されることを確認する。

## Semgrep extension の設定

Semgrep のルールセットを設定する。

```bash
# OWASP ルールセットを使用
semgrep --config "p/owasp-top-ten" --dry-run .
```

`.semgrepignore` ファイルで除外パターンを設定する:

```
.venv/
node_modules/
*.min.js
```

## pre-commit hook の設定（セキュリティ強化）

```bash
source .venv/bin/activate
pip install pre-commit

# .pre-commit-config.yaml の作成
cat <<'EOF' > .pre-commit-config.yaml
repos:
  - repo: https://github.com/Yelp/detect-secrets
    rev: v1.4.0
    hooks:
      - id: detect-secrets
        args: ['--baseline', '.secrets.baseline']
  - repo: https://github.com/semgrep/semgrep
    rev: v1.70.0
    hooks:
      - id: semgrep
        args: ['--config', 'p/security-audit']
EOF

pre-commit install
```

## 検収コマンド

```bash
code --list-extensions | grep -E "trivy|semgrep"
# 2 拡張が一覧に含まれること
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
- [08_lintとformat適用](08_lintとformat適用.md)
