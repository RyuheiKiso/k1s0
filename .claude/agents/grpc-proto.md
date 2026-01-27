# gRPC/Protocol Buffers エージェント

Protocol Buffers の定義、lint、gRPC サービス開発を支援するエージェント。

## 対象領域

- `**/proto/` - Protocol Buffers 定義ファイル
- gRPC サービス実装（Rust/Go）

## 主な操作

### buf lint

```bash
# Protocol Buffers の lint 検証
./scripts/buf-check.sh

# または直接実行
buf lint
```

### コード生成

Protocol Buffers からの gRPC コード生成は各サービスのビルドプロセスで自動実行される。

**Rust の場合:**
- `tonic-build` を使用
- `build.rs` で自動生成

**Go の場合:**
- `protoc-gen-go` と `protoc-gen-go-grpc` を使用

## ディレクトリ構造

```
<service>/
├── proto/
│   ├── <service>.proto    # サービス定義
│   └── gen/               # 生成コード（.gitignore 対象）
└── ...
```

## 規約

### ADR-0005: gRPC コントラクト管理

詳細は `docs/adr/0005-grpc-contract-management.md` を参照。

### リトライ設定

gRPC リトライを設定する場合は以下を遵守:

1. ADR でリトライ戦略を文書化
2. 完全な設定を記述（max_attempts, initial_backoff, max_backoff, backoff_multiplier, retryable_status_codes）
3. Lint ルール K030-K032 で検証される

## gRPC サービス実装パターン

### Rust (tonic)

```rust
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl MyService for MyServiceImpl {
    async fn my_method(
        &self,
        request: Request<MyRequest>,
    ) -> Result<Response<MyResponse>, Status> {
        // 実装
    }
}
```

### Go (grpc-go)

```go
func (s *server) MyMethod(ctx context.Context, req *pb.MyRequest) (*pb.MyResponse, error) {
    // 実装
}
```

## 関連 Crate/パッケージ

**Rust:**
- `tonic`: gRPC フレームワーク
- `prost`: Protocol Buffers 実装
- `tonic-build`: ビルド時コード生成
- `k1s0-grpc-server`: gRPC サーバ基盤
- `k1s0-grpc-client`: gRPC クライアント基盤

## CI 検証

GitHub Actions `buf.yml` で以下を検証:
- proto ファイルの lint
- breaking change 検出（将来対応予定）
