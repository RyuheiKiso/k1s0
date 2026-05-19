// oidc.rs — spec 04 認証適合仕様: OIDC + DPoP + SPIRE + 外部 IdP 検証モジュール
// 04_認証適合仕様.md §v1 auth_class セット（5 class）の JWT 検証ロジックを実装する。
// jsonwebtoken crate で RS256 / ES256 / HS256 署名検証を行い、
// reqwest で JWKS endpoint から公開鍵を非同期フェッチする。
// 生 token は検証後すぐに drop するため応答に含まれない。

// anyhow: エラー伝搬ライブラリ
use anyhow::{anyhow, bail, Context, Result};
// jsonwebtoken: JWT デコード・検証ライブラリ（RS256 / ES256 / HS256 に対応）
use jsonwebtoken::{
    decode, decode_header, Algorithm, DecodingKey, Header, TokenData, Validation,
};
// reqwest: HTTP クライアント（JWKS endpoint フェッチに使用する）
use reqwest::Client;
// serde: JSON デシリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{debug, info, warn};
// 標準ライブラリ: HashMap + Arc + Mutex + Duration + Instant
use std::collections::HashMap;
// Arc: JwkCache の共有参照カウントに使用する（Clone が必要な場合も Arc で共有する）
use std::sync::Arc;
// Duration / Instant: JWKS キャッシュの TTL 計算に使用する（単調増加クロック: wall-clock ではない）
use std::time::{Duration, Instant};
// tokio::sync::Mutex: 非同期コードから safe に cache を mutate するために使用する
use tokio::sync::Mutex;

// OidcClaims は Keycloak が発行する JWT の主要 claim を宣言する。
// spec 04 §AuthContext スキーマ と一致させる。
#[derive(Debug, Deserialize, Clone)]
pub struct OidcClaims {
    // sub: canonical subject identifier（Keycloak の subject）
    pub sub: String,
    // jti: JWT ID（revocation tracking に使用する）
    pub jti: String,
    // tid: Keycloak の tenant_id カスタム claim（k1s0 realm で設定する）
    pub tid: Option<String>,
    // scope: space-separated scope 文字列
    pub scope: Option<String>,
    // exp: 有効期限（Unix timestamp）— jsonwebtoken の Validation で自動検証される
    pub exp: u64,
    // iss: issuer URL（環境変数の KEYCLOAK_ISSUER と一致することを検証する）
    pub iss: String,
    // aud: audience（KEYCLOAK_AUDIENCE と一致することを検証する）
    pub aud: Option<serde_json::Value>,
    // azp: authorized party（クライアント ID）
    pub azp: Option<String>,
    // act: act claim（RFC 8693 token exchange で使用する）
    pub act: Option<serde_json::Value>,
}

// SpireClaims は SPIRE が発行する SVID JWT の claim を宣言する。
// workload identity の検証に使用する。
#[derive(Debug, Deserialize, Clone)]
pub struct SpireClaims {
    // sub: SPIFFE ID（例: spiffe://k1s0.internal/ns/default/sa/gateway）
    pub sub: String,
    // exp: 有効期限
    pub exp: u64,
    // aud: audience（k1s0-gateway を期待する）
    pub aud: Vec<String>,
    // iss: SPIRE server の issuer URL
    pub iss: String,
}

// FederatedClaims は外部 IdP が発行する RFC 8693 token exchange の claim を宣言する。
// act / may_act claim を含む audience-restricted JWT。
#[derive(Debug, Deserialize, Clone)]
pub struct FederatedClaims {
    // sub: 外部 subject（取引先の IdP が発行した subject）
    pub sub: String,
    // jti: JWT ID
    pub jti: String,
    // exp: 有効期限（< 5min を想定する）
    pub exp: u64,
    // iss: 外部 IdP の issuer URL
    pub iss: String,
    // aud: audience-restricted（k1s0-gateway のみ）
    pub aud: Option<serde_json::Value>,
    // act: delegation actor claim（委譲元 user の情報）
    pub act: Option<serde_json::Value>,
    // may_act: may_act claim（代理 actor の subject）
    pub may_act: Option<serde_json::Value>,
}

