---
id: env.test.test_editor_ide
axis: test
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.test.test_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- test 軸エンジニアは VS Code に Python / Java / Playwright / Rust extension を導入し、5 verification_class のテスト開発を手元で完結できる状態を検収条件とする。

## VS Code の必須 extension

| extension | ID | 用途 |
|---|---|---|
| Python | `ms-python.python` | pytest / Hypothesis 実行・デバッグ |
| Pylance | `ms-python.vscode-pylance` | Python 型推論・IntelliSense |
| Ruff | `charliermarsh.ruff` | Python lint（ruff）|
| Java Extension Pack | `vscjava.vscode-java-pack` | Java 21 / jqwik / pitest 開発 |
| Playwright | `ms-playwright.playwright` | Playwright test runner |
| rust-analyzer | `rust-lang.rust-analyzer` | Rust / proptest / cargo-mutants |
| ESLint | `dbaeumer.vscode-eslint` | Vitest / Stryker スクリプト lint |
| Docker | `ms-azuretools.vscode-docker` | Pact broker / Litmus コンテナ管理 |
| Remote - WSL | `ms-vscode-remote.remote-wsl` | WSL2 から VS Code 起動 |

```bash
# CLI でインストール
code --install-extension ms-python.python
code --install-extension ms-python.vscode-pylance
code --install-extension charliermarsh.ruff
code --install-extension vscjava.vscode-java-pack
code --install-extension ms-playwright.playwright
code --install-extension rust-lang.rust-analyzer
code --install-extension dbaeumer.vscode-eslint
code --install-extension ms-azuretools.vscode-docker
code --install-extension ms-vscode-remote.remote-wsl
```

## VS Code settings.json 推奨設定

```json
{
  "python.defaultInterpreterPath": "${workspaceFolder}/.venv/bin/python",
  "python.testing.pytestEnabled": true,
  "python.testing.pytestArgs": ["--tb=short"],
  "[python]": {
    "editor.defaultFormatter": "charliermarsh.ruff",
    "editor.formatOnSave": true
  },
  "java.home": "/usr/lib/jvm/java-21-openjdk-amd64",
  "rust-analyzer.cargo.features": "all",
  "playwright.reuseBrowser": false
}
```

## Java Extension Pack の設定

Java Extension Pack インストール後に Java 21 のパスを VS Code に認識させる。

```json
{
  "java.configuration.runtimes": [
    {
      "name": "JavaSE-21",
      "path": "/usr/lib/jvm/java-21-openjdk-amd64",
      "default": true
    }
  ]
}
```

## Playwright extension の活用

Test Explorer でテストをブラウザ別に GUI 実行できる。Pact contract test のデバッグには pytest デバッガを使う。

## 検収コマンド

```bash
code --list-extensions | grep -E "ms-python|pylance|ruff|java|playwright|rust-lang|eslint|docker|remote-wsl"
```

7 件以上ヒットすることを確認する。

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
- [08_lintとformat適用](08_lintとformat適用.md)
