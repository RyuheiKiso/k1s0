// k1s0-impl: IMPL-cross_bff-0001 realizes=FR-cross_bff-001
// k1s0 BFF 認証エッジゲートウェイのメインファイル
// axum を使用した httpOnly cookie / Authorization 中継 / CSRF / CORS / PKCE 対応 BFF
use axum::{
    // axum ルーター: ルーティング設定に使用する
    Router,
    // axum ルート設定関数をインポートする
    routing::{get, post},
    // axum リクエスト/レスポンス型をインポートする
    extract::{State, Json, Query},
    // axum レスポンス型をインポートする
    response::{IntoResponse, Response},
    // axum HTTP ステータスコードをインポートする
    http::StatusCode,
};
// HTTP ヘッダー型をインポートする
use http::{HeaderMap, HeaderValue, header};
// serde のデシリアライズトレイトをインポートする
use serde::{Deserialize, Serialize};
// JSON Web Token ライブラリをインポートする
use jsonwebtoken::{decode, DecodingKey, Validation};
// UUID ライブラリをインポートする (CSRF トークン生成に使用する)
use uuid::Uuid;
// Arc で共有状態をスレッドセーフに管理する
use std::sync::Arc;
// RwLock で共有状態を読み書きロックで保護する
use tokio::sync::RwLock;
// HashMap でセッション/CSRF トークンを管理する
use std::collections::HashMap;
// tracing でログを記録する
use tracing::{info, warn, error};
// anyhow エラーハンドリングをインポートする
use anyhow::Result;

// BFF アプリケーション設定を保持する構造体
#[derive(Clone, Debug)]
struct AppConfig {
    // OIDC プロバイダーの発行者 URL (IdP エンドポイント)
    oidc_issuer: String,
    // クライアント ID: OAuth2/PKCE フローに使用する
    client_id: String,
    // クライアントシークレット: トークン交換に使用する (Confidential クライアント用)
    client_secret: String,
    // リダイレクト URI: PKCE コールバック先 URI
    redirect_uri: String,
    // Cookie セキュア設定: 本番環境では true に設定する
    cookie_secure: bool,
    // CORS 許可オリジンリスト
    allowed_origins: Vec<String>,
}

// セッション情報を保持する構造体
#[derive(Clone, Debug)]
struct Session {
    // アクセストークン: IdP から取得したアクセストークン
    access_token: String,
    // リフレッシュトークン: silent renew に使用するリフレッシュトークン
    refresh_token: Option<String>,
    // CSRF トークン: CSRF 攻撃防止用の乱数トークン
    csrf_token: String,
    // セッション有効期限: Unix タイムスタンプ
    expires_at: u64,
}

// アプリケーション共有状態を保持する構造体
#[derive(Clone)]
struct AppState {
    // アプリケーション設定: Arc で共有する
    config: Arc<AppConfig>,
    // セッションストア: セッション ID → Session のマップ
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    // HTTP クライアント: IdP との通信に使用する
    http_client: reqwest::Client,
}

// PKCE 認証コードフローの認可リクエストパラメータ
#[derive(Deserialize)]
struct AuthorizeParams {
    // state パラメータ: CSRF 防止用の不透明文字列
    state: Option<String>,
}

// PKCE コールバックパラメータ
#[derive(Deserialize)]
struct CallbackParams {
    // 認可コード: IdP から受け取る認可コード
    code: String,
    // state パラメータ: 認可リクエスト時の state と照合する
    state: Option<String>,
}

// トークンレスポンスの JSON 構造体
#[derive(Deserialize, Serialize)]
struct TokenResponse {
    // アクセストークン: IdP から受け取るアクセストークン
    access_token: String,
    // リフレッシュトークン: IdP から受け取るリフレッシュトークン
    refresh_token: Option<String>,
    // トークン有効期限: 秒数
    expires_in: u64,
    // トークンタイプ: "Bearer"
    token_type: String,
}

