// rpc.rs — k1s0 tier1 Library: RPC L1+ facade trait
// tonic 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// 単項 RPC / サーバー登録 / リクエストハンドラの 3 trait を定義する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// RpcClient は tonic を L1+ ラップする単項 RPC クライアント facade trait。
// 公開 API シグネチャに OSS 型（tonic::transport::Channel 等）を一切含まない。
#[async_trait]
pub trait RpcClient: Send + Sync {
    // call_unary は単項 RPC を呼び出してレスポンスバイト列を返す。
    // service は gRPC サービス名（例: "k1s0.tier1.v1.GatewayService"）。
    // method は gRPC メソッド名（例: "CreateSession"）。
    // req はシリアライズ済みリクエストバイト列（protobuf 推奨）。
    async fn call_unary(&self, service: &str, method: &str, req: Vec<u8>) -> crate::Result<Vec<u8>>;
}

// RpcServer は tonic を L1+ ラップするサービス登録 facade trait。
// 公開 API シグネチャに OSS 型（tonic::transport::Server 等）を一切含まない。
pub trait RpcServer: Send + Sync {
    // register_service はサービスエンドポイントを handler に紐付けて登録する。
    // service はサービス名（例: "k1s0.tier1.v1.GatewayService"）。
    // handler はリクエストを処理する Box<dyn RpcHandler>。
    fn register_service(&mut self, service: &str, handler: Box<dyn RpcHandler>);
}

// RpcHandler は単一サービスのリクエスト処理 facade trait。
// 具体的なメソッドディスパッチは実装側で行う（switch 文やマッチ式等）。
#[async_trait]
pub trait RpcHandler: Send + Sync {
    // handle はリクエストを処理してレスポンスバイト列を返す。
    // method はメソッド名（例: "CreateSession"）。
    // req はシリアライズ済みリクエストバイト列（protobuf 推奨）。
    async fn handle(&self, method: &str, req: Vec<u8>) -> crate::Result<Vec<u8>>;
}
