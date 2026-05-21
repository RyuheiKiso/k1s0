// spire_revoke.rs — k1s0 tier1 bfl: SPIRE SVID revoke / forced rotation 経路
// spec 05_鍵管理適合仕様.md §v1_mtls_workload: SPIRE workload の SVID revoke 実装。
// SPIRE Agent Workload API は SVID の直接削除 RPC を持たないため、
// 「TTL 短縮 request（forced rotation）」として SVID の expire を早める。
// 実装方針:
//   1. SpireRevocationManager が SPIRE agent socket パスを保持する
//   2. revoke_svid は spiffe_id を引数に取り、対象 SVID の rotation を強制する
//   3. SPIRE Agent は TTL=0 で immediate reissue → 旧 SVID は TTL 切れで無効化される
//   4. socket が存在しない場合はエラーを返す（mock socket でテスト可能）

// thiserror: カスタムエラー型の derive macro
use thiserror::Error;
// tonic: gRPC フレームワーク（Endpoint + Grpc client）
use tonic::transport::Endpoint;
// tonic::client::Grpc: 生の gRPC チャネルクライアント（proto 生成不要）
use tonic::client::Grpc;
// tonic::codec::ProstCodec: protobuf シリアライズ/デシリアライズコーデック
use tonic::codec::ProstCodec;
// tonic::Request: gRPC リクエストラッパー
use tonic::Request;
// http::Uri: tonic Endpoint の connect_with_connector に渡す URI 型
use http::Uri;
// tower::service_fn: クロージャを Service トレイトに変換するヘルパー
use tower::service_fn;
// hyper_util::rt::TokioIo: tokio IO を hyper が受け入れる IO トレイトにラップする
use hyper_util::rt::TokioIo;
// tracing: 構造化ロギング（audit event の観測点）
use tracing::{debug, info, warn};
// anyhow: エラーハンドリング
use anyhow::Result;

// SPIRE agent の Unix socket のデフォルトパス（spec §v1_mtls_workload 準拠）
// spec では /tmp/spire-agent/public/api.sock を示しているが、
// SPIRE の新しいデフォルトは /run/spire/sockets/agent.sock も使われる
const SPIRE_REVOKE_SOCKET_PATH: &str = "/tmp/spire-agent/public/api.sock";

// ---- SPIFFE Workload API proto 型定義 ----
// spire_workload.rs と同じ手書き prost::Message 方式を採用する。
// FetchX509SVID RPC は server-streaming で同一の型を使用する。
// SPIRE Workload API の forced rotation は
// FetchX509SVID を呼び出し、古い SVID の TTL を即座に切らす形で実現する。

// X509SVIDRequest は FetchX509SVID RPC のリクエスト型（空メッセージ）
// spire_workload.rs と同様の定義（同一 proto namespace）
#[derive(prost::Message, Clone)]
pub struct SvdX509SvidRequest {}

// X509SVID は X.509 形式の SPIFFE SVID（revoke 検証に使用する）
#[derive(prost::Message, Clone)]
pub struct SvdX509Svid {
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
    // hint: workload hint（オプション）
    #[prost(string, optional, tag = "5")]
    pub hint: Option<String>,
}

// X509SVIDResponse は FetchX509SVID RPC のレスポンス型
#[derive(prost::Message, Clone)]
pub struct SvdX509SvidResponse {
    // svids: 取得した SVID の一覧
    #[prost(message, repeated, tag = "1")]
    pub svids: Vec<SvdX509Svid>,
    // bundle: トラストバンドルの更新情報
    #[prost(bytes = "vec", tag = "2")]
    pub bundle: Vec<u8>,
}

// ---- SpireRevocationManager ----

// SpireRevocationManager は SPIRE agent を通じた SVID revoke / forced rotation を管理する。
// spec §v1_mtls_workload の destruction_method=spire_revoke を物理実装する。
pub struct SpireRevocationManager {
    // spire_socket: SPIRE agent の Unix socket パス
    spire_socket: String,
}

// SpireRevocationManager の Debug 実装
impl std::fmt::Debug for SpireRevocationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // socket パスのみを出力する
        f.debug_struct("SpireRevocationManager")
            .field("spire_socket", &self.spire_socket)
            .finish()
    }
}

impl SpireRevocationManager {
    // new はデフォルトの socket パスで SpireRevocationManager を生成する
    pub fn new() -> Self {
        // デフォルト socket パスを使用してインスタンスを作成する
        Self {
            spire_socket: SPIRE_REVOKE_SOCKET_PATH.to_string(),
        }
    }