// バックチャネルログアウトリクエストの JSON 構造体
#[derive(Deserialize)]
struct BackchannelLogoutRequest {
    // ログアウトトークン: IdP から送信される logout_token
    logout_token: String,
}

// JWT クレームの構造体 (最小限の必要フィールドのみ定義する)
#[derive(Deserialize)]
struct Claims {
    // サブジェクト: ユーザー識別子
    sub: String,
    // セッション ID: バックチャネルログアウトに使用する
    sid: Option<String>,
    // 有効期限: Unix タイムスタンプ
    exp: u64,
}

// httpOnly セキュア Cookie を設定するヘルパー関数
fn build_session_cookie(
    // セッション ID: Cookie の値として設定する
    session_id: &str,
    // セキュア設定: true の場合 Secure 属性を付与する
    secure: bool,
) -> String {
    // Cookie の Secure 属性を決定する
    let secure_attr = if secure { "; Secure" } else { "" };
    // httpOnly SameSite=Strict Cookie を生成する
    format!(
        "k1s0_session={}; HttpOnly{}; SameSite=Strict; Path=/",
        session_id, secure_attr
    )
}

// CSRF トークンを生成するヘルパー関数
fn generate_csrf_token() -> String {
    // UUID v4 を使用して CSRF トークンを生成する
    Uuid::new_v4().to_string()
}

// ヘルスチェックエンドポイントのハンドラ
async fn health_handler() -> impl IntoResponse {
    // ヘルスチェック応答を返す
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

// 認証開始エンドポイント: PKCE フローを開始して IdP にリダイレクトする
async fn authorize_handler(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // クエリパラメータを Query で受け取る
    Query(params): Query<AuthorizeParams>,
) -> impl IntoResponse {
    // PKCE コードベリファイアを生成する (43〜128 文字のランダム文字列)
    let code_verifier = Uuid::new_v4().to_string().replace("-", "");
    // CSRF 防止用の state パラメータを生成する
    let state_param = params.state.unwrap_or_else(generate_csrf_token);
    // PKCE コードチャレンジを SHA256 でハッシュ化する (簡略実装)
    let code_challenge = format!("{}_challenge", code_verifier);
    // IdP の認可エンドポイント URL を組み立てる
    let auth_url = format!(
        "{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid+profile+email&state={}&code_challenge={}&code_challenge_method=S256",
        state.config.oidc_issuer,
        state.config.client_id,
        state.config.redirect_uri,
        state_param,
        code_challenge,
    );
    // 認証開始ログを出力する
    info!("PKCE 認証フロー開始: auth_url={}", auth_url);
    // IdP にリダイレクトするレスポンスを返す
    let mut headers = HeaderMap::new();
    // Location ヘッダーを設定してリダイレクトする
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&auth_url).unwrap_or(HeaderValue::from_static("/")),
    );
    // 302 Found でリダイレクトする
    (StatusCode::FOUND, headers)
}

