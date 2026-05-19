// spire_workload.rs — SPIRE Workload API 経由で X.509-SVID を取得する adapter
// spec 04 §mTLS workload identity: SPIRE agent socket から SVID を取得する
// tonic gRPC + tokio UnixStream で SPIRE Workload API の FetchX509SVID RPC を呼び出す

// thiserror: エラー型の derive macro をインポートする
use thiserror::Error;
// tonic: gRPC フレームワーク（Endpoint + Grpc client）
use tonic::transport::Endpoint;
// tonic::client::Grpc: 生の gRPC チャネルクライアント（コード生成不要）
use tonic::client::Grpc;
// tonic::codec::ProstCodec: protobuf シリアライズ/デシリアライズコーデック
use tonic::codec::ProstCodec;
// tonic::Request: gRPC リクエストラッパー
use tonic::Request;
// http::Uri: tonic Endpoint の connect_with_connector に渡す URI 型
use http::Uri;
// tower::service_fn: クロージャを Service トレイトに変換するヘルパー
use tower::service_fn;
// hyper_util::rt::TokioIo: tokio の非同期 IO を hyper が受け入れる IO トレイトにラップする
use hyper_util::rt::TokioIo;
// tracing: 構造化ロギング
use tracing::{debug, warn};

// SPIRE agent の Unix socket のデフォルトパス
const SPIRE_SOCKET_PATH: &str = "/run/spire/sockets/agent.sock";

// ---- SPIFFE Workload API proto 型定義 ----
// workload.proto を手書きで prost::Message derive して定義する。
// buf generate を使わずに protobuf ワイヤーフォーマットと互換にする。
// 参照: https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Workload_Endpoint.md

// X509SVIDRequest は FetchX509SVID RPC のリクエスト型（空メッセージ）
#[derive(prost::Message, Clone)]
pub struct X509SvidRequest {}

// X509SVID は X.509 形式の SPIFFE SVID を表す proto メッセージ
#[derive(prost::Message, Clone)]
pub struct X509Svid {
    // spiffe_id: SPIFFE ID（例: spiffe://k1s0.io/ns/default/sa/bfl）
    #[prost(string, tag = "1")]
    pub spiffe_id: String,
    // x509_svid: DER エンコードされた X.509 証明書チェーン
    #[prost(bytes = "vec", tag = "2")]
    pub x509_svid: Vec<u8>,
    // x509_svid_key: DER エンコードされた秘密鍵（PKCS#8）
    #[prost(bytes = "vec", tag = "3")]
    pub x509_svid_key: Vec<u8>,
    // bundle: トラストバンドル証明書（DER エンコード）
    #[prost(bytes = "vec", tag = "4")]
    pub bundle: Vec<u8>,
}

// X509SVIDResponse は FetchX509SVID RPC のレスポンス型（複数 SVID を含む）
#[derive(prost::Message, Clone)]
pub struct X509SvidResponse {
    // svids: 取得した SVID の一覧（通常 1 件）
    #[prost(message, repeated, tag = "1")]
    pub svids: Vec<X509Svid>,
    // bundle_update: トラストバンドルの更新情報（使用しない場合は空）
    #[prost(bytes = "vec", tag = "2")]
    pub bundle: Vec<u8>,
}

// SpireWorkloadClient は SPIRE agent と通信して X.509-SVID を取得するクライアント
pub struct SpireWorkloadClient {
    // SPIRE agent の Unix socket パス（環境によって変更可能）
    socket_path: String,
}

impl SpireWorkloadClient {
    // new はデフォルトの socket パス（/run/spire/sockets/agent.sock）でクライアントを生成する
    pub fn new() -> Self {
        // SPIRE のデフォルト socket パスを使用してインスタンスを作成する
        Self { socket_path: SPIRE_SOCKET_PATH.to_string() }
    }

    // with_socket はカスタム socket パスでクライアントを生成する（テスト環境やカスタム SPIRE 設定に使用する）
    pub fn with_socket(path: &str) -> Self {
        // 指定された socket パスを使用してインスタンスを作成する
        Self { socket_path: path.to_string() }
    }

