pub mod auth_grpc;
pub mod audit_grpc;

// gRPC サービス実装。tonic の proto コンパイルが必要なため、
// 現時点ではスタブとして定義する。
// build.rs で tonic-build を実行後に proto 型を使用可能になる。
