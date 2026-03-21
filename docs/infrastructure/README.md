# インフラ設計書

k1s0 プロジェクトのインフラ・CI/CD・開発環境・セキュリティに関する設計書一覧。

## overview — インフラ概要

| ドキュメント | 内容 |
|------------|------|
| [overview/インフラ設計.md](./overview/インフラ設計.md) | インフラ全体設計・技術選定方針 |

---

## cicd — CI/CD 設計

| ドキュメント | 内容 |
|------------|------|
| [cicd/CI-CD設計.md](./cicd/CI-CD設計.md) | GitHub Actions パイプライン設計・デプロイフロー |
| [cicd/go-stub-policy.md](./cicd/go-stub-policy.md) | Go スタブ生成ポリシー・CI 運用ルール |
| [cicd/migration-design-process.md](./cicd/migration-design-process.md) | マイグレーション設計プロセス・CI 組み込み方針 |

---

## devenv — 開発環境

| ドキュメント | 内容 |
|------------|------|
| [devenv/windows-quickstart.md](./devenv/windows-quickstart.md) | Windows 開発環境クイックスタート（推奨） |
| [devenv/WSL2開発環境セットアップ.md](./devenv/WSL2開発環境セットアップ.md) | WSL2 開発環境セットアップ手順 |
| [devenv/devcontainer設計.md](./devenv/devcontainer設計.md) | Dev Container 設計・VSCode 統合 |
| [devenv/共用開発サーバー設計.md](./devenv/共用開発サーバー設計.md) | 共用開発サーバー構成設計 |

---

## docker — Docker 設計

| ドキュメント | 内容 |
|------------|------|
| [docker/docker-compose設計.md](./docker/docker-compose設計.md) | docker-compose 全体設計・サービス構成 |
| [docker/compose-インフラサービス設計.md](./docker/compose-インフラサービス設計.md) | PostgreSQL・Redis・Kafka 等インフラサービスの compose 設計 |
| [docker/compose-システムサービス設計.md](./docker/compose-システムサービス設計.md) | k1s0 system サーバー群の compose 設計 |
| [docker/compose-可観測性サービス設計.md](./docker/compose-可観測性サービス設計.md) | Prometheus・Grafana・Loki・Jaeger 等の compose 設計 |
| [docker/Dockerイメージ戦略.md](./docker/Dockerイメージ戦略.md) | マルチステージビルド・イメージサイズ最適化戦略 |
| [docker/ポート割り当て.md](./docker/ポート割り当て.md) | 全サービスのポート番号割り当て一覧 |

---

## kubernetes — Kubernetes 設計

| ドキュメント | 内容 |
|------------|------|
| [kubernetes/kubernetes設計.md](./kubernetes/kubernetes設計.md) | Kubernetes クラスター設計・リソース構成 |
| [kubernetes/helm設計.md](./kubernetes/helm設計.md) | Helm チャート設計・バリュー管理 |

---

## security — セキュリティ設計

| ドキュメント | 内容 |
|------------|------|
| [security/secrets-management.md](./security/secrets-management.md) | シークレット管理設計（Vault連携） |
| [security/security-scanning.md](./security/security-scanning.md) | セキュリティスキャン設計（SAST/DAST/依存関係スキャン） |
| [security/Vault設計.md](./security/Vault設計.md) | HashiCorp Vault 設計・ポリシー・シークレットエンジン |

---

## service-mesh — サービスメッシュ設計

| ドキュメント | 内容 |
|------------|------|
| [service-mesh/サービスメッシュ設計.md](./service-mesh/サービスメッシュ設計.md) | Istio / Envoy サービスメッシュ設計・mTLS・トラフィック管理 |

---

## terraform — Terraform 設計

| ドキュメント | 内容 |
|------------|------|
| [terraform/terraform設計.md](./terraform/terraform設計.md) | IaC（Infrastructure as Code）設計・モジュール構成 |

---

## distribution — アプリ配布設計

| ドキュメント | 内容 |
|------------|------|
| [distribution/アプリ配布基盤設計.md](./distribution/アプリ配布基盤設計.md) | クライアントアプリ配布基盤設計（App Registry連携） |

---

## justfile

| ドキュメント | 内容 |
|------------|------|
| [justfile.md](./justfile.md) | just タスクランナー設計・よく使うコマンド一覧 |

---

## 関連ドキュメント

- [アーキテクチャ設計書](../architecture/README.md) — 全体設計方針
- [サーバー設計書](../servers/README.md) — デプロイ対象サーバー一覧
- [テンプレート仕様](../templates/README.md) — インフラコード生成テンプレート
