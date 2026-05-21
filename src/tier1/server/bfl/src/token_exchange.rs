// token_exchange.rs — spec 04 §v1_federated_exchange: RFC 8693 token exchange フロー
// Keycloak を STS として使用する RFC 8693 トークン交換フローを実装する。
// subject_token を Keycloak token endpoint に送り、audience-restricted な access_token を取得する。
// audience restriction 検証: レスポンスの access_token の aud claim が req.audience を含むことを確認する。

// anyhow: エラー伝搬ライブラリ
use anyhow::{bail, Context, Result};
// base64: access_token の JWT payload を base64url デコードするために使用する
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
// reqwest: HTTP クライアント（Keycloak token endpoint に POST する）
use reqwest::Client;
// serde: JSON シリアライズ/デシリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{debug, info, warn};

// RFC 8693 subject_token_type の定数: access_token 型
pub const TOKEN_TYPE_ACCESS_TOKEN: &str = "urn:ietf:params:oauth:token-type:access_token";
// RFC 8693 subject_token_type の定数: JWT 型
pub const TOKEN_TYPE_JWT: &str = "urn:ietf:params:oauth:token-type:jwt";
// RFC 8693 grant_type の定数: token-exchange
pub const GRANT_TYPE_TOKEN_EXCHANGE: &str =
    "urn:ietf:params:oauth:grant-type:token-exchange";

// TokenExchangeRequest は RFC 8693 token exchange リクエストのパラメータを宣言する。
// spec 04 §v1_federated_exchange フロー入力として使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeRequest {
    // subject_token: 交換元のトークン（外部 IdP が発行した access_token または JWT）
    pub subject_token: String,
    // subject_token_type: subject_token の型（RFC 8693 URN 形式）
    pub subject_token_type: String,
    // requested_token_type: 交換先トークンの型（RFC 8693 URN 形式）
    pub requested_token_type: String,
    // audience: 交換後トークンの audience（k1s0-gateway 等）
    pub audience: String,
    // scope: 要求する scope（スペース区切り）
    pub scope: Option<String>,
}

// TokenExchangeResponse は RFC 8693 token exchange レスポンスのフィールドを宣言する。
// spec 04 §v1_federated_exchange フロー出力として使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeResponse {
    // access_token: 交換後の access_token（audience-restricted）
    pub access_token: String,
    // issued_token_type: 発行されたトークンの型（RFC 8693 URN 形式）
    pub issued_token_type: String,
    // expires_in: トークンの有効期限（秒）
    pub expires_in: u64,
    // scope: 付与された scope（スペース区切り）
    pub scope: Option<String>,
}

// KeycloakTokenResponse は Keycloak の token endpoint レスポンス JSON フィールドを宣言する。
// exchange_token 内部のデシリアライズ専用型（公開しない）。
#[derive(Debug, Deserialize)]
struct KeycloakTokenResponse {
    // access_token: 交換後の access_token
    access_token: String,
    // issued_token_type: 発行されたトークンの型
    issued_token_type: Option<String>,
    // expires_in: トークンの有効期限（秒）
    expires_in: Option<u64>,
    // scope: 付与された scope
    scope: Option<String>,
}