    // fetch_x509_svid は SPIRE agent から X.509-SVID を取得する
    // tonic gRPC over Unix socket で SpiffeWorkloadAPI.FetchX509SVID RPC を呼び出す
    pub async fn fetch_x509_svid(&self) -> Result<X509SvidInfo, SpireError> {
        // socket ファイルが存在するか確認する（agent が起動していないと socket がない）
        if !std::path::Path::new(&self.socket_path).exists() {
            // socket が存在しない場合はエラーを返す（テスト環境ではモック使用を推奨）
            return Err(SpireError::SocketNotFound(self.socket_path.clone()));
        }

        // ---- tonic gRPC チャネルを Unix socket 経由で接続する ----

        // socket_path を clone してクロージャに移動する
        let socket_path = self.socket_path.clone();
        // tonic Endpoint を構築する（Unix socket では URI は dummy URL で OK）
        let endpoint = Endpoint::from_static("http://localhost:0");
        // Unix socket コネクターを構築する（service_fn で closure を Service に変換する）
        let channel = endpoint
            .connect_with_connector(service_fn(move |_: Uri| {
                // socket_path を再 clone してクロージャ内で使用する
                let path = socket_path.clone();
                // 非同期でUnix socket に接続する
                async move {
                    // tokio::net::UnixStream で SPIRE agent に接続する
                    let stream = tokio::net::UnixStream::connect(&path)
                        .await
                        .map_err(|e| SpireError::ConnectionFailed(e.to_string()))?;
                    // hyper_util::rt::TokioIo で tokio IO を hyper が受け入れる IO にラップする
                    Ok::<_, SpireError>(TokioIo::new(stream))
                }
            }))
            .await
            .map_err(|e| SpireError::ConnectionFailed(e.to_string()))?;

        // ---- gRPC FetchX509SVID RPC を呼び出す ----

        // Grpc クライアントをチャネルから生成する
        let mut grpc_client: Grpc<_> = Grpc::new(channel);
        // channel が ready になるまで待機する
        grpc_client.ready().await
            .map_err(|e| SpireError::ConnectionFailed(format!("gRPC channel not ready: {e}")))?;

        // FetchX509SVID の RPC パスを設定する
        // SPIFFE Workload Endpoint spec の service 名: SpiffeWorkloadAPI
        let path: http::uri::PathAndQuery =
            "/SpiffeWorkloadAPI/FetchX509SVID".parse()
            .map_err(|e| SpireError::ConnectionFailed(format!("invalid RPC path: {e}")))?;

        // ProstCodec を生成する（X509SvidRequest → X509SvidResponse のコーデック）
        let codec: ProstCodec<X509SvidRequest, X509SvidResponse> = ProstCodec::default();

        // server-streaming RPC を呼び出す（FetchX509SVID はサーバーストリーミング）
        let mut stream = grpc_client
            .server_streaming(Request::new(X509SvidRequest {}), path, codec)
            .await
            .map_err(|e| SpireError::RpcFailed(e.to_string()))?
            .into_inner();

        // ストリームから最初のレスポンスを取得する
        let response = stream
            .message()
            .await
            .map_err(|e| SpireError::RpcFailed(e.to_string()))?
            .ok_or_else(|| SpireError::EmptyResponse)?;

        // レスポンスに SVID が含まれているか確認する
        let svid = response
            .svids
            .into_iter()
            .next()
            .ok_or_else(|| SpireError::EmptyResponse)?;

        // SPIFFE ID が空でないことを確認する
        if svid.spiffe_id.is_empty() {
            // 空の SPIFFE ID はエラーとする
            return Err(SpireError::InvalidSvid("empty spiffe_id".to_string()));
        }

        // 取得成功をログに記録する
        debug!(spiffe_id = %svid.spiffe_id, "X.509 SVID fetched from SPIRE agent");

        // X509SvidInfo に変換して返す
        Ok(X509SvidInfo {
            // SPIFFE ID を設定する
            spiffe_id: svid.spiffe_id,
            // DER エンコードされた証明書チェーンを設定する
            x509_der: svid.x509_svid,
            // DER エンコードされた秘密鍵を設定する
            private_key_der: svid.x509_svid_key,
        })
    }
}

// Default trait を実装して SpireWorkloadClient::new() をデフォルト生成に使えるようにする
impl Default for SpireWorkloadClient {
    // Default::default() は SpireWorkloadClient::new() と同等
    fn default() -> Self {
        // デフォルトコンストラクタを呼び出す
        Self::new()
    }
}

// X509SvidInfo は取得した X.509-SVID の公開情報を保持する構造体
// 秘密鍵は zeroize で使用後にメモリからクリアする必要がある
pub struct X509SvidInfo {
    // SPIFFE ID（例: spiffe://k1s0.io/workload/bfl）
    pub spiffe_id: String,
    // DER エンコードされた X.509 証明書チェーン
    pub x509_der: Vec<u8>,
    // DER エンコードされた秘密鍵（使用後は必ず zeroize する）
    pub private_key_der: Vec<u8>,
}

// SpireError は SPIRE クライアントのエラー型（thiserror derive で Display を自動生成する）
#[derive(Debug, Error)]
pub enum SpireError {
    // SPIRE agent の socket ファイルが存在しない場合のエラー
    #[error("SPIRE agent socket not found: {0}")]
    SocketNotFound(String),
    // gRPC 接続に失敗した場合のエラー（tonic transport エラーをラップする）
    #[error("SPIRE gRPC connection failed: {0}")]
    ConnectionFailed(String),
    // gRPC RPC 呼び出しに失敗した場合のエラー（tonic Status エラーをラップする）
    #[error("SPIRE FetchX509SVID RPC failed: {0}")]
    RpcFailed(String),
    // レスポンスが空の場合のエラー
    #[error("SPIRE returned empty X509SVID response")]
    EmptyResponse,
    // 無効な SVID が返された場合のエラー
    #[error("SPIRE returned invalid SVID: {0}")]
    InvalidSvid(String),
}

// SPIRE workload client のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // 存在しない socket パスを指定した場合に SocketNotFound エラーが返ることを確認する
    #[tokio::test]
    async fn test_fetch_x509_svid_socket_not_found() {
        // 存在しない socket パスでクライアントを生成する
        let client = SpireWorkloadClient::with_socket("/nonexistent/spire.sock");
        // fetch_x509_svid を呼び出す（SocketNotFound エラーが返るはず）
        let result = client.fetch_x509_svid().await;
        // エラーが返ることを確認する
        assert!(result.is_err(), "存在しない socket パスでは SocketNotFound エラーを返すべき");
        // エラーの種類が SocketNotFound であることを確認する
        assert!(matches!(result.unwrap_err(), SpireError::SocketNotFound(_)));
    }

    // Default を使った生成テスト
    #[test]
    fn test_spire_workload_client_default() {
        // Default::default() でクライアントを生成する
        let client = SpireWorkloadClient::default();
        // socket_path がデフォルト値であることを確認する
        assert_eq!(client.socket_path, SPIRE_SOCKET_PATH, "デフォルト socket パスが期待値と異なる");
    }
}