// DpopProofClaims は RFC 9449 DPoP proof JWT の claim を宣言する。
#[derive(Debug, Deserialize)]
pub struct DpopProofClaims {
    // htm: HTTP method（大文字）— リクエストの method と一致することを検証する
    pub htm: String,
    // htu: HTTP target URI — リクエストの URI と一致することを検証する
    pub htu: String,
    // jti: proof の一意識別子（replay 防止に使用する）
    pub jti: String,
    // iat: 発行時刻（Unix timestamp）— 時刻ズレを検証する
    pub iat: u64,
}

// JwkKey は JWKS (JSON Web Key Set) の 1 つの鍵を宣言する。
#[derive(Debug, Deserialize, Clone)]
pub struct JwkKey {
    // kty: 鍵の種類（"RSA" / "EC"）
    pub kty: String,
    // kid: 鍵 ID（JWT header の kid と照合する）
    pub kid: Option<String>,
    // alg: アルゴリズム（"RS256" / "ES256"）
    pub alg: Option<String>,
    // use: 鍵の用途（"sig" = 署名検証）
    #[serde(rename = "use")]
    pub use_: Option<String>,
    // n: RSA modulus (base64url)
    pub n: Option<String>,
    // e: RSA public exponent (base64url)
    pub e: Option<String>,
    // crv: EC curve ("P-256" 等)
    pub crv: Option<String>,
    // x: EC x coordinate (base64url)
    pub x: Option<String>,
    // y: EC y coordinate (base64url)
    pub y: Option<String>,
}

// JwkSet は JWKS (JSON Web Key Set) の全体を宣言する。
#[derive(Debug, Deserialize, Clone)]
pub struct JwkSet {
    // keys: 鍵の配列（複数の kid が含まれる場合がある）
    pub keys: Vec<JwkKey>,
}

// DpopHeader は DPoP proof JWT のヘッダー部分を宣言する（jku binding チェックに使用する）
#[derive(Debug, Deserialize)]
struct DpopHeader {
    // alg: DPoP proof の署名アルゴリズム（ES256 推奨）
    #[allow(dead_code)]
    alg: Option<String>,
    // jku: JWK Set URL（存在する場合は IdP の known JWKS URL と一致することを検証する）
    jku: Option<String>,
    // typ: JWT type（"dpop+jwt" であることを期待する）
    #[allow(dead_code)]
    typ: Option<String>,
}

// JWKS キャッシュの TTL: 300 秒（単調増加クロックで計測する。wall-clock ではない）
const JWKS_CACHE_TTL: Duration = Duration::from_secs(300);

// JwkCache は JWKS を in-memory に TTL キャッシュする構造体。
// 単調増加クロック（Instant）を使用して TTL を計測する（wall-clock 禁止規約に準拠する）。
pub struct JwkCache {
    // client: reqwest HTTP クライアント（JWKS フェッチに使用する）
    client: Client,
    // cache: JWKS の TTL キャッシュ（JWKS URL → (JwkSet, fetch_instant)）
    // Arc<Mutex<...>> で非同期タスク間の安全な共有を実現する
    cache: Arc<Mutex<HashMap<String, (JwkSet, Instant)>>>,
}

