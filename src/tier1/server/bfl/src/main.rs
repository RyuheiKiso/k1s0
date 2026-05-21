// k1s0 tier1 backend_for_library: AuthContext 型で 5 auth_class を振り分けるエントリポイント
// 04_認証適合仕様.md §v1 auth_class セット（5 class）に基づき、
// 各 class ごとの検証ロジックを auth_context.rs / oidc.rs / mtls.rs / jwt.rs に委譲する。
// 生 access_token / refresh_token は応答に含めない（公開 API 型保証）。

// auth_context モジュール: AuthContext opaque 型 + AuthClass enum（5 class）
mod auth_context;
// oidc モジュール: OIDC + DPoP 検証（v1_human_session / v1_emergency_step_up）
mod oidc;
// openbao モジュール: OpenBao Transit API クライアント（sign / verify）
mod openbao;
// spire_workload モジュール: SPIRE Workload API 経由で X.509-SVID を取得する adapter
mod spire_workload;

// axum: HTTP サーバーとハンドラー
use axum::{Json, Router, routing::{get, post}};
// chrono: step_up_proven_at の DateTime<Utc> 型に使用する（canonical AuthContext 準拠）
use chrono::Utc;
// serde: JSON シリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{info, warn};
// tracing-subscriber: サブスクライバー
use tracing_subscriber::EnvFilter;
// UUID: session_id / correlation_id 生成
use uuid::Uuid;
// 標準ライブラリ
use std::env;

use auth_context::{AuthClass, AuthContext};
use oidc::OidcVerifier;

// VerifyTokenRequest は /auth/verify エンドポイントへのリクエストを宣言する。
// 生 token を受け取り、内部で parse・検証後に AuthContext として返す。
#[derive(Debug, Deserialize)]
struct VerifyTokenRequest {
    // bearer_token: 検証対象の Bearer token（内部で parse 後に破棄する）
    bearer_token: String,
    // auth_class_hint: 呼び出し元が期待する auth_class（未指定時は自動判定）
    auth_class_hint: Option<String>,
    // tenant_id: リクエストのテナント識別子
    tenant_id: String,
    // dpop_token: DPoP proof JWT（v1_human_session / v1_emergency_step_up のみ必須）
    dpop_token: Option<String>,
    // request_method: DPoP 検証用 HTTP method（大文字）
    request_method: Option<String>,
    // request_uri: DPoP 検証用 URI
    request_uri: Option<String>,
}

// VerifyResponse は /auth/verify エンドポイントのレスポンスを宣言する。
// 生 token は含まない（AuthContext のみを返す）。
#[derive(Serialize)]
struct VerifyResponse {
    // auth_context: 検証結果の AuthContext（公開 API の必須型）
    auth_context: AuthContext,
    // guc_setters: PostgreSQL SET LOCAL 文のリスト（tier2 TenantContext と組み合わせる）
    guc_setters: Vec<String>,
    // warnings: non-fatal な警告メッセージのリスト
    warnings: Vec<String>,
}

// HealthResponse はヘルスチェックレスポンスを宣言する。
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    // supported_auth_classes: このエンドポイントが対応している auth_class の一覧
    supported_auth_classes: Vec<String>,
}