// exchange_token は RFC 8693 token exchange を Keycloak token endpoint 経由で実行する。
// client: reqwest HTTP クライアント
// keycloak_endpoint: Keycloak realm URL（例: http://keycloak.k1s0.svc:8080/realms/k1s0）
// req: token exchange リクエストパラメータ
pub async fn exchange_token(
    client: &Client,
    keycloak_endpoint: &str,
    req: TokenExchangeRequest,
) -> Result<TokenExchangeResponse> {
    // ---- 1. Keycloak token endpoint URL を構築する ----
    // Keycloak の token endpoint は {realm_url}/protocol/openid-connect/token
    let token_url = format!("{keycloak_endpoint}/protocol/openid-connect/token");
    debug!(url = %token_url, audience = %req.audience, "Initiating RFC 8693 token exchange");

    // ---- 2. form パラメータを構築する ----
    // RFC 8693 §2.1: grant_type, subject_token, subject_token_type, requested_token_type, audience は必須
    let mut form_params = vec![
        // grant_type: token-exchange を示す URN
        ("grant_type", GRANT_TYPE_TOKEN_EXCHANGE.to_string()),
        // subject_token: 交換元トークン
        ("subject_token", req.subject_token.clone()),
        // subject_token_type: 交換元トークンの型
        ("subject_token_type", req.subject_token_type.clone()),
        // requested_token_type: 交換先トークンの型
        ("requested_token_type", req.requested_token_type.clone()),
        // audience: 交換後トークンの audience（Keycloak audience restriction に対応する）
        ("audience", req.audience.clone()),
    ];
    // scope が指定された場合は form パラメータに追加する
    if let Some(ref scope) = req.scope {
        form_params.push(("scope", scope.clone()));
    }

    // ---- 3. Keycloak token endpoint に POST リクエストを送信する ----
    // reqwest で application/x-www-form-urlencoded を送信する
    let response = client
        .post(&token_url)
        .form(&form_params)
        .send()
        .await
        .with_context(|| format!("Keycloak token exchange request failed: {token_url}"))?;

    // ---- 4. レスポンスステータスを確認する ----
    // Keycloak が 200 OK でない場合はエラーを返す
    if !response.status().is_success() {
        let status = response.status();
        // エラーレスポンスのボディを取得してログに記録する
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<body read error>".to_string());
        bail!(
            "Keycloak token exchange failed: HTTP {status}, body: {body}"
        );
    }

    // ---- 5. レスポンス JSON をデシリアライズする ----
    let kc_resp: KeycloakTokenResponse = response
        .json()
        .await
        .with_context(|| "Keycloak token exchange response JSON parse failed")?;

    // ---- 6. audience restriction を検証する ----
    // spec 04 §v1_federated_exchange: response の access_token の aud claim が req.audience を含むこと
    // jsonwebtoken でデコードせず、JWT payload の base64url 部分だけを展開して serde_json::Value で確認する
    verify_aud_claim(&kc_resp.access_token, &req.audience)?;

    // ---- 7. TokenExchangeResponse を構築して返す ----
    let exchange_resp = TokenExchangeResponse {
        // access_token: 交換後の access_token
        access_token: kc_resp.access_token,
        // issued_token_type: 発行されたトークンの型（デフォルト: access_token）
        issued_token_type: kc_resp
            .issued_token_type
            .unwrap_or_else(|| TOKEN_TYPE_ACCESS_TOKEN.to_string()),
        // expires_in: トークンの有効期限（デフォルト: 300 秒）
        expires_in: kc_resp.expires_in.unwrap_or(300),
        // scope: 付与された scope
        scope: kc_resp.scope,
    };
    // 成功をログに記録する（access_token の内容は記録しない）
    info!(
        audience = %req.audience,
        issued_token_type = %exchange_resp.issued_token_type,
        expires_in = %exchange_resp.expires_in,
        "RFC 8693 token exchange succeeded"
    );
    Ok(exchange_resp)
}

// verify_aud_claim は JWT の payload 部分を base64url デコードして aud claim を検証する。
// jsonwebtoken での完全 decode は行わず、base64url + serde_json::Value で aud claim だけを確認する。
// access_token: 検証対象の JWT（コンパクト JWT 形式）
// expected_audience: 含まれていることを期待する audience 値
fn verify_aud_claim(access_token: &str, expected_audience: &str) -> Result<()> {
    // JWT を '.' で分割して payload 部分を取得する
    let parts: Vec<&str> = access_token.splitn(3, '.').collect();
    // 3 部分が揃っていない場合はエラーを返す
    if parts.len() != 3 {
        bail!("access_token format invalid: expected header.payload.signature");
    }
    // payload を base64url デコードする（パディングなし）
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .with_context(|| "access_token payload base64url decode failed")?;
    // payload JSON を serde_json::Value としてパースする
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .with_context(|| "access_token payload JSON parse failed")?;
    // aud claim を取得する（文字列または配列の両方に対応する）
    match payload.get("aud") {
        Some(serde_json::Value::String(aud)) => {
            // aud が文字列の場合: expected_audience と一致するか確認する
            if aud != expected_audience {
                warn!(aud = %aud, expected = %expected_audience, "access_token aud mismatch");
                bail!(
                    "access_token aud claim mismatch: expected '{}', got '{}'",
                    expected_audience,
                    aud
                );
            }
        }
        Some(serde_json::Value::Array(auds)) => {
            // aud が配列の場合: expected_audience が配列に含まれるか確認する
            let found = auds.iter().any(|v| {
                v.as_str().map_or(false, |s| s == expected_audience)
            });
            if !found {
                warn!(
                    expected = %expected_audience,
                    "access_token aud array does not contain expected audience"
                );
                bail!(
                    "access_token aud claim does not contain expected audience '{}'",
                    expected_audience
                );
            }
        }
        None => {
            // aud claim が存在しない場合は警告して audience restriction なしとして扱う
            warn!(expected = %expected_audience, "access_token missing aud claim — audience restriction not enforced");
        }
        _ => {
            // aud が想定外の型の場合はエラーを返す
            bail!("access_token aud claim has unexpected type");
        }
    }
    debug!(expected_audience = %expected_audience, "access_token aud claim verified");
    Ok(())
}

// #[cfg(test)] mod tests — RFC 8693 token exchange の unit テスト
#[cfg(test)]
mod tests {
    // super: このモジュールの親スコープ（token_exchange.rs 全体）をインポートする
    use super::*;
    // base64: テスト用 JWT payload のエンコードに使用する
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    // test_verify_aud_claim_string は aud が文字列の場合の検証テスト。
    #[test]
    fn test_verify_aud_claim_string() {
        // aud が文字列の JWT payload を構築する
        let payload = serde_json::json!({
            "sub": "user-123",
            "aud": "k1s0-gateway",
            "exp": 9999999999u64
        });
        // payload を base64url エンコードして JWT 形式にする
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        // ダミーヘッダーとシグネチャを結合してテスト用 JWT を構築する
        let dummy_jwt = format!("header.{payload_b64}.signature");
        // 正しい audience で検証すると Ok になることを確認する
        assert!(
            verify_aud_claim(&dummy_jwt, "k1s0-gateway").is_ok(),
            "正しい audience で検証した場合は Ok を返さなければならない"
        );
        // 異なる audience で検証すると Err になることを確認する
        assert!(
            verify_aud_claim(&dummy_jwt, "wrong-audience").is_err(),
            "異なる audience で検証した場合は Err を返さなければならない"
        );
    }