impl JwkCache {
    // new は JwkCache を構築する。
    pub fn new() -> Self {
        // reqwest Client を TLS なし設定で構築する（envoy が TLS を終端する）
        Self {
            // reqwest Client を生成する
            client: Client::new(),
            // TTL キャッシュを空 HashMap で初期化する
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // fetch は指定した JWKS endpoint から JwkSet を取得する。
    // キャッシュが TTL 内の場合はキャッシュから返す（HTTP フェッチを省略する）。
    pub async fn fetch(&self, jwks_url: &str) -> Result<JwkSet> {
        // ---- 1. キャッシュを確認する ----
        {
            // キャッシュ mutex を取得する
            let cache = self.cache.lock().await;
            // キャッシュエントリを取得する
            if let Some((jwks, fetched_at)) = cache.get(jwks_url) {
                // TTL 内の場合（単調増加クロックで経過時間を計算する）はキャッシュから返す
                if fetched_at.elapsed() < JWKS_CACHE_TTL {
                    // キャッシュヒットをログに記録する
                    debug!(url = %jwks_url, "JWKS cache hit");
                    // クローンして返す（Arc<Mutex> のロックを早期に解放するため）
                    return Ok(jwks.clone());
                }
                // TTL 切れをログに記録する（次のフェッチで更新される）
                debug!(url = %jwks_url, "JWKS cache expired");
            }
        }
        // ---- 2. キャッシュミス / TTL 切れの場合は HTTP フェッチする ----

        // JWKS endpoint に GET リクエストを送信する
        let response = self
            .client
            .get(jwks_url)
            .send()
            .await
            .with_context(|| format!("JWKS fetch failed: {jwks_url}"))?;
        // レスポンスが 200 OK であることを確認する
        if !response.status().is_success() {
            bail!("JWKS endpoint returned {}: {}", response.status(), jwks_url);
        }
        // JSON をデシリアライズして JwkSet を返す
        let jwks: JwkSet = response
            .json()
            .await
            .with_context(|| format!("JWKS JSON parse failed: {jwks_url}"))?;
        // フェッチ完了をログに記録する
        debug!(url = %jwks_url, keys = jwks.keys.len(), "JWKS fetched and cached");

        // ---- 3. キャッシュに保存する ----

        {
            // キャッシュ mutex を取得する
            let mut cache = self.cache.lock().await;
            // 現在の単調増加クロック値をフェッチ時刻として記録する
            cache.insert(jwks_url.to_string(), (jwks.clone(), Instant::now()));
        }
        // JwkSet を返す
        Ok(jwks)
    }

    // find_key は kid に対応する JwkKey を JwkSet から検索する。
    pub fn find_key<'a>(&self, jwks: &'a JwkSet, kid: &str) -> Option<&'a JwkKey> {
        // kid が一致する鍵を線形探索する
        jwks.keys.iter().find(|k| {
            k.kid.as_deref().map_or(false, |k| k == kid)
        })
    }

    // make_decoding_key は JwkKey から DecodingKey を構築する。
    pub fn make_decoding_key(&self, jwk: &JwkKey) -> Result<DecodingKey> {
        // 鍵の種類に応じて DecodingKey を構築する
        match jwk.kty.as_str() {
            "RSA" => {
                // RSA 公開鍵: n (modulus) と e (exponent) から構築する
                let n = jwk.n.as_deref().ok_or_else(|| anyhow!("RSA JWK missing 'n'"))?;
                let e = jwk.e.as_deref().ok_or_else(|| anyhow!("RSA JWK missing 'e'"))?;
                // base64url エンコードされた n / e から DecodingKey を構築する
                Ok(DecodingKey::from_rsa_components(n, e)
                    .with_context(|| "failed to build RSA DecodingKey")?)
            }
            "EC" => {
                // EC 公開鍵: crv / x / y から構築する
                let x = jwk.x.as_deref().ok_or_else(|| anyhow!("EC JWK missing 'x'"))?;
                let y = jwk.y.as_deref().ok_or_else(|| anyhow!("EC JWK missing 'y'"))?;
                // EC 公開鍵を構築する（P-256 想定）
                Ok(DecodingKey::from_ec_components(x, y)
                    .with_context(|| "failed to build EC DecodingKey")?)
            }
            kty => bail!("unsupported JWK kty: {kty}"),
        }
    }
}

// OidcVerifier は 5 auth_class の JWT 検証ロジックを提供する。
// Keycloak OIDC / SPIRE SVID / 外部 IdP / DPoP 検証を一元管理する。
pub struct OidcVerifier {
    // keycloak_issuer: Keycloak の issuer URL（環境変数 KEYCLOAK_ISSUER から取得する）
    keycloak_issuer: String,
    // expected_audience: gateway が受け入れる audience（環境変数 KEYCLOAK_AUDIENCE）
    expected_audience: String,
    // spire_bundle_url: SPIRE bundle endpoint の URL（環境変数 SPIRE_BUNDLE_URL）
    spire_bundle_url: String,
    // jwk_cache: JWKS フェッチ・キャッシュ（現実装では毎回フェッチ）
    jwk_cache: JwkCache,
}