// PKCE コールバックエンドポイント: 認可コードを受け取ってトークンを取得する
async fn callback_handler(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // クエリパラメータを Query で受け取る
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    // 認可コードのログを出力する
    info!("PKCE コールバック受信: code={}...", &params.code[..8.min(params.code.len())]);
    // トークンエンドポイントに認可コードを送信してトークンを取得する
    let token_result = state.http_client
        .post(format!("{}/token", state.config.oidc_issuer))
        .form(&[
            // 付与タイプ: authorization_code を指定する
            ("grant_type", "authorization_code"),
            // 認可コードを送信する
            ("code", &params.code),
            // リダイレクト URI を送信する
            ("redirect_uri", &state.config.redirect_uri),
            // クライアント ID を送信する
            ("client_id", &state.config.client_id),
            // クライアントシークレットを送信する
            ("client_secret", &state.config.client_secret),
        ])
        .send()
        .await;
    // トークン取得結果を処理する
    let token_resp = match token_result {
        // 成功の場合はレスポンスを取得する
        Ok(resp) => resp,
        // 失敗の場合はエラーレスポンスを返す
        Err(e) => {
            // トークン取得失敗ログを出力する
            error!("トークン取得失敗: {}", e);
            // 500 Internal Server Error を返す
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new()).into_response();
        }
    };
    // トークンレスポンスを JSON でデシリアライズする
    let token_data: TokenResponse = match token_resp.json().await {
        // 成功の場合はトークンデータを取得する
        Ok(data) => data,
        // 失敗の場合はエラーレスポンスを返す
        Err(e) => {
            // JSON パース失敗ログを出力する
            error!("トークンレスポンス JSON パース失敗: {}", e);
            // 500 Internal Server Error を返す
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new()).into_response();
        }
    };
    // セッション ID を生成する (UUID v4)
    let session_id = Uuid::new_v4().to_string();
    // CSRF トークンを生成する
    let csrf_token = generate_csrf_token();
    // セッション有効期限を計算する (現在時刻 + expires_in)
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + token_data.expires_in;
    // セッションを作成してセッションストアに保存する
    let session = Session {
        // アクセストークンを設定する
        access_token: token_data.access_token.clone(),
        // リフレッシュトークンを設定する
        refresh_token: token_data.refresh_token,
        // CSRF トークンを設定する
        csrf_token: csrf_token.clone(),
        // セッション有効期限を設定する
        expires_at,
    };
    // セッションストアにセッションを書き込む
    state.sessions.write().await.insert(session_id.clone(), session);
    // セッション作成ログを出力する
    info!("セッション作成完了: session_id={}...", &session_id[..8]);
    // レスポンスヘッダーを組み立てる
    let mut headers = HeaderMap::new();
    // httpOnly セキュア Cookie を設定する
    let cookie = build_session_cookie(&session_id, state.config.cookie_secure);
    // Set-Cookie ヘッダーを設定する
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap_or(HeaderValue::from_static("")),
    );
    // ルートにリダイレクトする
    headers.insert(header::LOCATION, HeaderValue::from_static("/"));
    // 302 Found でリダイレクトする
    (StatusCode::FOUND, headers).into_response()
}

// Silent renew エンドポイント: リフレッシュトークンでアクセストークンを更新する
async fn silent_renew_handler(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // リクエストヘッダーを受け取る
    headers: HeaderMap,
) -> impl IntoResponse {
    // Cookie ヘッダーからセッション ID を取得する
    let session_id = match headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            // Cookie 文字列から k1s0_session を取得する
            s.split(';').find_map(|part| {
                // k1s0_session= で始まる Cookie 値を取得する
                part.trim().strip_prefix("k1s0_session=")
            })
        })
    {
        // セッション ID が存在する場合は取得する
        Some(id) => id.to_string(),
        // セッション ID が存在しない場合は 401 Unauthorized を返す
        None => {
            // セッション ID なしのリクエストをログに記録する
            warn!("セッション ID なしの silent renew リクエスト");
            // 401 Unauthorized を返す
            return (StatusCode::UNAUTHORIZED, "セッションが見つかりません").into_response();
        }
    };
    // セッションストアからセッションを取得する
    let sessions = state.sessions.read().await;
    // セッションを取得する
    let session = match sessions.get(&session_id) {
        // セッションが存在する場合は取得する
        Some(s) => s.clone(),
        // セッションが存在しない場合は 401 Unauthorized を返す
        None => {
            // セッションなしのリクエストをログに記録する
            warn!("無効なセッション ID: session_id={}...", &session_id[..8.min(session_id.len())]);
            // 401 Unauthorized を返す
            return (StatusCode::UNAUTHORIZED, "無効なセッション").into_response();
        }
    };
    // ロックを解放する
    drop(sessions);
    // リフレッシュトークンが存在するかどうかを確認する
    let refresh_token = match &session.refresh_token {
        // リフレッシュトークンが存在する場合は取得する
        Some(rt) => rt.clone(),
        // リフレッシュトークンが存在しない場合は 401 を返す
        None => {
            // リフレッシュトークンなしのログを記録する
            warn!("リフレッシュトークンなし: session_id={}...", &session_id[..8.min(session_id.len())]);
            return (StatusCode::UNAUTHORIZED, "リフレッシュトークンがありません").into_response();
        }
    };
    // リフレッシュトークンでアクセストークンを更新する
    let renew_result = state.http_client
        .post(format!("{}/token", state.config.oidc_issuer))
        .form(&[
            // 付与タイプ: refresh_token を指定する
            ("grant_type", "refresh_token"),
            // リフレッシュトークンを送信する
            ("refresh_token", &refresh_token),
            // クライアント ID を送信する
            ("client_id", &state.config.client_id),
            // クライアントシークレットを送信する
            ("client_secret", &state.config.client_secret),
        ])
        .send()
        .await;
    // トークン更新結果を処理する
    match renew_result {
        // 成功の場合はトークンを更新する
        Ok(_) => {
            // silent renew 成功ログを出力する
            info!("Silent renew 成功: session_id={}...", &session_id[..8.min(session_id.len())]);
            // 200 OK を返す
            (StatusCode::OK, Json(serde_json::json!({"renewed": true}))).into_response()
        }
        // 失敗の場合はエラーレスポンスを返す
        Err(e) => {
            // silent renew 失敗ログを出力する
            error!("Silent renew 失敗: {}", e);
            // 500 Internal Server Error を返す
            (StatusCode::INTERNAL_SERVER_ERROR, "トークン更新失敗").into_response()
        }
    }
}

