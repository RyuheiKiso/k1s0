// dpop_failure_paths.rs — Phase C: DPoP proof 検証の failure-path テスト
// 04_認証適合仕様 §DPoP proof 検証（RFC 9449 準拠）の全 failure case を網羅する。
// DPoPProofVerifier::verify_dpop_proof の境界条件と異常系を検証する。

// テスト対象: bfl crate の DPoPProofVerifier
use k1s0_tier1_bfl::dpop::DPoPProofVerifier;
// テストヘルパー: compact JWT builder
use tier1_bfl_security_tests::jwt_helpers::{
    dpop_header, dpop_payload, expired_dpop_payload, make_dpop_jwt,
};
// serde_json: JSON 構築
use serde_json::json;

// ----------------------------------------
// Happy path
// ----------------------------------------

/// test_dpop_valid_proof — 有効な DPoP proof が Ok を返すことを確認する
#[tokio::test]
async fn test_dpop_valid_proof() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // 有効な DPoP proof JWT を構築する
    let jwt = make_dpop_jwt(
        dpop_header(),
        dpop_payload("POST", "https://api.k1s0.internal/token", "jti-valid-001"),
    );
    // verify_dpop_proof が Ok を返すことを確認する
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // 有効な proof は Ok で返さなければならない
    assert!(result.is_ok(), "valid DPoP proof must be Ok, got: {:?}", result.err());
}

// ----------------------------------------
// Failure path: format
// ----------------------------------------

/// test_dpop_invalid_format_two_parts — 2 部分しかない compact JWT を拒否する
#[tokio::test]
async fn test_dpop_invalid_format_two_parts() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // 不正な形式: header.payload のみ（signature 欠落）
    let jwt = "header.payload";
    // verify_dpop_proof がエラーを返すことを確認する
    let result = verifier
        .verify_dpop_proof(jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // 不正な形式は Err でなければならない
    assert!(result.is_err(), "2-part JWT must be rejected");
}

/// test_dpop_invalid_format_empty — 空文字列を拒否する
#[tokio::test]
async fn test_dpop_invalid_format_empty() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // 空文字列を渡す
    let result = verifier
        .verify_dpop_proof("", "POST", "https://api.k1s0.internal/token", None)
        .await;
    // 空文字列は Err でなければならない
    assert!(result.is_err(), "empty JWT must be rejected");
}

// ----------------------------------------
// Failure path: header typ
// ----------------------------------------

/// test_dpop_wrong_typ — typ が "dpop+jwt" でない場合を拒否する
#[tokio::test]
async fn test_dpop_wrong_typ() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // typ が "jwt"（DPoP でない）のヘッダーを作成する
    let header = json!({"typ": "JWT", "alg": "ES256"});
    let jwt = make_dpop_jwt(header, dpop_payload("POST", "https://api.k1s0.internal/token", "jti-typ-001"));
    // verify_dpop_proof がエラーを返すことを確認する
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // 誤った typ は Err でなければならない
    assert!(result.is_err(), "wrong typ must be rejected");
    // エラーメッセージに typ 情報が含まれることを確認する
    let err = result.unwrap_err().to_string();
    assert!(err.contains("typ"), "error must mention typ, got: {err}");
}

// ----------------------------------------
// Failure path: algorithm downgrade
// ----------------------------------------

/// test_dpop_alg_downgrade_to_hs256 — alg が HS256（対称鍵）の場合を拒否する
#[tokio::test]
async fn test_dpop_alg_downgrade_to_hs256() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // alg が HS256（対称鍵）のヘッダーを作成する（alg confusion attack の典型）
    let header = json!({"typ": "dpop+jwt", "alg": "HS256"});
    let jwt = make_dpop_jwt(header, dpop_payload("POST", "https://api.k1s0.internal/token", "jti-alg-001"));
    // verify_dpop_proof がエラーを返すことを確認する
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // HS256 は alg confusion attack のリスクがあるため拒否されなければならない
    assert!(result.is_err(), "HS256 alg must be rejected to prevent alg confusion");
}

/// test_dpop_alg_none_attack — alg が "none" の場合を拒否する
#[tokio::test]
async fn test_dpop_alg_none_attack() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // alg が "none" のヘッダーを作成する（署名なし攻撃の典型）
    let header = json!({"typ": "dpop+jwt", "alg": "none"});
    let jwt = make_dpop_jwt(header, dpop_payload("POST", "https://api.k1s0.internal/token", "jti-none-001"));
    // verify_dpop_proof がエラーを返すことを確認する
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // alg=none は署名なし攻撃の典型であり拒否されなければならない
    assert!(result.is_err(), "alg=none must be rejected to prevent signature bypass");
}

