# gRPC ガイド

> **仕様**: テーブル定義・APIスキーマは [gRPC設計.md](./gRPC設計.md) を参照。

## パッケージレベルバージョニングの背景

gRPC のバージョニングは **proto パッケージ名にメジャーバージョンを含める** 方式を採用する。

バージョンアップ時は新しいパッケージディレクトリを作成し、旧バージョンと並行運用する。

```
api/proto/k1s0/service/order/
├── v1/
│   └── order.proto     # 旧バージョン（非推奨期間中は維持）
└── v2/
    └── order.proto     # 新バージョン
```

## Go Interceptor 実装例

```go
// internal/infra/grpc/interceptor.go

func UnaryErrorInterceptor() grpc.UnaryServerInterceptor {
    return func(
        ctx context.Context,
        req interface{},
        info *grpc.UnaryServerInfo,
        handler grpc.UnaryHandler,
    ) (interface{}, error) {
        resp, err := handler(ctx, req)
        if err != nil {
            // ドメインエラーを gRPC ステータスに変換
            if domainErr, ok := err.(*domain.Error); ok {
                st := status.New(mapToGRPCCode(domainErr.Code), domainErr.Message)
                st, _ = st.WithDetails(&errdetails.ErrorInfo{
                    Reason: domainErr.Code,
                    Domain: "k1s0",
                })
                return nil, st.Err()
            }
            return nil, status.Error(codes.Internal, "internal error")
        }
        return resp, nil
    }
}
```

## Rust Interceptor 実装例

```rust
// src/infra/grpc/interceptor.rs

use tonic::{Request, Status};

pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = req.metadata().get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing authorization token"))?;

    // JWT 検証ロジック
    validate_token(token)
        .map_err(|e| Status::unauthenticated(format!("invalid token: {}", e)))?;

    Ok(req)
}
```