// CSRF トークン検証ミドルウェア: リクエストの X-CSRF-Token ヘッダーを検証する
async fn csrf_check(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // リクエストヘッダーを受け取る
    headers: HeaderMap,
    // リクエストを受け取る
    request: axum::extract::Request,
    // 次のミドルウェアを受け取る
    next: axum::middleware::Next,
) -> Response {
    // GET/HEAD/OPTIONS リクエストは CSRF チェックを免除する
    let method = request.method().clone();
    if method == http::Method::GET
        || method == http::Method::HEAD
        || method == http::Method::OPTIONS
    {
        // CSRF チェック免除: GET/HEAD/OPTIONS は安全なメソッドとして扱う
        return next.run(request).await;
    }
    // X-CSRF-Token ヘッダーを取得する
    let csrf_token = match headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
    {
        // CSRF トークンが存在する場合は取得する
        Some(token) => token.to_string(),
        // CSRF トークンが存在しない場合は 403 Forbidden を返す
        None => {
            // CSRF トークンなしのリクエストをログに記録する
            warn!("CSRF トークンが見つからない");
            // 403 Forbidden を返す
            return (StatusCode::FORBIDDEN, "CSRF トークンが必要です").into_response();
        }
    };
    // Cookie からセッション ID を取得して CSRF トークンを検証する (簡略実装)
    let session_id_opt = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|part| part.trim().strip_prefix("k1s0_session=")).map(str::to_string));
    // セッション ID が存在する場合は CSRF トークンを検証する
    if let Some(session_id) = session_id_opt {
        // セッションストアからセッションを取得する
        let sessions = state.sessions.read().await;
        // セッションを検索する
        if let Some(session) = sessions.get(&session_id) {
            // CSRF トークンがセッションのものと一致するかどうかを確認する
            if session.csrf_token != csrf_token {
                // CSRF トークン不一致ログを記録する
                warn!("CSRF トークン不一致: session_id={}...", &session_id[..8.min(session_id.len())]);
                // ロックを解放する
                drop(sessions);
                // 403 Forbidden を返す
                return (StatusCode::FORBIDDEN, "無効な CSRF トークン").into_response();
            }
        }
        // ロックを解放する
        drop(sessions);
    }
    // CSRF チェック通過: 次のミドルウェアに処理を委譲する
    next.run(request).await
}

