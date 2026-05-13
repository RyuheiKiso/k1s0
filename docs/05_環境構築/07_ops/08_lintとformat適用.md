---
id: env.ops.ops_lint_format
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.ops_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- ops 軸エンジニアは Backstage TechDocs lint（techdocs-cli）/ Prometheus alerting rule lint（promtool check rules）/ k6 スクリプト ESLint の 3 lint を手元で実行できることを本ページの検収条件とする。

## Backstage TechDocs lint（techdocs-cli）

```bash
# techdocs-cli のインストール
npm install -g @techdocs/cli
techdocs-cli --version

# TechDocs ビルド（docs/ ディレクトリに mkdocs.yml がある場合）
techdocs-cli build --no-docker
# または
techdocs-cli build --source-dir . --output-dir ./site
```

TechDocs は MkDocs を使ってドキュメントを生成する。`mkdocs.yml` が存在しない場合は Backstage catalog-info.yaml の `backstage.io/techdocs-ref` アノテーションを確認する。

## Prometheus alerting rule lint（promtool check rules）

promtool は Prometheus に同梱されているが、コンテナ外からも使いたい場合は単体でインストールする。

```bash
# promtool 単体インストール（Prometheus バイナリに同梱）
PROM_VERSION=$(curl -s https://api.github.com/repos/prometheus/prometheus/releases/latest | grep tag_name | cut -d '"' -f 4)
curl -LO "https://github.com/prometheus/prometheus/releases/download/${PROM_VERSION}/prometheus-${PROM_VERSION#v}.linux-amd64.tar.gz"
tar xzf "prometheus-${PROM_VERSION#v}.linux-amd64.tar.gz"
sudo cp "prometheus-${PROM_VERSION#v}.linux-amd64/promtool" /usr/local/bin/
promtool --version
```

alert rule の lint:

```bash
# alert rule ファイルの検証
promtool check rules alerts/*.yaml
```

エラーがなければ alert rule の構文は正常。

## k6 スクリプト ESLint

k6 スクリプトは JavaScript（ES modules）で書くため ESLint で品質を維持する。

```bash
# ESLint と k6 用 type definitions のインストール
pnpm add -D eslint @types/k6 eslint-config-prettier
# または
npm install -D eslint @types/k6 eslint-config-prettier

# k6 スクリプトの lint
npx eslint --ext .js k6/
```

`.eslintrc.json` の最小構成:

```json
{
  "env": { "es2020": true },
  "extends": ["eslint:recommended"],
  "parserOptions": { "ecmaVersion": 2020, "sourceType": "module" },
  "rules": { "no-unused-vars": "warn" }
}
```

## markdownlint（TechDocs MD ファイル）

Backstage TechDocs の Markdown ファイルに対しても markdownlint を適用する。

```bash
npx markdownlint-cli --config docs/00_format/linters/markdownlint.json 'docs/**/*.md'
```

## 検収コマンド

```bash
techdocs-cli --version
promtool --version
npx eslint --version
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