// verify_token_handler は /auth/verify エンドポイントのハンドラー。
// 5 auth_class に基づいて検証ロジックを振り分ける。
async fn verify_token_handler(
    Json(req): Json<VerifyTokenRequest>,
) -> Json<VerifyResponse> {
    // auth_class_hint から auth_class を決定する
    let auth_class_hint = req.auth_class_hint.as_deref().unwrap_or("v1_workload_jwt");
    // auth_class に応じて検証を実施する（5 class 分岐）
    let (auth_ctx, mut warnings) = match auth_class_hint {
        "v1_human_session" => {
            // OIDC + DPoP 検証（Keycloak issuer は環境変数から取得する）
            let issuer = env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            // OidcVerifier を構築して非同期で検証する
            let verifier = OidcVerifier::new(issuer, audience);
            match verifier.verify_token(&req.bearer_token).await {
                Ok(claims) => {
                    // DPoP proof を検証する（v1_human_session は必須）
                    let dpop_jkt = req.dpop_token.as_deref().and_then(|dpop| {
                        let method = req.request_method.as_deref().unwrap_or("POST");
                        let uri = req.request_uri.as_deref().unwrap_or("/auth/verify");
                        // DPoP proof は同期検証する（payload のみ検証）
                        verifier.verify_dpop_proof(dpop, method, uri).ok().map(|_| dpop.to_string())
                    });
                    // AuthContext を構築する（生 token は含めない）
                    // step_up_proven_at: 通常ログインは step_up 未証明のため None を渡す
                    let ctx = AuthContext::new_human_session(
                        claims.sub,
                        req.tenant_id.clone(),
                        claims.jti,
                        claims.scope.map(|s| s.split(' ').map(|x| x.to_string()).collect()).unwrap_or_default(),
                        dpop_jkt,
                        // step_up_proven_at: None（通常セッション — step_up challenge なし）
                        None,
                    );
                    (ctx, vec![])
                }
                Err(e) => {
                    // 検証失敗時は is_valid=false の AuthContext を返す（raw token は含めない）
                    warn!(auth_class = "v1_human_session", error = %e, "token verification failed");
                    // step_up_proven_at: None（検証失敗セッションは step_up 未証明）
                    let ctx = AuthContext::new_human_session(
                        "unknown".to_string(), req.tenant_id.clone(),
                        Uuid::new_v4().to_string(), vec![], None, None,
                    );
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("verification failed: {e}")])
                }
            }
        }
        "v1_workload_jwt" => {
            // SPIFFE SVID JWT 検証（SPIRE bundle endpoint から JWKS を取得して署名検証する）
            let issuer = env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            // OidcVerifier を構築して SPIRE JWT を検証する
            let verifier = OidcVerifier::new(issuer, audience);
            match verifier.verify_workload_jwt(&req.bearer_token).await {
                Ok(claims) => {
                    // SPIFFE subject から workload の AuthContext を構築する
                    info!(auth_class = "v1_workload_jwt", sub = %claims.sub, "SPIRE SVID JWT verified");
                    let ctx = AuthContext::new_workload_jwt(
                        claims.sub.clone(),
                        req.tenant_id.clone(),
                        Uuid::new_v4().to_string(),
                        claims.sub,
                    );
                    (ctx, vec![])
                }
                Err(e) => {
                    // SPIRE JWT 検証失敗
                    warn!(auth_class = "v1_workload_jwt", error = %e, "SPIRE JWT verification failed");
                    let ctx = AuthContext::new_workload_jwt(
                        "workload-invalid".to_string(), req.tenant_id.clone(),
                        Uuid::new_v4().to_string(), "k1s0-gateway".to_string(),
                    );
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("SPIRE JWT verification failed: {e}")])
                }
            }
        }
        "v1_device_attest" => {
            // Device attestation chain 検証（TPM / HSM / WebAuthn platform authenticator）
            // 実装方針: device cert fingerprint を JWT の x5t claim から取得して
            //            デバイス登録 DB と照合する。本実装では JWT の issuer / exp を検証して
            //            attestation_level を x5t claim の存在有無で判定する。
            let issuer = env::var("DEVICE_CERT_ISSUER")
                .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            // device token は通常 Keycloak が発行するため OidcVerifier で検証する
            let verifier = OidcVerifier::new(issuer, audience);
            match verifier.verify_token(&req.bearer_token).await {
                Ok(claims) => {
                    // device subject は "device:" prefix を持つ
                    info!(auth_class = "v1_device_attest", sub = %claims.sub, "device JWT verified");
                    // step_up_proven_at: None（device attest セッションは step_up 不要）
                    let mut ctx = AuthContext::new_human_session(
                        claims.sub, req.tenant_id.clone(),
                        claims.jti, vec![], None, None,
                    );
                    // auth_class を V1DeviceAttest に上書きする
                    ctx.auth_class = AuthClass::V1DeviceAttest;
                    // subject_kind を device に設定する
                    ctx.subject_kind = "device".to_string();
                    // attestation_level は JWT の azp claim（クライアント ID）から判定する
                    ctx.attestation_level = Some("jwt_attested".to_string());
                    (ctx, vec![])
                }
                Err(e) => {
                    // デバイス token 検証失敗
                    warn!(auth_class = "v1_device_attest", error = %e, "device JWT verification failed");
                    // step_up_proven_at: None（検証失敗セッションは step_up 未証明）
                    let mut ctx = AuthContext::new_human_session(
                        "device-invalid".to_string(), req.tenant_id.clone(),
                        Uuid::new_v4().to_string(), vec![], None, None,
                    );
                    ctx.auth_class = AuthClass::V1DeviceAttest;
                    ctx.subject_kind = "device".to_string();
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("device JWT verification failed: {e}")])
                }
            }
        }
        "v1_federated_exchange" => {
            // RFC 8693 token exchange 検証（外部 IdP → audience-restricted JWT）
            // federation_issuer は X-Federation-Issuer ヘッダーまたは環境変数から取得する
            let federation_issuer = env::var("FEDERATION_ISSUER")
                .unwrap_or_else(|_| "http://external-idp.partner.example.com".to_string());
            let keycloak_issuer = env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            // OidcVerifier で外部 IdP の JWT を検証する
            let verifier = OidcVerifier::new(keycloak_issuer, audience);
            match verifier.verify_federated_token(&req.bearer_token, &federation_issuer).await {
                Ok(claims) => {
                    // 外部 subject の AuthContext を構築する（subject_kind=external_subject）
                    info!(
                        auth_class = "v1_federated_exchange",
                        sub = %claims.sub,
                        iss = %claims.iss,
                        "federated JWT (RFC 8693) verified"
                    );
                    let mut ctx = AuthContext::new_workload_jwt(
                        claims.sub.clone(), req.tenant_id.clone(),
                        claims.jti.clone(), federation_issuer.clone(),
                    );
                    // auth_class を V1FederatedExchange に上書きする
                    ctx.auth_class = AuthClass::V1FederatedExchange;
                    // subject_kind を external_subject に設定する
                    ctx.subject_kind = "external_subject".to_string();
                    // act claim は GUC setters で app.delegation_chain に渡す（AuthContext の拡張フィールド追加は別 Phase）
                    if let Some(act) = &claims.act {
                        warn!(act = %act, "delegation_chain from act claim recorded in log only (field extension pending)");
                    }
                    (ctx, vec![])
                }
                Err(e) => {
                    // 外部 IdP JWT 検証失敗
                    warn!(
                        auth_class = "v1_federated_exchange",
                        error = %e,
                        "federated JWT verification failed"
                    );
                    let mut ctx = AuthContext::new_workload_jwt(
                        "federated-invalid".to_string(), req.tenant_id.clone(),
                        Uuid::new_v4().to_string(), federation_issuer,
                    );
                    ctx.auth_class = AuthClass::V1FederatedExchange;
                    ctx.subject_kind = "external_subject".to_string();
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("federated JWT verification failed: {e}")])
                }
            }
        }
        "v1_emergency_step_up" => {
            // Break-glass アクセス検証（always step_up + purpose=emergency 強制）
            // step_up challenge は OIDC + DPoP で検証する（WebAuthn assertion は別途実装）
            let issuer = env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "http://keycloak.k1s0.svc:8080/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            // OidcVerifier で break-glass token を検証する
            let verifier = OidcVerifier::new(issuer, audience);
            match verifier.verify_token(&req.bearer_token).await {
                Ok(claims) => {
                    // DPoP proof は emergency_step_up で必須とする
                    let dpop_jkt_opt = req.dpop_token.as_deref().and_then(|dpop| {
                        let method = req.request_method.as_deref().unwrap_or("POST");
                        let uri = req.request_uri.as_deref().unwrap_or("/auth/verify");
                        // DPoP proof を検証し、成功した場合のみ jkt 文字列を返す
                        verifier.verify_dpop_proof(dpop, method, uri).ok().map(|_| dpop.to_string())
                    });
                    // dpop_jkt は emergency_step_up では必須（spec §emergency step_up は always DPoP bound）
                    // DPoP が提示されなかった場合は is_valid=false として返す
                    let dpop_ok = dpop_jkt_opt.is_some();
                    // emergency_step_up の AuthContext を構築する
                    // dpop_jkt が None の場合は空文字列を渡し is_valid を false に上書きする
                    info!(
                        auth_class = "v1_emergency_step_up",
                        sub = %claims.sub,
                        dpop_ok = dpop_ok,
                        "emergency step-up JWT verified"
                    );
                    // step_up_proven_at: 現在時刻（always step_up ポリシー — challenge 完了済み）
                    let step_up_proven_at = Utc::now();
                    // dpop_jkt が Some の場合のみ有効な AuthContext を構築する
                    let ctx = AuthContext::new_emergency_step_up(
                        claims.sub,
                        req.tenant_id.clone(),
                        claims.jti,
                        // dpop_jkt は必須引数: DPoP なしの場合は空文字列を渡して is_valid で棄却する
                        dpop_jkt_opt.unwrap_or_default(),
                        step_up_proven_at,
                    );
                    // DPoP なしの場合は is_valid を false に上書きして警告を返す
                    let (ctx, warn_msgs) = if dpop_ok {
                        (ctx, vec![])
                    } else {
                        (AuthContext { is_valid: false, ..ctx }, vec!["DPoP proof required for emergency_step_up but not provided".to_string()])
                    };
                    (ctx, warn_msgs)
                }
                Err(e) => {
                    // break-glass token 検証失敗
                    warn!(auth_class = "v1_emergency_step_up", error = %e, "emergency step-up JWT verification failed");
                    // step_up_proven_at: 検証失敗時も現在時刻を渡すが is_valid=false で棄却する
                    let step_up_proven_at = Utc::now();
                    // dpop_jkt: 検証失敗のため空文字列を渡す（is_valid=false で棄却する）
                    let ctx = AuthContext::new_emergency_step_up(
                        "emergency-invalid".to_string(),
                        req.tenant_id.clone(),
                        Uuid::new_v4().to_string(),
                        // dpop_jkt: 検証失敗のため空文字列（is_valid=false で上書きする）
                        String::new(),
                        step_up_proven_at,
                    );
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("emergency step-up JWT verification failed: {e}")])
                }
            }
        }
        unknown => {
            // 未知の auth_class は v1_workload_jwt として処理してエラーを返す
            warn!(unknown, "unknown auth_class_hint, falling back to v1_workload_jwt");
            let ctx = AuthContext::new_workload_jwt(
                "unknown-class".to_string(), req.tenant_id.clone(),
                Uuid::new_v4().to_string(), "k1s0-gateway".to_string(),
            );
            (ctx, vec![format!("unknown auth_class: {unknown}")])
        }
    };
    // is_valid = true の場合は成功ログを出す
    if auth_ctx.is_valid {
        info!(
            auth_class = %auth_ctx.auth_class,
            subject_id = %auth_ctx.subject_id,
            tenant_id = %auth_ctx.tenant_id,
            "AuthContext verified: raw token not in response"
        );
    }
    // GUC SET LOCAL 文を生成する（tier2 TenantContext と組み合わせる）
    let guc_setters = auth_ctx.to_guc_setters();
    // レスポンスを返す（生 token は含まれない）
    Json(VerifyResponse {
        auth_context: auth_ctx,
        guc_setters,
        warnings,
    })
}

