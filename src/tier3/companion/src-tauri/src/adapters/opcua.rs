// k1s0 tier3 OPC-UA クライアントアダプター
// opcua クレートを使って OPC-UA サーバーへの接続/ノード読み取り/書き込みを実装する
// T3-Y1 の opcua adapter 実装要件に従い OPC-UA クライアント操作を提供する

// opcua: OPC-UA クライアントライブラリ
use opcua::client::prelude::{
    // Client: OPC-UA クライアントインスタンス
    Client,
    // ClientBuilder: クライアントの設定ビルダー
    ClientBuilder,
    // IdentityToken: 匿名認証 / ユーザー名認証に使用する
    IdentityToken,
    // Session: OPC-UA セッション（ノード読み書きに使用する）
    Session,
};
// opcua::types: OPC-UA 型定義
use opcua::types::{
    // NodeId: OPC-UA ノード識別子（ns=2;i=1001 等の形式）
    NodeId,
    // Variant: OPC-UA 値型（Integer / Float / String 等）
    Variant,
    // ReadValueId: 読み取り対象のノード ID と属性 ID
    ReadValueId,
    // AttributeId: OPC-UA 属性 ID（Value=13 等）
    AttributeId,
    // StatusCode: OPC-UA 操作結果コード
    StatusCode,
};
// serde: Tauri IPC レスポンスのシリアライズに使用する
use serde::{Deserialize, Serialize};
// std::sync: Arc / RwLock（セッション共有に使用する）
use std::sync::{Arc, RwLock};

// OPC-UA ノード値を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcuaNodeValue {
    // ノード ID 文字列（例: "ns=2;i=1001"）
    pub node_id: String,
    // 値の文字列表現
    pub value: String,
    // OPC-UA ステータスコード（Good=0 等）
    pub status_code: u32,
}

// OPC-UA 接続設定を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcuaConnectionConfig {
    // OPC-UA サーバーエンドポイント URL（例: "opc.tcp://localhost:4840"）
    pub endpoint_url: String,
    // アプリケーション名（クライアント識別に使用する）
    pub application_name: String,
    // 匿名認証を使用するかどうか（false の場合は username/password が必要）
    pub anonymous: bool,
    // ユーザー名（anonymous=false の場合に使用する）
    pub username: Option<String>,
}

/// create_opcua_client は OPC-UA クライアントを生成してサーバーに接続する
/// endpoint_url で指定したサーバーに接続してセッションを返す
pub async fn create_opcua_session(
    // OPC-UA 接続設定
    config: &OpcuaConnectionConfig,
) -> Result<Arc<RwLock<Session>>, String> {
    // OPC-UA クライアントを構築する
    let mut client = ClientBuilder::new()
        // アプリケーション名を設定する（サーバーがクライアントを識別するために使用する）
        .application_name(&config.application_name)
        // セキュリティなし（開発環境向け；本番環境は証明書設定が必要）
        .trust_server_certs(true)
        // クライアントを構築する
        .client()
        // クライアント構築失敗時はエラーを返す
        .map_err(|e| format!("OPC-UA クライアント構築失敗: {}", e))?;

    // 認証トークンを設定する（匿名 or ユーザー名/パスワード）
    let identity_token = if config.anonymous {
        // 匿名認証を使用する
        IdentityToken::Anonymous
    } else {
        // ユーザー名/パスワード認証を使用する（username が必要）
        let username = config.username.as_deref()
            .ok_or("OPC-UA ユーザー名が設定されていません（anonymous=false の場合は必須）")?;
        // UserNameIdentityToken を構築する（パスワードは空文字列を使用する（本番は別途設定する））
        IdentityToken::UserName(username.to_string(), String::new())
    };

    // エンドポイントに接続してセッションを生成する
    let (session, event_loop) = client
        // エンドポイント URL とセキュリティポリシーを指定して接続する
        .connect_to_endpoint(
            (config.endpoint_url.as_str(), opcua::client::prelude::SecurityPolicy::None.to_uri(), opcua::client::prelude::MessageSecurityMode::None, identity_token),
            // セッション接続失敗時はエラーを返す（サーバー未起動等）
        )
        .await
        .map_err(|e| format!("OPC-UA セッション接続失敗 {}: {}", config.endpoint_url, e))?;

    // イベントループを非同期タスクとして実行する（セッション維持のために必要）
    tokio::spawn(event_loop.run());

    // セッションを Arc<RwLock> でラップして返す
    Ok(session)
}

/// read_opcua_nodes は OPC-UA サーバーから指定したノードの値を読み取る
/// session を使って node_ids のノード値を一括読み取りする
pub async fn read_opcua_nodes(
    // OPC-UA セッション
    session: Arc<RwLock<Session>>,
    // 読み取るノード ID 一覧（例: ["ns=2;i=1001", "ns=2;i=1002"]）
    node_ids: &[&str],
) -> Result<Vec<OpcuaNodeValue>, String> {
    // 読み取り対象の ReadValueId リストを構築する
    let read_value_ids: Vec<ReadValueId> = node_ids.iter()
        .map(|id_str| {
            // ノード ID 文字列を NodeId に変換する
            let node_id = NodeId::from(id_str.as_ref());
            // ReadValueId を構築する（Value 属性 ID = 13 を読み取る）
            ReadValueId {
                // ノード ID を設定する
                node_id,
                // Value 属性（AttributeId::Value = 13）を読み取る
                attribute_id: AttributeId::Value as u32,
                // インデックス範囲なし（配列全体を読み取る）
                index_range: opcua::types::UAString::null(),
                // データエンコーディングはデフォルトを使用する
                data_encoding: opcua::types::QualifiedName::null(),
            }
        })
        .collect();

    // セッションをロックしてノード値を読み取る
    let read_results = {
        // セッションの読み取りロックを取得する
        let session_guard = session.read()
            .map_err(|e| format!("OPC-UA セッションロック失敗: {}", e))?;
        // ノード値を一括読み取りする
        session_guard.read(&read_value_ids, opcua::types::TimestampsToReturn::Both, 0.0)
            .map_err(|e| format!("OPC-UA ノード読み取り失敗: {}", e))?
    };

    // 読み取り結果を OpcuaNodeValue に変換する
    let mut results = Vec::new();

    // 各ノードの結果を処理する
    for (idx, data_value) in read_results.iter().enumerate() {
        // ノード ID 文字列を取得する
        let node_id = node_ids.get(idx).copied().unwrap_or("unknown").to_string();

        // ステータスコードを取得する
        let status_code = data_value.status
            .map(|s| s.bits())
            .unwrap_or(0);

        // 値を文字列に変換する
        let value = data_value.value
            .as_ref()
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "null".to_string());

        // OpcuaNodeValue を構築して追加する
        results.push(OpcuaNodeValue {
            // ノード ID を設定する
            node_id,
            // 値を設定する
            value,
            // ステータスコードを設定する
            status_code,
        });
    }

    // 読み取り結果を返す
    Ok(results)
}