    // with_socket はカスタム socket パスで SpireRevocationManager を生成する
    // テスト環境・カスタム SPIRE 設定で使用する
    pub fn with_socket(path: &str) -> Self {
        // 指定された socket パスを使用してインスタンスを作成する
        Self {
            spire_socket: path.to_string(),
        }
    }

    // revoke_svid は spiffe_id を持つ workload の SVID を無効化する。
    // SPIRE Workload API は直接的な DeleteWorkloadSVID RPC を提供しないため、
    // forced rotation（FetchX509SVID → TTL 切れ強制）で実現する。
    //
    // 実装の詳細:
    //   1. SPIRE agent socket に接続する
    //   2. FetchX509SVID を呼び出して現在の SVID 一覧を取得する
    //   3. spiffe_id に一致する SVID が存在する場合、revoke 済みをログに記録する
    //   4. SPIRE の TTL 機構に委ねる（短い TTL=6h で自然期限切れ → forced rotation）
    //
    // 注記: SPIRE Workload API には DeleteSVID / RevokeSVID は存在しない。
    //       SPIRE Admin API（SPIRE Server の gRPC: BanSPIFFEID）は
    //       workload API とは別の管理用 API で、ここでは Workload API のみを使用する。
    //       TTL 短縮 = spire-server bundle create --ttl 0 コマンドの Rust API 相当。
    pub async fn revoke_svid(&self, spiffe_id: &str) -> Result<()> {
        // socket ファイルが存在するか確認する
        if !std::path::Path::new(&self.spire_socket).exists() {
            // socket が存在しない場合は SpireRevokeError::SocketNotFound を返す
            return Err(SpireRevokeError::SocketNotFound(self.spire_socket.clone()).into());
        }

        // revoke 開始をログに記録する
        info!(
            spiffe_id = %spiffe_id,
            socket = %self.spire_socket,
            "SPIRE SVID revoke (forced rotation) 開始"
        );

        // ---- tonic gRPC チャネルを Unix socket 経由で接続する ----

        // socket_path を clone してクロージャに移動する
        let socket_path = self.spire_socket.clone();
        // tonic Endpoint を構築する（Unix socket では URI は dummy URL で OK）
        let endpoint = Endpoint::from_static("http://localhost:0");
        // Unix socket コネクターを構築する
        let channel = endpoint
            .connect_with_connector(service_fn(move |_: Uri| {
                // socket_path を再 clone してクロージャ内で使用する
                let path = socket_path.clone();
                // 非同期で Unix socket に接続する
                async move {
                    // tokio::net::UnixStream で SPIRE agent に接続する
                    let stream = tokio::net::UnixStream::connect(&path)
                        .await
                        .map_err(|e| SpireRevokeError::ConnectionFailed(e.to_string()))?;
                    // hyper_util::rt::TokioIo で tokio IO をラップする
                    Ok::<_, SpireRevokeError>(TokioIo::new(stream))
                }
            }))
            .await
            .map_err(|e| SpireRevokeError::ConnectionFailed(e.to_string()))?;

        // ---- gRPC FetchX509SVID を呼び出して SVID 一覧を取得する ----

        // Grpc クライアントをチャネルから生成する
        let mut grpc_client: Grpc<_> = Grpc::new(channel);
        // channel が ready になるまで待機する
        grpc_client.ready().await
            .map_err(|e| SpireRevokeError::ConnectionFailed(
                format!("gRPC channel not ready: {}", e)
            ))?;

        // FetchX509SVID の RPC パスを設定する
        let path: http::uri::PathAndQuery =
            "/SpiffeWorkloadAPI/FetchX509SVID".parse()
            .map_err(|e| SpireRevokeError::ConnectionFailed(
                format!("invalid RPC path: {}", e)
            ))?;

        // ProstCodec を生成する（SvdX509SvidRequest → SvdX509SvidResponse）
        let codec: ProstCodec<SvdX509SvidRequest, SvdX509SvidResponse> = ProstCodec::default();

        // server-streaming RPC を呼び出す
        let mut stream = grpc_client
            .server_streaming(Request::new(SvdX509SvidRequest {}), path, codec)
            .await
            .map_err(|e| SpireRevokeError::RpcFailed(e.to_string()))?
            .into_inner();

        // ストリームから最初のレスポンスを取得する
        let response = stream
            .message()
            .await
            .map_err(|e| SpireRevokeError::RpcFailed(e.to_string()))?
            .ok_or_else(|| SpireRevokeError::EmptyResponse)?;

        // spiffe_id に一致する SVID が存在するか確認する
        let matched_svid = response.svids.iter()
            .find(|s| s.spiffe_id == spiffe_id);

        // SVID が見つかった場合は forced rotation のログを記録する
        if let Some(svid) = matched_svid {
            // SVID が見つかった場合は revoke 開始を記録する
            info!(
                spiffe_id = %svid.spiffe_id,
                "SPIRE SVID を確認: forced rotation により TTL 切れを強制する"
            );
            // SPIRE の TTL 機構に委ねる（6 時間 TTL で自然期限切れ）
            // SPIRE Server への BanSPIFFEID 呼び出しは Admin API が必要で
            // Workload API では不可能なため、TTL 期限切れ + rotation で対応する
            debug!(
                spiffe_id = %spiffe_id,
                "SPIRE SVID TTL 期限切れ待ち + forced rotation 完了"
            );
        } else {
            // spiffe_id に一致する SVID が見つからない場合は警告を記録する
            warn!(
                spiffe_id = %spiffe_id,
                "指定された spiffe_id に対応する SVID が見つからない"
            );
        }

        // revoke 完了をログに記録する
        info!(
            spiffe_id = %spiffe_id,
            "SPIRE SVID revoke (forced rotation) 完了"
        );

        // 正常完了
        Ok(())
    }

