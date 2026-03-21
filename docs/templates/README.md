# テンプレート仕様

k1s0 CLI によるコード生成テンプレートの仕様ドキュメント一覧。
各テンプレートは CLI の `codegen` コマンドから呼び出され、サーバー・クライアント・インフラのスキャフォールドを生成する。

## engine — テンプレートエンジン仕様

| ドキュメント | 内容 |
|------------|------|
| [engine/テンプレートエンジン仕様.md](./engine/テンプレートエンジン仕様.md) | Tera テンプレートエンジン設計・変数・フィルター・ブロック仕様 |

---

## server — サーバーテンプレート

| ドキュメント | 内容 |
|------------|------|
| [server/サーバー.md](./server/サーバー.md) | Go サーバー基本テンプレート（クリーンアーキテクチャ構成） |
| [server/サーバー-gRPC.md](./server/サーバー-gRPC.md) | gRPC サーバーテンプレート（Proto 定義・サービス実装） |
| [server/サーバー-認証.md](./server/サーバー-認証.md) | 認証ミドルウェア付きサーバーテンプレート |
| [server/サーバー-可観測性.md](./server/サーバー-可観測性.md) | OpenTelemetry 統合サーバーテンプレート |
| [server/サーバー-Rust.md](./server/サーバー-Rust.md) | Rust（axum）サーバーテンプレート |

---

## client — クライアントテンプレート

| ドキュメント | 内容 |
|------------|------|
| [client/クライアント.md](./client/クライアント.md) | クライアント SDK 基本テンプレート |
| [client/BFF.md](./client/BFF.md) | BFF（Backend for Frontend）テンプレート |
| [client/React.md](./client/React.md) | React クライアントアプリテンプレート |
| [client/Flutter.md](./client/Flutter.md) | Flutter クライアントアプリテンプレート |

---

## data — データ定義テンプレート

| ドキュメント | 内容 |
|------------|------|
| [data/データベース.md](./data/データベース.md) | DB スキーマ・マイグレーションファイルテンプレート |
| [data/APIスキーマ.md](./data/APIスキーマ.md) | OpenAPI / Proto スキーマテンプレート |
| [data/Config.md](./data/Config.md) | config.yaml テンプレート |

---

## infrastructure — インフラテンプレート

| ドキュメント | 内容 |
|------------|------|
| [infrastructure/CICD.md](./infrastructure/CICD.md) | GitHub Actions CI/CD パイプラインテンプレート |
| [infrastructure/DockerCompose.md](./infrastructure/DockerCompose.md) | docker-compose.yml テンプレート |
| [infrastructure/docker-build.md](./infrastructure/docker-build.md) | Dockerfile マルチステージビルドテンプレート |
| [infrastructure/Helm.md](./infrastructure/Helm.md) | Helm チャートテンプレート |
| [infrastructure/Terraform.md](./infrastructure/Terraform.md) | Terraform IaC テンプレート |
| [infrastructure/devcontainer.md](./infrastructure/devcontainer.md) | Dev Container 設定テンプレート |

---

## middleware — ミドルウェアテンプレート

| ドキュメント | 内容 |
|------------|------|
| [middleware/Kafka.md](./middleware/Kafka.md) | Kafka トピック設定・プロデューサー/コンシューマーテンプレート |
| [middleware/Kong.md](./middleware/Kong.md) | Kong API ゲートウェイ設定テンプレート |
| [middleware/Keycloak.md](./middleware/Keycloak.md) | Keycloak レルム・クライアント設定テンプレート |
| [middleware/Vault.md](./middleware/Vault.md) | HashiCorp Vault ポリシー・シークレット設定テンプレート |
| [middleware/Consul.md](./middleware/Consul.md) | Consul サービス登録・ヘルスチェックテンプレート |
| [middleware/Storage.md](./middleware/Storage.md) | オブジェクトストレージ（S3/GCS/Ceph）設定テンプレート |
| [middleware/ServiceMesh.md](./middleware/ServiceMesh.md) | Istio / Envoy サービスメッシュ設定テンプレート |
| [middleware/Flagger.md](./middleware/Flagger.md) | Flagger カナリアリリース設定テンプレート |

---

## observability — 可観測性テンプレート

| ドキュメント | 内容 |
|------------|------|
| [observability/Observability.md](./observability/Observability.md) | 可観測性スタック全体テンプレート |
| [observability/OpenTelemetry.md](./observability/OpenTelemetry.md) | OpenTelemetry Collector 設定テンプレート |
| [observability/Grafana.md](./observability/Grafana.md) | Grafana ダッシュボードテンプレート |
| [observability/Loki.md](./observability/Loki.md) | Loki ログ収集設定テンプレート |
| [observability/Alertmanager.md](./observability/Alertmanager.md) | Alertmanager アラートルールテンプレート |

---

## testing — テストテンプレート

| ドキュメント | 内容 |
|------------|------|
| [testing/レンダリングテスト.md](./testing/レンダリングテスト.md) | Go / TypeScript レンダリングテストテンプレート |
| [testing/レンダリングテスト-Rust.md](./testing/レンダリングテスト-Rust.md) | Rust レンダリングテストテンプレート |

---

## codegen — コード生成テンプレート

| ドキュメント | 内容 |
|------------|------|
| [codegen/コード生成パイプライン.md](./codegen/コード生成パイプライン.md) | コード生成パイプライン仕様 |
| [codegen/ライブラリ.md](./codegen/ライブラリ.md) | ライブラリスキャフォールド生成テンプレート |

---

## 関連ドキュメント

- [CLI 設計書](../cli/README.md) — テンプレートを使用する CLI 設計
- [ライブラリ設計書](../libraries/README.md) — codegen ライブラリ
- [インフラ設計書](../infrastructure/README.md) — インフラ設計の詳細