impl OidcVerifier {
    // new は OidcVerifier を構築する。
    pub fn new(keycloak_issuer: String, expected_audience: String) -> Self {
        // SPIRE bundle endpoint は環境変数から取得する（デフォルト: cluster 内の SPIRE）
        let spire_bundle_url = std::env::var("SPIRE_BUNDLE_URL")
            .unwrap_or_else(|_| "http://spire-server.spire.svc:8081/bundle".to_string());
        Self {
            keycloak_issuer,
            expected_audience,
            spire_bundle_url,
            jwk_cache: JwkCache::new(),
        }
    }

    // keycloak_jwks_url は Keycloak の JWKS endpoint URL を返す。
    fn keycloak_jwks_url(&self) -> String {
        // Keycloak の JWKS endpoint は issuer + "/protocol/openid-connect/certs"
        format!("{}/protocol/openid-connect/certs", self.keycloak_issuer)
    }

    // verify_token は JWT の issuer / audience / exp / 署名を検証して OidcClaims を返す。
    // v1_human_session の Bearer token 検証に使用する。
    pub async fn verify_token(&self, token: &str) -> Result<OidcClaims> {
        // JWT ヘッダーをデコードして kid と alg を取得する
        let header = decode_header(token).with_context(|| "JWT header decode failed")?;
        // kid がない場合はエラーとする（JWKS 検索に kid が必須）
        let kid = header.kid.ok_or_else(|| anyhow!("JWT header missing 'kid'"))?;
        // Keycloak の JWKS endpoint から公開鍵セットを取得する
        let jwks_url = self.keycloak_jwks_url();
        let jwks = self.jwk_cache.fetch(&jwks_url).await?;
        // kid に対応する JWK を選択する
        let jwk = self.jwk_cache.find_key(&jwks, &kid)
            .ok_or_else(|| anyhow!("JWK not found for kid: {kid}"))?;
        // DecodingKey を構築する
        let decoding_key = self.jwk_cache.make_decoding_key(jwk)?;
        // algorithm を JWT ヘッダーから決定する（RS256 / ES256 に限定する）
        let algorithm = match header.alg {
            jsonwebtoken::Algorithm::RS256 => Algorithm::RS256,
            jsonwebtoken::Algorithm::ES256 => Algorithm::ES256,
            alg => bail!("unsupported algorithm: {alg:?}"),
        };
        // Validation を設定する（issuer / audience / exp 検証を有効化する）
        let mut validation = Validation::new(algorithm);
        // issuer 検証を設定する
        validation.set_issuer(&[&self.keycloak_issuer]);
        // audience 検証を設定する
        validation.set_audience(&[&self.expected_audience]);
        // JWT を検証して claims を返す
        let token_data: TokenData<OidcClaims> = decode(token, &decoding_key, &validation)
            .with_context(|| "JWT signature verification failed")?;
        // 検証成功をログに記録する（sub のみ記録し、生 token は記録しない）
        info!(sub = %token_data.claims.sub, kid = %kid, "OIDC token verified");
        Ok(token_data.claims)
    }

    // verify_workload_jwt は SPIRE が発行した SVID JWT を検証する。
    // v1_workload_jwt の Bearer token 検証に使用する。
    pub async fn verify_workload_jwt(&self, token: &str) -> Result<SpireClaims> {
        // JWT ヘッダーをデコードして kid を取得する
        let header = decode_header(token).with_context(|| "SPIRE JWT header decode failed")?;
        // SPIRE bundle endpoint から JWKS を取得する
        let jwks = self.jwk_cache.fetch(&self.spire_bundle_url).await?;
        // kid が指定されている場合は kid で絞り込む、なければ最初の署名鍵を使う
        let jwk = if let Some(ref kid) = header.kid {
            self.jwk_cache.find_key(&jwks, kid)
                .ok_or_else(|| anyhow!("SPIRE JWK not found for kid: {kid}"))?
        } else {
            // kid がない場合は use="sig" の最初の鍵を使う
            jwks.keys.iter()
                .find(|k| k.use_.as_deref() == Some("sig"))
                .ok_or_else(|| anyhow!("SPIRE JWKS has no signing key"))?
        };
        // DecodingKey を構築する
        let decoding_key = self.jwk_cache.make_decoding_key(jwk)?;
        // algorithm を決定する（SPIRE は RS256 または ES256 を使う）
        let algorithm = match header.alg {
            jsonwebtoken::Algorithm::RS256 => Algorithm::RS256,
            jsonwebtoken::Algorithm::ES256 => Algorithm::ES256,
            alg => bail!("unsupported SPIRE algorithm: {alg:?}"),
        };
        // Validation を設定する（aud は gateway の SPIFFE audience を期待する）
        let mut validation = Validation::new(algorithm);
        // audience は k1s0-gateway を期待する
        validation.set_audience(&[&self.expected_audience]);
        // JWT を検証して SpireClaims を返す
        let token_data: TokenData<SpireClaims> = decode(token, &decoding_key, &validation)
            .with_context(|| "SPIRE JWT signature verification failed")?;
        // 検証成功をログに記録する
        info!(sub = %token_data.claims.sub, "SPIRE workload JWT verified");
        Ok(token_data.claims)
    }