// ----------------------------------------
// Failure path: htm mismatch
// ----------------------------------------

/// test_dpop_htm_mismatch — HTTP method が一致しない場合を拒否する
#[tokio::test]
async fn test_dpop_htm_mismatch() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // htm が "GET" の proof に対して "POST" で検証を試みる
    let jwt = make_dpop_jwt(
        dpop_header(),
        json!({
            "jti": "jti-htm-001",
            "htu": "https://api.k1s0.internal/token",
            "htm": "GET",
            "iat": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        }),
    );
    // method が "POST" で検証する（proof は "GET" 用）
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // htm mismatch は CSRF 対策のために拒否されなければならない
    assert!(result.is_err(), "htm mismatch must be rejected");
}

// ----------------------------------------
// Failure path: htu mismatch
// ----------------------------------------

/// test_dpop_htu_mismatch — HTTP target URI が一致しない場合を拒否する
#[tokio::test]
async fn test_dpop_htu_mismatch() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // htu が異なる URI の proof を作成する
    let jwt = make_dpop_jwt(
        dpop_header(),
        dpop_payload("POST", "https://other.k1s0.internal/other", "jti-htu-001"),
    );
    // 正規の htu で検証を試みる（proof は別の htu 用）
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // htu mismatch は SSRF / token 再利用攻撃を防ぐために拒否されなければならない
    assert!(result.is_err(), "htu mismatch must be rejected to prevent token reuse");
}

// ----------------------------------------
// Failure path: expired iat
// ----------------------------------------

/// test_dpop_expired_iat — iat が 60 秒を超えた proof を拒否する
#[tokio::test]
async fn test_dpop_expired_iat() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // iat が 120 秒前の期限切れ proof を作成する
    let jwt = make_dpop_jwt(
        dpop_header(),
        expired_dpop_payload("POST", "https://api.k1s0.internal/token", "jti-exp-001"),
    );
    // 期限切れ proof の検証を試みる
    let result = verifier
        .verify_dpop_proof(&jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // 期限切れ proof は replay 攻撃を防ぐために拒否されなければならない（RFC 9449 §11.1）
    assert!(result.is_err(), "expired iat (>60s) must be rejected to prevent replay");
}

// ----------------------------------------
// Failure path: ath mismatch
// ----------------------------------------

/// test_dpop_ath_mismatch — ath claim が access_token と不一致の場合を拒否する
#[tokio::test]
async fn test_dpop_ath_mismatch() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // 誤った ath を持つ proof を作成する（別の access_token の hash）
    let wrong_ath = "aGVsbG8gd29ybGQ"; // "hello world" の base64url（不正な ath）
    let jwt = make_dpop_jwt(
        dpop_header(),
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            json!({
                "jti": "jti-ath-001",
                "htu": "https://api.k1s0.internal/resource",
                "htm": "GET",
                "iat": now,
                "ath": wrong_ath
            })
        },
    );
    // 正規の access_token で検証を試みる（ath は別の token に対するもの）
    let result = verifier
        .verify_dpop_proof(
            &jwt,
            "GET",
            "https://api.k1s0.internal/resource",
            Some("correct_access_token"),
        )
        .await;
    // ath mismatch は token 束縛の破壊を意味するため拒否されなければならない
    assert!(result.is_err(), "ath mismatch must be rejected to enforce token binding");
}

// ----------------------------------------
// Failure path: invalid base64 header
// ----------------------------------------

/// test_dpop_invalid_base64_header — base64 デコード不可のヘッダーを拒否する
#[tokio::test]
async fn test_dpop_invalid_base64_header() {
    // DPoP verifier を初期化する
    let verifier = DPoPProofVerifier::new();
    // 不正な base64 のヘッダーを含む compact JWT を作成する
    let jwt = "!!!invalid_base64!!!.validpayload.fakesig";
    // verify_dpop_proof がエラーを返すことを確認する
    let result = verifier
        .verify_dpop_proof(jwt, "POST", "https://api.k1s0.internal/token", None)
        .await;
    // base64 デコード失敗は Err でなければならない
    assert!(result.is_err(), "invalid base64 header must be rejected");
}
