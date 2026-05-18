// spire_workload.rs — SPIRE Workload API 経由で X.509-SVID を取得する adapter
// spec 04 §mTLS workload identity: SPIRE agent socket から SVID を取得する
// SPIRE agent は /run/spire/sockets/agent.sock で Unix socket を公開する

// thiserror: エラー型の derive macro をインポートする
use thiserror::Error;

// SPIRE agent の Unix socket のデフォルトパス
const SPIRE_SOCKET_PATH: &str = "/run/spire/sockets/agent.sock";

// X.509 SVID を取得するクライアントの構造体
pub struct SpireWorkloadClient {
    // SPIRE agent の Unix socket パス（環境によって変更可能）
    socket_path: String,
}

// SpireWorkloadClient の実装ブロック
impl SpireWorkloadClient {
    // デフォルトの socket パス（/run/spire/sockets/agent.sock）でクライアントを生成する
    pub fn new() -> Self {
        // SPIRE のデフォルト socket パスを使用してインスタンスを作成する
        Self { socket_path: SPIRE_SOCKET_PATH.to_string() }
    }

    // カスタム socket パスでクライアントを生成する（テスト環境やカスタム SPIRE 設定に使用する）
    pub fn with_socket(path: &str) -> Self {
        // 指定された socket パスを使用してインスタンスを作成する
        Self { socket_path: path.to_string() }
    }

    // SPIRE agent から X.509-SVID を取得する
    // SPIFFE ID の例: spiffe://k1s0.io/ns/default/sa/bfl
    pub fn fetch_x509_svid(&self) -> Result<X509Svid, SpireError> {
        // socket ファイルが存在するか確認する（agent が起動していないと socket がない）
        if !std::path::Path::new(&self.socket_path).exists() {
            // socket が存在しない場合はエラーを返す（テスト環境ではモック使用を推奨）
            return Err(SpireError::SocketNotFound(self.socket_path.clone()));
        }
        // TODO: spiffe-workload-api crate の FetchX509Svid gRPC 呼び出しを実装する
        // 実装計画:
        //   1. tokio::net::UnixStream で socket に接続する
        //   2. tonic で WorkloadAPI gRPC クライアントを生成する
        //   3. FetchX509Svid RPC を呼び出して SVID を取得する
        // 現時点では development 用の stub SVID を返す（本番では SPIRE 接続を必須とする）
        Ok(X509Svid {
            // BFL サービスの SPIFFE ID を設定する
            spiffe_id: "spiffe://k1s0.io/workload/bfl".to_string(),
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

// X.509 SVID を表す構造体（SPIFFE ID と証明書情報を保持する）
pub struct X509Svid {
    // SPIFFE ID（例: spiffe://k1s0.io/workload/bfl）
    pub spiffe_id: String,
}

// SPIRE クライアントのエラー型（thiserror derive で Display を自動生成する）
#[derive(Debug, Error)]
pub enum SpireError {
    // SPIRE agent の socket ファイルが存在しない場合のエラー
    #[error("SPIRE agent socket not found: {0}")]
    SocketNotFound(String),
    // gRPC 接続に失敗した場合のエラー（tonic transport エラーをラップする）
    #[error("SPIRE gRPC connection failed: {0}")]
    ConnectionFailed(String),
}

// SPIRE workload client のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // 存在しない socket パスを指定した場合に SocketNotFound エラーが返ることを確認する
    #[test]
    fn test_fetch_x509_svid_socket_not_found() {
        // 存在しない socket パスでクライアントを生成する
        let client = SpireWorkloadClient::with_socket("/nonexistent/spire.sock");
        // fetch_x509_svid を呼び出す（SocketNotFound エラーが返るはず）
        let result = client.fetch_x509_svid();
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