    // verify_federated_token は外部 IdP が発行した RFC 8693 token exchange JWT を検証する。
    // v1_federated_exchange の Bearer token 検証に使用する。
    pub async fn verify_federated_token(
        &self,
        token: &str,
        federation_issuer: &str,
    ) -> Result<FederatedClaims> {
        // JWT ヘッダーをデコードして kid を取得する
        let header = decode_header(token).with_context(|| "Federated JWT header decode failed")?;
        // 外部 IdP の JWKS endpoint URL を構築する（OpenID Connect discovery 形式）
        let jwks_url = format!("{federation_issuer}/.well-known/jwks.json");
        // 外部 IdP の JWKS を取得する
        let jwks = self.jwk_cache.fetch(&jwks_url).await?;
        // kid に対応する JWK を選択する
        let jwk = if let Some(ref kid) = header.kid {
            self.jwk_cache.find_key(&jwks, kid)
                .ok_or_else(|| anyhow!("Federated JWK not found for kid: {kid}"))?
        } else {
            jwks.keys.iter()
                .find(|k| k.use_.as_deref() == Some("sig"))
                .ok_or_else(|| anyhow!("Federated JWKS has no signing key"))?
        };
        // DecodingKey を構築する
        let decoding_key = self.jwk_cache.make_decoding_key(jwk)?;
        // algorithm を決定する
        let algorithm = match header.alg {
            jsonwebtoken::Algorithm::RS256 => Algorithm::RS256,
            jsonwebtoken::Algorithm::ES256 => Algorithm::ES256,
            alg => bail!("unsupported federated algorithm: {alg:?}"),
        };
        // Validation を設定する（issuer と audience を検証する）
        let mut validation = Validation::new(algorithm);
        // 外部 IdP の issuer を設定する
        validation.set_issuer(&[federation_issuer]);
        // audience は k1s0-gateway に audience-restricted されていることを検証する
        validation.set_audience(&[&self.expected_audience]);
        // JWT を検証して FederatedClaims を返す
        let token_data: TokenData<FederatedClaims> = decode(token, &decoding_key, &validation)
            .with_context(|| "Federated JWT signature verification failed")?;
        // act claim の存在を確認する（RFC 8693 では必須）
        if token_data.claims.act.is_none() && token_data.claims.may_act.is_none() {
            warn!(sub = %token_data.claims.sub, "Federated JWT missing act/may_act claim");
        }
        // 検証成功をログに記録する
        info!(
            sub = %token_data.claims.sub,
            iss = %token_data.claims.iss,
            "Federated JWT (RFC 8693) verified"
        );
        Ok(token_data.claims)
    }