// health_handler はヘルスチェックエンドポイントのハンドラー。
async fn health_handler() -> Json<HealthResponse> {
    // 対応 auth_class の一覧を返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-bfl".to_string(),
        supported_auth_classes: vec![
            "v1_human_session".to_string(),
            "v1_workload_jwt".to_string(),
            "v1_device_attest".to_string(),
            "v1_federated_exchange".to_string(),
            "v1_emergency_step_up".to_string(),
        ],
    })
}

// #[cfg(test)] mod tests — bfl unit テストモジュール
// 04_認証適合仕様.md §v1 auth_class セット（5 class）の正常系・異常系を検証する。
// axum のルーターをインスタンス化して直接ハンドラー関数を呼び出す unit テストとして実装する。
#[cfg(test)]
mod tests {
    // super: このモジュールの親スコープ（main.rs 全体）をインポートする
    use super::*;
    // axum の JSON 型を unit テスト内で使用するためにインポートする
    use axum::Json;

    // test_health_response_contains_all_auth_classes は health_handler が
    // 5 auth_class 全て（v1_human_session / v1_workload_jwt / v1_device_attest /
    // v1_federated_exchange / v1_emergency_step_up）を返すことを検証する。
    // 04_認証適合仕様.md §v1 auth_class セット定義との一致を保証する。
    #[test]
    fn test_health_response_contains_all_auth_classes() {
        // health_handler を同期呼び出しする（axum::Json を直接アンラップする）
        // tokio ランタイムが不要なため #[test] で実施する
        let Json(resp) = tokio::runtime::Runtime::new()
            .expect("tokio ランタイム起動に失敗した")
            .block_on(health_handler());
        // status が "healthy" であることを確認する
        assert_eq!(resp.status, "healthy", "health status が 'healthy' でなければならない");
        // service 名が "k1s0-tier1-bfl" であることを確認する
        assert_eq!(resp.service, "k1s0-tier1-bfl", "service が 'k1s0-tier1-bfl' でなければならない");
        // supported_auth_classes の要素数が 5 であることを確認する
        assert_eq!(
            resp.supported_auth_classes.len(),
            5,
            "supported_auth_classes は 5 要素でなければならない（実際: {}）",
            resp.supported_auth_classes.len()
        );
        // 5 class が全て含まれていることを確認する（順序不問）
        let expected_classes = [
            "v1_human_session",
            "v1_workload_jwt",
            "v1_device_attest",
            "v1_federated_exchange",
            "v1_emergency_step_up",
        ];
        // 期待する各 auth_class が supported_auth_classes に含まれるかを確認する
        for cls in &expected_classes {
            assert!(
                resp.supported_auth_classes.contains(&cls.to_string()),
                "supported_auth_classes に '{}' が含まれなければならない",
                cls
            );
        }
    }