// バックチャネルログアウトエンドポイント: IdP からのログアウト通知を処理する
async fn backchannel_logout_handler(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // リクエストボディを JSON でパースする
    Json(body): Json<BackchannelLogoutRequest>,
) -> impl IntoResponse {
    // バックチャネルログアウト受信ログを出力する
    info!("バックチャネルログアウト受信");
    // logout_token を JWT として検証する (発行者・署名を確認する)
    let validation = Validation::default();
    // JWT デコードキーを設定する (実際の実装では IdP の公開鍵を使用する)
    let decoding_key = DecodingKey::from_secret(b"placeholder_key");
    // logout_token をデコードして sid を取得する
    match decode::<Claims>(&body.logout_token, &decoding_key, &validation) {
        // デコード成功の場合はセッションを無効化する
        Ok(token_data) => {
            // sid (セッション ID) を取得する
            let sid = token_data.claims.sid.unwrap_or_default();
            // セッションストアから該当セッションを削除する
            let mut sessions = state.sessions.write().await;
            // sid に一致するセッションを全て削除する
            sessions.retain(|_, session| {
                // セッションの CSRF トークンを SID として扱う (簡略実装)
                !session.csrf_token.starts_with(&sid)
            });
            // バックチャネルログアウト処理完了ログを出力する
            info!("バックチャネルログアウト処理完了: sub={}", token_data.claims.sub);
            // 200 OK を返す
            (StatusCode::OK, Json(serde_json::json!({"logged_out": true}))).into_response()
        }
        // デコード失敗の場合は 400 Bad Request を返す
        Err(e) => {
            // logout_token 検証失敗ログを記録する
            warn!("logout_token 検証失敗: {}", e);
            // 400 Bad Request を返す
            (StatusCode::BAD_REQUEST, "無効な logout_token").into_response()
        }
    }
}

// API プロキシエンドポイント: Authorization ヘッダーを中継してバックエンドに転送する
async fn api_proxy_handler(
    // アプリケーション状態を State で受け取る
    State(state): State<AppState>,
    // リクエストヘッダーを受け取る
    headers: HeaderMap,
) -> impl IntoResponse {
    // Cookie からセッション ID を取得する
    let session_id = match headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|part| part.trim().strip_prefix("k1s0_session=")).map(str::to_string))
    {
        // セッション ID が存在する場合は取得する
        Some(id) => id,
        // セッション ID が存在しない場合は 401 Unauthorized を返す
        None => {
            return (StatusCode::UNAUTHORIZED, "認証が必要です").into_response();
        }
    };
    // セッションストアからセッションを取得する
    let sessions = state.sessions.read().await;
    // セッションからアクセストークンを取得する
    let access_token = match sessions.get(&session_id) {
        // セッションが存在する場合はアクセストークンを取得する
        Some(session) => session.access_token.clone(),
        // セッションが存在しない場合は 401 Unauthorized を返す
        None => {
            // ロックを解放する
            drop(sessions);
            return (StatusCode::UNAUTHORIZED, "無効なセッション").into_response();
        }
    };
    // ロックを解放する
    drop(sessions);
    // Authorization ヘッダーを Bearer トークンとして設定する
    let mut response_headers = HeaderMap::new();
    // Authorization ヘッダーを組み立てる
    let auth_value = format!("Bearer {}", access_token);
    // Authorization ヘッダーを設定する
    response_headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&auth_value).unwrap_or(HeaderValue::from_static("")),
    );
    // API プロキシ成功ログを出力する
    info!("API プロキシ: Authorization ヘッダーを中継する");
    // 200 OK でアクセストークンを中継したレスポンスを返す
    (StatusCode::OK, response_headers, Json(serde_json::json!({"proxied": true}))).into_response()
}

