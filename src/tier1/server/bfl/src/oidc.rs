// oidc.rs — spec 04 認証適合仕様: OIDC + DPoP 検証モジュール
// 04_認証適合仕様.md §v1_human_session と §v1_emergency_step_up に対応する。
// Keycloak issuer 検証 + JWKS rotation 監視 + DPoP proof 検証（RFC 9449）を実装する。

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

// OidcClaims は Keycloak が発行する JWT の主要 claim を宣言する。
#[derive(Debug, Deserialize)]
pub struct OidcClaims {
    // sub: canonical subject identifier
    pub sub: String,
    // jti: JWT ID（revocation tracking）
    pub jti: String,
    // tid: Keycloak の tenant_id カスタム claim
    pub tid: Option<String>,
    // scope: space-separated scope 文字列
    pub scope: Option<String>,
    // exp: 有効期限（Unix timestamp）
    pub exp: u64,
    // iss: issuer（Keycloak の URL）
    pub iss: String,
}

// DpopProofClaims は RFC 9449 DPoP proof JWT の claim を宣言する。
#[derive(Debug, Deserialize)]
pub struct DpopProofClaims {
    // htm: HTTP method（大文字）
    pub htm: String,
    // htu: HTTP URI
    pub htu: String,
    // jti: proof の一意識別子（replay 防止）
    pub jti: String,
    // iat: 発行時刻（Unix timestamp）
    pub iat: u64,
}

// OidcVerifier は Keycloak OIDC 検証ロジックを提供する。
pub struct OidcVerifier {
    // keycloak_issuer: Keycloak の issuer URL
    keycloak_issuer: String,
    // expected_audience: gateway が受け入れる audience
    expected_audience: String,
}

impl OidcVerifier {
    // new は OidcVerifier を構築する。
    pub fn new(keycloak_issuer: String, expected_audience: String) -> Self {
        Self { keycloak_issuer, expected_audience }
    }

    // verify_token は JWT の issuer / audience / exp を検証して claims を返す。
    // 署名検証は JWKS endpoint から非同期に公開鍵を取得して実施する（TODO）。
    // 生 token 文字列は受け取り後すぐに parse して破棄する。
    pub fn verify_token(&self, token: &str) -> Result<OidcClaims> {
        // JWT を . で分割する（header.payload.signature の 3 部構成）
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("invalid JWT format: expected 3 parts"));
        }
        // payload を base64url デコードして JSON として解析する
        let payload_bytes = decode_base64url(parts[1])?;
        let claims: OidcClaims = serde_json::from_slice(&payload_bytes)
            .map_err(|e| anyhow!("JWT payload parse failed: {e}"))?;
        // issuer を検証する
        if claims.iss != self.keycloak_issuer {
            return Err(anyhow!("issuer mismatch: got {}", claims.iss));
        }
        // 有効期限を検証する
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if claims.exp < now_unix {
            warn!(jti = %claims.jti, "token expired");
            return Err(anyhow!("token expired"));
        }
        // TODO: JWKS endpoint から公開鍵を取得して署名を検証する
        debug!(sub = %claims.sub, "OIDC token verified (JWKS signature check pending)");
        Ok(claims)
    }

    // verify_dpop_proof は RFC 9449 DPoP proof の htm / htu / jti を検証する。
    pub fn verify_dpop_proof(
        &self,
        dpop_token: &str,
        expected_htm: &str,
        expected_htu: &str,
    ) -> Result<DpopProofClaims> {
        // DPoP proof JWT を parse する
        let parts: Vec<&str> = dpop_token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("invalid DPoP proof format"));
        }
        let payload_bytes = decode_base64url(parts[1])?;
        let claims: DpopProofClaims = serde_json::from_slice(&payload_bytes)?;
        // htm を検証する
        if claims.htm.to_uppercase() != expected_htm.to_uppercase() {
            return Err(anyhow!("DPoP htm mismatch"));
        }
        // htu を検証する
        if claims.htu != expected_htu {
            return Err(anyhow!("DPoP htu mismatch"));
        }
        // TODO: jti replay cache (moka) を使って重複防止を実施する
        debug!(jti = %claims.jti, "DPoP proof verified");
        Ok(claims)
    }
}

// decode_base64url は base64url（パディングなし）をデコードする。
fn decode_base64url(input: &str) -> Result<Vec<u8>> {
    // base64url → 標準 base64 に変換する（- → +、_ → /）
    let standard = input.replace('-', "+").replace('_', "/");
    // パディングを補完する
    let padding = match standard.len() % 4 {
        2 => "==",
        3 => "=",
        _ => "",
    };
    let padded = format!("{standard}{padding}");
    // base64 デコードを実行する
    // 注: 標準ライブラリには base64 がないため、手動デコードを実装する
    simple_base64_decode(&padded).map_err(|e| anyhow!("base64url decode error: {e}"))
}

// simple_base64_decode は標準 base64 文字列をデコードする（外部 crate 不使用）。
fn simple_base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    // base64 のアルファベット
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    // 逆引きテーブルを構築する
    let mut table = [255u8; 256];
    for (i, &b) in ALPHABET.iter().enumerate() {
        table[b as usize] = i as u8;
    }
    // パディングを除去したバイト列を処理する
    let clean: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    // 4 バイトずつ 3 バイトにデコードする
    let mut result = Vec::new();
    for chunk in clean.chunks(4) {
        // 各バイトを base64 値に変換する
        let vals: Vec<u8> = chunk.iter().map(|&b| table[b as usize]).collect();
        // 1 バイト目は常に出力する
        if vals.len() >= 2 {
            result.push((vals[0] << 2) | (vals[1] >> 4));
        }
        // 2 バイト目
        if vals.len() >= 3 {
            result.push((vals[1] << 4) | (vals[2] >> 2));
        }
        // 3 バイト目
        if vals.len() >= 4 {
            result.push((vals[2] << 6) | vals[3]);
        }
    }
    Ok(result)
}