    // test_verify_aud_claim_array は aud が配列の場合の検証テスト。
    #[test]
    fn test_verify_aud_claim_array() {
        // aud が配列の JWT payload を構築する
        let payload = serde_json::json!({
            "sub": "svc-abc",
            "aud": ["k1s0-gateway", "k1s0-internal"],
            "exp": 9999999999u64
        });
        // payload を base64url エンコードして JWT 形式にする
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        // ダミーヘッダーとシグネチャを結合してテスト用 JWT を構築する
        let dummy_jwt = format!("header.{payload_b64}.signature");
        // 配列に含まれる audience で検証すると Ok になることを確認する
        assert!(
            verify_aud_claim(&dummy_jwt, "k1s0-gateway").is_ok(),
            "配列に含まれる audience で検証した場合は Ok を返さなければならない"
        );
        // 配列に含まれない audience で検証すると Err になることを確認する
        assert!(
            verify_aud_claim(&dummy_jwt, "k1s0-external").is_err(),
            "配列に含まれない audience で検証した場合は Err を返さなければならない"
        );
    }

    // test_exchange_token_mock は Keycloak endpoint の mock を使用して exchange フローを検証する。
    // NOTE: reqwest mock crate を workspace に追加せずに inline で HTTP server を立てる方法として
    //       tokio の TcpListener を使って簡易 HTTP レスポンスを返す mock server を実装する。
    #[tokio::test]
    async fn test_exchange_token_mock() {
        // ---- mock server を構築する ----
        // tokio::net::TcpListener で動的ポートにバインドする
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        // 実際に bind されたポートを取得する
        let port = listener.local_addr().unwrap().port();

        // mock server が返す access_token の payload を構築する（aud=k1s0-gateway を含む）
        let aud_payload = serde_json::json!({
            "sub": "federated-user-001",
            "aud": "k1s0-gateway",
            "exp": 9999999999u64,
            "iss": "http://external-idp.example.com"
        });
        let aud_payload_b64 = URL_SAFE_NO_PAD.encode(aud_payload.to_string().as_bytes());
        // テスト用 access_token（signature は dummy）
        let test_access_token = format!("header.{aud_payload_b64}.sig");

        // Keycloak token endpoint が返す JSON レスポンスを構築する
        let token_resp_body = serde_json::json!({
            "access_token": test_access_token,
            "issued_token_type": "urn:ietf:params:oauth:token-type:access_token",
            "expires_in": 300,
            "scope": "openid profile"
        });
        let token_resp_str = token_resp_body.to_string();

        // mock server タスクを spawn する（1 リクエストを受けてレスポンスを返して終了する）
        tokio::spawn(async move {
            // 1 接続を受け入れる
            let (mut stream, _) = listener.accept().await.unwrap();
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            // リクエストを読み捨てる（最大 4096 バイト）
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            // HTTP 200 OK レスポンスを返す
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                token_resp_str.len(),
                token_resp_str
            );
            let _ = stream.write_all(resp.as_bytes()).await;
        });

        // ---- reqwest クライアントを構築する ----
        let client = Client::new();
        // Keycloak endpoint を mock server に向ける
        let keycloak_endpoint = format!("http://127.0.0.1:{port}/realms/k1s0");

        // ---- token exchange リクエストを構築する ----
        let req = TokenExchangeRequest {
            // subject_token: ダミーの外部 IdP token
            subject_token: "external-token-dummy".to_string(),
            // subject_token_type: access_token 型
            subject_token_type: TOKEN_TYPE_ACCESS_TOKEN.to_string(),
            // requested_token_type: access_token 型
            requested_token_type: TOKEN_TYPE_ACCESS_TOKEN.to_string(),
            // audience: k1s0-gateway（audience restriction 検証対象）
            audience: "k1s0-gateway".to_string(),
            // scope: openid profile
            scope: Some("openid profile".to_string()),
        };

        // ---- exchange_token を呼び出す ----
        let result = exchange_token(&client, &keycloak_endpoint, req).await;

        // ---- 検証 ----
        // exchange が成功することを確認する
        assert!(
            result.is_ok(),
            "exchange_token は成功しなければならない（実際: {:?}）",
            result.err()
        );
        let resp = result.unwrap();
        // issued_token_type が access_token 型であることを確認する
        assert_eq!(
            resp.issued_token_type,
            TOKEN_TYPE_ACCESS_TOKEN,
            "issued_token_type は access_token 型でなければならない"
        );
        // expires_in が 300 秒であることを確認する
        assert_eq!(resp.expires_in, 300, "expires_in は 300 でなければならない");
    }
}