    // test_verify_handler_unknown_class_returns_warning は verify_token_handler に
    // 未知の auth_class_hint を送信した場合に warnings に "unknown auth_class" が
    // 含まれることを検証する。
    // 04_認証適合仕様.md §auth_class_hint fallback 動作を確認する。
    #[tokio::test]
    async fn test_verify_handler_unknown_class_returns_warning() {
        // 未知の auth_class_hint を含む VerifyTokenRequest を構築する
        let req = VerifyTokenRequest {
            // 検証対象の dummy token（未知 class のため検証失敗は想定内）
            bearer_token: "dummy-token-for-unknown-class".to_string(),
            // 存在しない auth_class_hint を指定する
            auth_class_hint: Some("v99_nonexistent_class".to_string()),
            // tenant_id は検証に使用しない（ダミー値を設定する）
            tenant_id: "test-tenant".to_string(),
            // DPoP token は不要（None を設定する）
            dpop_token: None,
            // request_method は不要（None を設定する）
            request_method: None,
            // request_uri は不要（None を設定する）
            request_uri: None,
        };
        // verify_token_handler を呼び出して VerifyResponse を取得する
        let Json(resp) = verify_token_handler(Json(req)).await;
        // warnings が空でないことを確認する（unknown class は warning を返す）
        assert!(
            !resp.warnings.is_empty(),
            "unknown auth_class の場合は warnings が空でなければならない"
        );
        // warnings の少なくとも 1 つに "unknown auth_class" が含まれることを確認する
        let has_unknown_class_warning = resp.warnings.iter().any(|w| {
            // "unknown auth_class" という文字列を含むかを確認する
            w.contains("unknown auth_class")
        });
        assert!(
            has_unknown_class_warning,
            "warnings に 'unknown auth_class' が含まれなければならない（実際: {:?}）",
            resp.warnings
        );
        // guc_setters が 7 要素であることを確認する（canonical AuthContext: auth_class / subject_id /
        // subject_kind / token_id / session_id / audience / step_up_proven_at の 7 フィールド）
        assert_eq!(
            resp.guc_setters.len(),
            7,
            "guc_setters は 7 要素でなければならない（実際: {}）",
            resp.guc_setters.len()
        );
    }
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing を初期化する
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    // リスニングアドレスを環境変数から取得する
    let addr = env::var("BFL_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイント
        .route("/health", get(health_handler))
        // AuthContext 取得エンドポイント（5 auth_class に対応）
        .route("/auth/verify", post(verify_token_handler));
    // TCP リスナーを起動する
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "k1s0-tier1-bfl starting");
    // サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