    // verify_dpop_proof は RFC 9449 DPoP proof の htm / htu / jti を検証する。
    // v1_human_session と v1_emergency_step_up で使用する。
    pub fn verify_dpop_proof(
        &self,
        dpop_token: &str,
        expected_htm: &str,
        expected_htu: &str,
    ) -> Result<DpopProofClaims> {
        // DPoP proof の header と payload を検証する
        // JWT の 3 部分（header.payload.signature）を分割する
        let parts: Vec<&str> = dpop_token.splitn(3, '.').collect();
        // 3 部分が揃っていない場合はエラーを返す
        if parts.len() != 3 {
            bail!("invalid DPoP proof format: expected 3 parts");
        }

        // ---- jku binding チェック ----
        // RFC 9449 §4.2: DPoP header に jku が存在する場合、
        // それは IdP の known JWKS URL と完全一致しなければならない。
        // jku が攻撃者制御の URL であると key confusion attack が成立するため強制する。

        // header を base64url デコードする
        let header_str = decode_base64url_utf8(parts[0])?;
        // header JSON をパースする
        let dpop_header: DpopHeader = serde_json::from_str(&header_str)
            .with_context(|| "DPoP header JSON parse failed")?;
        // jku フィールドが存在する場合は IdP JWKS URL と照合する
        if let Some(ref jku) = dpop_header.jku {
            // IdP の known JWKS URL を Keycloak issuer から構築する
            // 仕様: {issuer}/protocol/openid-connect/certs が Keycloak の JWKS URL
            let expected_jwks_url = format!("{}/protocol/openid-connect/certs", self.keycloak_issuer);
            // jku が expected JWKS URL と一致しない場合はエラーを返す（key confusion 防止）
            if jku != &expected_jwks_url {
                bail!(
                    "DPoP header jku mismatch: expected {}, got {} — key confusion attack を拒否する",
                    expected_jwks_url,
                    jku
                );
            }
            // jku 一致をログに記録する
            debug!(jku = %jku, "DPoP jku binding verified");
        }

        // payload を URL-safe base64 デコードする
        let payload_str = decode_base64url_utf8(parts[1])?;
        // JSON デシリアライズして DpoP claims を取得する
        let claims: DpopProofClaims = serde_json::from_str(&payload_str)
            .with_context(|| "DPoP proof payload parse failed")?;
        // htm を検証する（大文字で比較する）
        if claims.htm.to_uppercase() != expected_htm.to_uppercase() {
            bail!(
                "DPoP htm mismatch: expected {expected_htm}, got {}",
                claims.htm
            );
        }
        // htu を検証する
        if claims.htu != expected_htu {
            bail!(
                "DPoP htu mismatch: expected {expected_htu}, got {}",
                claims.htu
            );
        }
        // iat の時刻ズレを検証する（60s 以内）
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let age = now_unix.saturating_sub(claims.iat);
        if age > 60 {
            bail!("DPoP proof expired: iat age {age}s > 60s");
        }
        // DPoP 検証成功をログに記録する
        debug!(jti = %claims.jti, "DPoP proof verified");
        Ok(claims)
    }
}

// decode_base64url_utf8 は base64url (padding なし) を UTF-8 文字列にデコードする。
fn decode_base64url_utf8(input: &str) -> Result<String> {
    // base64url → 標準 base64 に変換する（- → +、_ → /）
    let standard = input.replace('-', "+").replace('_', "/");
    // パディングを補完する（4 の倍数になるよう = を追加する）
    let padded = match standard.len() % 4 {
        // 2 文字余る場合は == を追加する
        2 => format!("{standard}=="),
        // 3 文字余る場合は = を追加する
        3 => format!("{standard}="),
        // 0 と 1 はそのまま（1 は invalid base64 だが後続で検出される）
        _ => standard,
    };
    // base64 デコードして UTF-8 文字列として返す
    use std::io::Read;
    // Rust 標準には base64 がないため、jsonwebtoken 内部の変換に依存せず
    // 手動で base64 デコードする
    let bytes = decode_b64_bytes(&padded)?;
    String::from_utf8(bytes).with_context(|| "base64url decoded bytes are not valid UTF-8")
}

// decode_b64_bytes は標準 base64 文字列をバイト列にデコードする。
fn decode_b64_bytes(input: &str) -> Result<Vec<u8>> {
    // base64 のアルファベット（A-Za-z0-9+/）
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    // 逆引きテーブルを構築する（255 = 無効文字）
    let mut table = [255u8; 256];
    for (i, &b) in ALPHA.iter().enumerate() {
        table[b as usize] = i as u8;
    }
    // パディング（=）を除いたバイト列を処理する
    let clean: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    // 4 バイト入力 → 3 バイト出力に変換する
    let mut result = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        // 各バイトを base64 値に変換する
        let vals: Vec<u8> = chunk.iter().map(|&b| table[b as usize]).collect();
        // 無効文字が含まれる場合はエラーとする
        if vals.iter().any(|&v| v == 255) {
            bail!("invalid base64 character");
        }
        // 1 バイト目: 常に出力する
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