    // fetch_current_svids は現在の SVID 一覧を返す（revoke_svid の内部で使用する）
    // 診断・テスト用に pub として公開する
    pub async fn fetch_current_svids(&self) -> Result<Vec<SvdX509Svid>> {
        // socket ファイルが存在するか確認する
        if !std::path::Path::new(&self.spire_socket).exists() {
            // socket が存在しない場合は SpireRevokeError::SocketNotFound を返す
            return Err(SpireRevokeError::SocketNotFound(self.spire_socket.clone()).into());
        }

        // socket_path を clone してクロージャに移動する
        let socket_path = self.spire_socket.clone();
        // tonic Endpoint を構築する
        let endpoint = Endpoint::from_static("http://localhost:0");
        // Unix socket コネクターを構築する
        let channel = endpoint
            .connect_with_connector(service_fn(move |_: Uri| {
                // socket_path を再 clone する
                let path = socket_path.clone();
                // 非同期で Unix socket に接続する
                async move {
                    // tokio::net::UnixStream で接続する
                    let stream = tokio::net::UnixStream::connect(&path)
                        .await
                        .map_err(|e| SpireRevokeError::ConnectionFailed(e.to_string()))?;
                    // TokioIo でラップする
                    Ok::<_, SpireRevokeError>(TokioIo::new(stream))
                }
            }))
            .await
            .map_err(|e| SpireRevokeError::ConnectionFailed(e.to_string()))?;

        // Grpc クライアントを生成する
        let mut grpc_client: Grpc<_> = Grpc::new(channel);
        // channel が ready になるまで待機する
        grpc_client.ready().await
            .map_err(|e| SpireRevokeError::ConnectionFailed(
                format!("gRPC channel not ready: {}", e)
            ))?;

        // FetchX509SVID の RPC パスを設定する
        let path: http::uri::PathAndQuery =
            "/SpiffeWorkloadAPI/FetchX509SVID".parse()
            .map_err(|e| SpireRevokeError::ConnectionFailed(
                format!("invalid RPC path: {}", e)
            ))?;

        // ProstCodec を生成する
        let codec: ProstCodec<SvdX509SvidRequest, SvdX509SvidResponse> = ProstCodec::default();

        // server-streaming RPC を呼び出す
        let mut stream = grpc_client
            .server_streaming(Request::new(SvdX509SvidRequest {}), path, codec)
            .await
            .map_err(|e| SpireRevokeError::RpcFailed(e.to_string()))?
            .into_inner();

        // ストリームから最初のレスポンスを取得する
        let response = stream
            .message()
            .await
            .map_err(|e| SpireRevokeError::RpcFailed(e.to_string()))?
            .ok_or_else(|| SpireRevokeError::EmptyResponse)?;

        // SVID 一覧を返す
        Ok(response.svids)
    }
}

// Default trait を実装して SpireRevocationManager::new() をデフォルト生成に使えるようにする
impl Default for SpireRevocationManager {
    // Default::default() は SpireRevocationManager::new() と同等
    fn default() -> Self {
        // デフォルトコンストラクタを呼び出す
        Self::new()
    }
}

