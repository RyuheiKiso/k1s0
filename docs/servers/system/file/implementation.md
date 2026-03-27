# system-file-server 実装設計

> **注記**: 本ドキュメントは file-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-file-server（ファイルサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（アップロード・ダウンロード・メタデータ管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア | usecase, domain |
| infrastructure | 設定・DB接続・LocalFsストレージ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/file/)

### ディレクトリ構成

```
regions/system/server/rust/file/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── file.rs                      # FileMetadata エンティティ（名前・サイズ・MIME・タグ・所有者）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── file_repository.rs           # FileRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── file_domain_service.rs       # テナント分離・アクセス制御ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── generate_upload_url.rs           # アップロード用プリサインドURL発行
│   │   ├── complete_upload.rs               # アップロード完了通知処理
│   │   ├── generate_download_url.rs         # ダウンロード用プリサインドURL発行
│   │   ├── get_file_metadata.rs             # ファイルメタデータ取得
│   │   ├── list_files.rs                    # ファイル一覧取得
│   │   ├── delete_file.rs                   # ファイル削除
│   │   └── update_file_tags.rs              # タグ更新
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── file_handler.rs              # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── file_grpc.rs                 # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                      # JWT 認証ミドルウェア
│   │       └── rbac.rs                      # RBAC ミドルウェア
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── file_metadata_postgres.rs        # FileRepository PostgreSQL 実装
│   │   ├── local_fs_storage.rs              # ローカルファイルシステムストレージ（tokio::fs）
│   │   ├── in_memory.rs                     # InMemory リポジトリ（dev/test 用）
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（ファイルイベント通知）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **FileDomainService**: テナント ID によるバケット/プレフィックス分離とアクセス制御ロジック。テナント ID をオブジェクトキープレフィックスに付与する（例: `tenant-abc/path/to/file`）

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `GenerateUploadUrlUseCase` | ストレージURL発行（file-server エンドポイントURL） |
| `CompleteUploadUseCase` | アップロード完了通知処理 + Kafka イベント発行 |
| `GenerateDownloadUrlUseCase` | ダウンロード用ストレージURL発行 |
| `GetFileMetadataUseCase` / `ListFilesUseCase` | メタデータ取得 |
| `DeleteFileUseCase` | ファイル削除（LocalFs + メタデータ） |
| `UpdateFileTagsUseCase` | タグ更新 |

#### 外部連携

- **LocalFs Storage** (`infrastructure/local_fs_storage.rs`): tokio::fs でローカルファイルシステム（PV マウント）を操作する。バックエンドは config で切り替え可能
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.file.events.v1` トピックにファイルイベントを通知する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_FILE_`
- LocalFs 操作失敗時はエラーレスポンスを返す（ファイル操作はリトライしない）

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | テナント分離・アクセス制御 | mockall によるリポジトリモック |
| InMemory テスト | リポジトリ | `in_memory.rs` による DB 不要テスト |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| LocalFs テスト | ストレージ操作 | `tempfile` クレートによる一時ディレクトリを使用したテスト |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・テナント分離設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
