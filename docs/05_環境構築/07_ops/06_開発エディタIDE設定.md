---
id: env.ops.ops_editor_ide
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.ops_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- ops 軸エンジニアは VS Code に Grafana extension / YAML extension / Backstage plugin を導入し、Prometheus alert rule の YAML 編集・k6 スクリプト編集・TechDocs 執筆を手元で完結できる状態を検収条件とする。

## VS Code の必須 extension

| extension | ID | 用途 |
|---|---|---|
| YAML | `redhat.vscode-yaml` | Prometheus alert rule / Kubernetes manifest の schema 検証 |
| Grafana | `grafana.vscode-grafana-scenes` | Grafana dashboard JSON の編集支援 |
| Backstage | `backstage.backstage` | Backstage catalog-info.yaml の IntelliSense |
| ESLint | `dbaeumer.vscode-eslint` | k6 スクリプト lint |
| Prettier | `esbenp.prettier-vscode` | k6 スクリプト format |
| Docker | `ms-azuretools.vscode-docker` | docker compose 管理 |
| Remote - WSL | `ms-vscode-remote.remote-wsl` | WSL2 からの VS Code 起動 |

```bash
# CLI でインストール
code --install-extension redhat.vscode-yaml
code --install-extension grafana.vscode-grafana-scenes
code --install-extension backstage.backstage
code --install-extension dbaeumer.vscode-eslint
code --install-extension esbenp.prettier-vscode
code --install-extension ms-azuretools.vscode-docker
code --install-extension ms-vscode-remote.remote-wsl
```

## VS Code の settings.json 推奨設定

```json
{
  "yaml.schemas": {
    "https://raw.githubusercontent.com/instrumenta/kubernetes-json-schema/master/v1.18.0-standalone-strict/all.json": ["*.k8s.yaml", "*.kubernetes.yaml"],
    "https://raw.githubusercontent.com/prometheus/prometheus/main/documentation/examples/prometheus.yml": ["prometheus.yml"]
  },
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[javascript]": {
    "editor.defaultFormatter": "dbaeumer.vscode-eslint"
  }
}
```

## YAML extension の Prometheus alert rule schema

YAML extension に Prometheus alert rule schema を設定することで、alerting rule の typo を VS Code 上で検出できる。

```yaml
# .vscode/settings.json に追加
{
  "yaml.schemas": {
    "https://json.schemastore.org/prometheus-alerting-rules.json": ["**/alerts/*.yaml", "**/alerts/*.yml"]
  }
}
```

## Backstage VS Code plugin の活用

Backstage plugin をインストールすると `catalog-info.yaml` の `kind: Component` / `kind: API` / `kind: System` の IntelliSense が有効になる。software template の作成時に必須フィールドを補完できる。

## 検収コマンド

```bash
code --list-extensions | grep -E "redhat.vscode-yaml|grafana|backstage|eslint|prettier|docker|remote-wsl"
```

6 件以上ヒットすることを確認する。

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
- [08_lintとformat適用](08_lintとformat適用.md)
