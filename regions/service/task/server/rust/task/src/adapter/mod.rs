// アダプター層のモジュール宣言。
// grpc: gRPCサービス実装、handler: RESTハンドラー、middleware: 認証・RBAC ミドルウェア
pub mod grpc;
pub mod handler;
pub mod middleware;