// アプリケーションルーターを構築する
fn build_router(state: AppState) -> Router {
    // CORS ミドルウェアを設定する
    let cors = tower_http::cors::CorsLayer::new()
        // 許可するオリジンを設定する (本番環境では許可オリジンを制限する)
        .allow_origin(tower_http::cors::Any)
        // 許可するメソッドを設定する
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        // 許可するヘッダーを設定する
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // ルーターを構築してエンドポイントを登録する
    Router::new()
        // ヘルスチェックエンドポイント: GET /health
        .route("/health", get(health_handler))
        // PKCE 認証開始エンドポイント: GET /auth/authorize
        .route("/auth/authorize", get(authorize_handler))
        // PKCE コールバックエンドポイント: GET /auth/callback
        .route("/auth/callback", get(callback_handler))
        // Silent renew エンドポイント: POST /auth/silent-renew
        .route("/auth/silent-renew", post(silent_renew_handler))
        // バックチャネルログアウトエンドポイント: POST /auth/backchannel-logout
        .route("/auth/backchannel-logout", post(backchannel_logout_handler))
        // API プロキシエンドポイント: GET /api/*
        .route("/api/proxy", get(api_proxy_handler))
        // CSRF ミドルウェアをレイヤーとして追加する
        .layer(axum::middleware::from_fn_with_state(state.clone(), csrf_check))
        // CORS ミドルウェアをレイヤーとして追加する
        .layer(cors)
        // tracing ミドルウェアをレイヤーとして追加する
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // アプリケーション状態を設定する
        .with_state(state)
}

// メイン関数: BFF サーバーを起動する
#[tokio::main]
async fn main() -> Result<()> {
    // tracing サブスクライバーを初期化する (環境変数 RUST_LOG でログレベルを制御する)
    tracing_subscriber::fmt()
        // 環境変数フィルターを設定する
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("k1s0_bff=info".parse()?)
        )
        // tracing サブスクライバーを初期化する
        .init();

    // アプリケーション設定を環境変数から読み込む
    let config = AppConfig {
        // OIDC 発行者 URL を環境変数から取得する
        oidc_issuer: std::env::var("OIDC_ISSUER")
            .unwrap_or_else(|_| "http://keycloak.k1s0-system.svc.cluster.local:8080/realms/k1s0".to_string()),
        // クライアント ID を環境変数から取得する
        client_id: std::env::var("OIDC_CLIENT_ID")
            .unwrap_or_else(|_| "k1s0-bff".to_string()),
        // クライアントシークレットを環境変数から取得する
        client_secret: std::env::var("OIDC_CLIENT_SECRET")
            .unwrap_or_else(|_| "placeholder_secret".to_string()),
        // リダイレクト URI を環境変数から取得する
        redirect_uri: std::env::var("OIDC_REDIRECT_URI")
            .unwrap_or_else(|_| "https://localhost:8443/auth/callback".to_string()),
        // Cookie セキュア設定を環境変数から取得する
        cookie_secure: std::env::var("COOKIE_SECURE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        // CORS 許可オリジンを環境変数から取得する
        allowed_origins: std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "https://localhost:3000".to_string())
            .split(',')
            .map(str::to_string)
            .collect(),
    };

    // アプリケーション共有状態を初期化する
    let state = AppState {
        // 設定を Arc でラップする
        config: Arc::new(config),
        // セッションストアを初期化する
        sessions: Arc::new(RwLock::new(HashMap::new())),
        // HTTP クライアントを構築する
        http_client: reqwest::Client::builder()
            // タイムアウトを 30 秒に設定する
            .timeout(std::time::Duration::from_secs(30))
            // TLS 設定を有効化する
            .use_rustls_tls()
            // HTTP クライアントを構築する
            .build()?,
    };

    // BFF サーバーのリスナーアドレスを設定する
    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    // TCP リスナーを作成する
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    // BFF サーバー起動ログを出力する
    info!("k1s0 BFF サーバー起動: addr={}", listen_addr);

    // axum サーバーを起動する
    axum::serve(listener, build_router(state)).await?;
    // 正常終了を返す
    Ok(())
}