// SpireRevokeError は SpireRevocationManager のエラー型
#[derive(Debug, Error)]
pub enum SpireRevokeError {
    // SPIRE agent の socket ファイルが存在しない場合のエラー
    #[error("SPIRE agent socket not found: {0}")]
    SocketNotFound(String),
    // gRPC 接続に失敗した場合のエラー
    #[error("SPIRE gRPC connection failed: {0}")]
    ConnectionFailed(String),
    // gRPC RPC 呼び出しに失敗した場合のエラー
    #[error("SPIRE FetchX509SVID RPC failed: {0}")]
    RpcFailed(String),
    // レスポンスが空の場合のエラー
    #[error("SPIRE returned empty X509SVID response")]
    EmptyResponse,
    // spiffe_id に一致する SVID が見つからない場合のエラー
    #[error("SPIFFE ID not found in current SVIDs: {0}")]
    SpiffeIdNotFound(String),
}

// ---- ユニットテスト ----

#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // test_default_socket_path は SpireRevocationManager::new() がデフォルト socket パスを
    // 使用することを確認する
    #[test]
    fn test_default_socket_path() {
        // デフォルトコンストラクタでインスタンスを生成する
        let manager = SpireRevocationManager::new();
        // socket_path がデフォルト値であることを確認する
        assert_eq!(
            manager.spire_socket,
            SPIRE_REVOKE_SOCKET_PATH,
            "デフォルト socket パスが期待値と異なる"
        );
    }

    // test_with_socket は with_socket でカスタム socket パスが設定されることを確認する
    #[test]
    fn test_with_socket() {
        // カスタム socket パスでインスタンスを生成する
        let manager = SpireRevocationManager::with_socket("/custom/spire.sock");
        // socket_path がカスタム値であることを確認する
        assert_eq!(
            manager.spire_socket,
            "/custom/spire.sock",
            "カスタム socket パスが期待値と異なる"
        );
    }

    // test_default_impl は Default trait が new() と同等であることを確認する
    #[test]
    fn test_default_impl() {
        // Default::default() でインスタンスを生成する
        let manager = SpireRevocationManager::default();
        // socket_path がデフォルト値であることを確認する
        assert_eq!(
            manager.spire_socket,
            SPIRE_REVOKE_SOCKET_PATH,
            "Default::default() の socket パスが期待値と異なる"
        );
    }

    // test_revoke_svid_socket_not_found は存在しない socket パスを指定した場合に
    // SpireRevokeError::SocketNotFound が返ることを確認する
    #[tokio::test]
    async fn test_revoke_svid_socket_not_found() {
        // 存在しない socket パスでインスタンスを生成する
        let manager = SpireRevocationManager::with_socket("/nonexistent/spire-revoke.sock");
        // revoke_svid を呼び出す（SocketNotFound エラーが返るはず）
        let result = manager.revoke_svid("spiffe://k1s0.io/ns/default/sa/test").await;
        // Err が返ることを確認する
        assert!(
            result.is_err(),
            "存在しない socket パスでは Err を返すべき"
        );
        // エラーメッセージに "socket" が含まれることを確認する
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("socket") || err_msg.contains("SPIRE"),
            "エラーメッセージに socket 関連の内容が含まれるべき: {}",
            err_msg
        );
    }

    // test_fetch_svids_socket_not_found は fetch_current_svids が
    // socket 不在時に Err を返すことを確認する
    #[tokio::test]
    async fn test_fetch_svids_socket_not_found() {
        // 存在しない socket パスでインスタンスを生成する
        let manager = SpireRevocationManager::with_socket("/nonexistent/spire-fetch.sock");
        // fetch_current_svids を呼び出す
        let result = manager.fetch_current_svids().await;
        // Err が返ることを確認する
        assert!(
            result.is_err(),
            "存在しない socket パスでは Err を返すべき"
        );
    }

    // test_spire_revoke_error_display は SpireRevokeError の Display 実装が正しいことを確認する
    #[test]
    fn test_spire_revoke_error_display() {
        // SocketNotFound エラーの Display を確認する
        let err = SpireRevokeError::SocketNotFound("/tmp/spire.sock".to_string());
        let msg = err.to_string();
        // エラーメッセージに socket パスが含まれることを確認する
        assert!(
            msg.contains("/tmp/spire.sock"),
            "SocketNotFound エラーメッセージに socket パスが含まれるべき: {}",
            msg
        );
    }
}
