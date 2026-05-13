---
id: env.ops.ops_oss_install
axis: ops
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.ops.ops_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- ops 軸エンジニアは Argo Rollouts CLI / Tekton CLI / Backstage / Grafana / Prometheus / Alertmanager / k6 を導入し、全て `--version` 応答またはローカル起動確認が取れることを本ページの検収条件とする。

## Argo Rollouts kubectl Plugin（kubectl-argo-rollouts）

```bash
curl -LO https://github.com/argoproj/argo-rollouts/releases/latest/download/kubectl-argo-rollouts-linux-amd64
chmod +x kubectl-argo-rollouts-linux-amd64
sudo mv kubectl-argo-rollouts-linux-amd64 /usr/local/bin/kubectl-argo-rollouts
kubectl-argo-rollouts version
```

Argo Rollouts はクラスタへのデプロイ戦略（Blue/Green / Canary）を管理する。ops 軸エンジニアは rollout strategy の設計と運用を担う。

## Tekton CLI（tkn）

```bash
curl -LO https://github.com/tektoncd/cli/releases/latest/download/tkn_Linux_x86_64.tar.gz
tar xvzf tkn_Linux_x86_64.tar.gz
sudo mv tkn /usr/local/bin/
tkn version
```

## Backstage（Node.js app）

Backstage は Node.js 20 LTS が必要。pnpm を推奨する。

```bash
# Node.js 20 LTS インストール（nvm 経由）
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
source ~/.bashrc
nvm install 20
nvm use 20
node --version   # v20.x.x

# pnpm インストール
npm install -g pnpm
pnpm --version

# Backstage app の初期化（新規の場合）
npx @backstage/create-app@latest
```

既存の Backstage 設定がリポジトリにある場合は `pnpm install` で依存を解決する。

## Grafana（docker）

```bash
docker pull grafana/grafana:latest
docker run -d --name grafana -p 3000:3000 grafana/grafana:latest
# ブラウザ: http://localhost:3000 (admin/admin)
docker stop grafana && docker rm grafana
```

本番では docker compose で他コンテナと一緒に起動する（07_テスト検証環境 参照）。

## Prometheus（docker）

```bash
docker pull prom/prometheus:latest
docker run -d --name prometheus -p 9090:9090 prom/prometheus:latest
# ブラウザ: http://localhost:9090
docker stop prometheus && docker rm prometheus
```

## Alertmanager（docker）

```bash
docker pull prom/alertmanager:latest
docker run -d --name alertmanager -p 9093:9093 prom/alertmanager:latest
docker stop alertmanager && docker rm alertmanager
```

## k6

03_必須ランタイム でインストール済み。確認コマンド:

```bash
k6 version
```

## 検収コマンド

```bash
kubectl-argo-rollouts version
tkn version
node --version
pnpm --version
k6 version
docker pull grafana/grafana:latest && echo "Grafana image OK"
docker pull prom/prometheus:latest && echo "Prometheus image OK"
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
