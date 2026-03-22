# ADR-0015: S3/AWS SDK依存の完全削除とローカルファイルシステムへの移行

## ステータス

承認済み

## コンテキスト

k1s0プロジェクトはAWSクラウドを使用しない方針である。しかし既存の実装では、S3互換ストレージ（Ceph RGW）へのアクセスに AWS SDK（`aws-sdk-s3`, `aws-config`, `aws-credential-types`）を使用しており、以下の問題が生じていた。

- **ビルド依存の肥大化**: AWS SDKはコンパイル時間が長く、バイナリサイズを増加させる
- **概念の不一致**: AWSを使用しないにもかかわらず、コードベース全体にAWS固有の概念（S3バケット、presigned URL、Signature V4等）が混在
- **設定の複雑化**: S3エンドポイント・バケット・リージョン・認証情報という複数の設定項目が必要
- **インフラの不一致**: Terraform Cephモジュール、K8sバックアップCronJob（etcd/vault）、CI publish-app.yamlがいずれもS3 APIに依存

影響範囲:
- Rustサーバー: `k1s0-file-server`（S3ストレージバックエンド）、`k1s0-app-registry`（presigned URLによるダウンロード）
- クライアントライブラリ: 4言語（Rust/Go/TypeScript/Dart）の file-client
- インフラ: Terraform Cephモジュール、K8s backup CronJob、CI ワークフロー
- DBスキーマ: `app_versions.s3_key` カラム

## 決定

S3/AWS SDKへの依存をすべて削除し、以下の代替実装に移行する。

1. **file-server**: S3ストレージバックエンドを廃止し、ローカルファイルシステム（PVベース）の `LocalFsStorageRepository` に置換する
2. **app-registry**: presigned URLによるダウンロード方式を廃止し、app-registryサーバーがファイルを直接ストリーミング配信する方式に変更する
3. **DBスキーマ**: `s3_key` カラムを `storage_key` に改名し、ストレージ実装に依存しない汎用名称にする
4. **インフラ**: S3を使用するすべてのインフラコンポーネントをPVCローカル保存に移行する

## 理由

- AWSを使用しない方針に合わせてコードベースを整合させる
- AWS SDKのビルド依存を排除することでビルド時間とバイナリサイズを削減する
- `FileStorageRepository` トレイト（file-server）は既にS3非依存の抽象化として設計されており、新実装の追加コストは低い
- ローカルFS（PVベース）は、Kubernetes環境ではPersistent Volumeとして適切に運用できる

## 影響

**ポジティブな影響**:

- AWS SDKの依存削除によりビルド時間を短縮（`aws-sdk-s3`, `aws-config`, `aws-credential-types`, `aws-smithy-types`の除去）
- コードベースとインフラ方針の整合性が取れる
- 設定がシンプルになる（S3の4項目 → ローカルパスのみ）
- バックアップ戦略がK8sネイティブ（PVCベース）に統一される

**ネガティブな影響・トレードオフ**:

- presigned URLによる署名付きURLの利点（一時的なアクセス委譲）が失われる
- app-registryがファイル配信のプロキシとなるため、大容量ファイルのダウンロード時に帯域負荷がかかる
- バックアップのオフサイト冗長性が低下する（PVC障害時のリカバリ手段を別途検討が必要）
- `s3_key` → `storage_key` のカラム改名にDBマイグレーションが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| AWS SDK維持（Ceph RGW向け） | 既存の実装を維持し、Ceph RGWへの接続を継続 | AWSを使用しない方針に反する |
| AWS SDKを使わずS3互換HTTP実装 | AWS Signature V4を自前実装してS3互換APIを直接呼び出す（Dartの実装と同様） | S3互換ストレージ自体を使わない方針なので不要 |
| MinIO Client SDK利用 | MinIO専用クライアントライブラリを使用 | S3互換ストレージ自体を廃止するため不要 |

## 参考

- [file-server設計書](../../servers/system/file/server.md)
- [app-registry設計書](../../servers/system/app-registry/server.md)
- [ADR-0002: モノリポ構成](0002-monorepo.md)
