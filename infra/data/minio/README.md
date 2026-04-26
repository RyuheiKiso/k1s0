# infra/data/minio — MinIO（S3 互換オブジェクトストレージ）

ADR-DATA-003 に従い、S3 互換オブジェクトストレージを MinIO（distributed mode）で運用する。
本ディレクトリは production-grade defaults を持ち、Argo CD で `minio` namespace に展開される。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | MinIO Helm chart の values（distributed mode 4 replica、PrometheusMetrics、SSL） |

## 利用するアプリ

- **Binding API**（S3 出力バインディング）— `t1-state` Pod が Dapr Output Binding 経由で参照
- **PostgreSQL バックアップ先** — CNPG が WAL アーカイブ + ベースバックアップを保存
- **Artifacts** — CI でビルドした SBOM / 監査ログアーカイブ等の保管

## デプロイ

```sh
helm repo add minio https://charts.min.io/
helm upgrade --install minio minio/minio -n minio --create-namespace -f infra/data/minio/values.yaml
```

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/70-minio/`） | prod（本ディレクトリ） |
|---|---|---|
| mode | standalone | distributed（4 server × 4 drive） |
| persistence | 5Gi | 100Gi × 4（合計 400Gi、erasure coding 有効） |
| ingress | 無効 | 有効（cert-manager 連携 TLS、Istio Gateway） |
| 監視 | 無効 | PrometheusMetrics 有効 |
| 認証 | dev パスワード平文 | OpenBao 動的シークレット（plan 04-06） |

## バケット運用

prod では以下のバケットを事前作成する（values.yaml の `buckets:` で宣言）:

| バケット | 用途 |
|---|---|
| `k1s0-artifacts` | CI ビルド成果物（SBOM、コンテナ image manifest 等） |
| `k1s0-events` | Kafka からのアーカイブ（コンプライアンス保管 7 年） |
| `k1s0-postgres-backup` | CNPG の WAL アーカイブ + ベースバックアップ |
| `k1s0-audit-export` | Audit ストアの定期 export（CSV / Parquet） |

## 関連設計

- [ADR-DATA-003](../../../docs/02_構想設計/adr/ADR-DATA-003-minio-s3.md)
- FR-T1-BINDING / Backup runbook（plan 05-XX）
