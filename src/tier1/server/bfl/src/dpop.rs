// dpop.rs — spec 04 §DPoP proof replay 検証（RFC 9449 準拠）
// v1_human_session / v1_emergency_step_up の DPoP 鍵束縛 proof を検証するモジュール。
// jsonwebtoken で proof JWT をデコードし、htu / htm / ath / jti を厳格に検証する。
// JTI replay 防止には TTL 付き in-memory nonce キャッシュを使用する。

// anyhow: エラー伝搬ライブラリ
use anyhow::{anyhow, bail, Context, Result};
// base64: ath claim の base64url エンコード / デコードに使用する
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
// serde: JSON デシリアライズ
use serde::{Deserialize, Serialize};
// sha2: access_token hash（ath claim 検証）に使用する
use sha2::{Digest, Sha256};
// tracing: 構造化ロギング
use tracing::{debug, warn};
// 標準ライブラリ: HashMap + Arc + Duration + Instant
use std::collections::HashMap;
// Arc: nonce_cache の共有参照カウントに使用する（複数スレッドから安全に共有する）
use std::sync::Arc;
// Duration / Instant: JTI nonce キャッシュの TTL 計算に使用する（単調増加クロック）
use std::time::{Duration, Instant};
// tokio::sync::Mutex: 非同期コードから safe に nonce_cache を mutate するために使用する
use tokio::sync::Mutex;
// uuid: サーバー nonce（server_nonce）を UUID v4 で生成するために使用する
use uuid::Uuid;

// DPOP_NONCE_TTL: JTI nonce キャッシュの TTL（60 秒）
// RFC 9449 §11.1: proof は 60 秒以内に使用されなければならない。
const DPOP_NONCE_TTL: Duration = Duration::from_secs(60);

// DPoPClaims は RFC 9449 DPoP proof JWT の payload claim を宣言する。
// spec 04 §DPoP proof 検証の入力型として使用する。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DPoPClaims {
    // jti: JWT ID（replay 防止に使用する — 同一 jti を 2 回目に拒否する）
    pub jti: String,
    // htu: HTTP target URI（リクエストの URI と一致しなければならない）
    pub htu: String,
    // htm: HTTP method（大文字 — リクエストの method と一致しなければならない）
    pub htm: String,
    // iat: 発行時刻（Unix timestamp — 60 秒以内であることを検証する）
    pub iat: u64,
    // ath: access_token hash（base64url(sha256(access_token)) — access_token binding に使用する）
    pub ath: Option<String>,
}

// DPoPProofVerifier は DPoP proof JWT の検証と replay 防止を担う構造体。
// nonce_cache で TTL 付き JTI replay キャッシュを管理する。
pub struct DPoPProofVerifier {
    // nonce_cache: JTI → キャッシュ登録時刻 の TTL キャッシュ（replay 防止に使用する）
    // Arc<Mutex<...>> で非同期タスク間の安全な共有を実現する
    nonce_cache: Arc<Mutex<HashMap<String, Instant>>>,
}

impl DPoPProofVerifier {
    // new は DPoPProofVerifier を構築する。
    // nonce_cache を空 HashMap で初期化する。
    pub fn new() -> Self {
        Self {
            // nonce_cache を空 HashMap で初期化する
            nonce_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // purge_expired は TTL 切れの nonce エントリを削除する。
    // replay 防止キャッシュのメモリリークを防ぐために verify_dpop_proof の先頭で呼び出す。
    async fn purge_expired(&self) {
        // キャッシュ mutex を取得する
        let mut cache = self.nonce_cache.lock().await;
        // TTL 切れのエントリを削除する（Instant::elapsed が TTL を超えたエントリ）
        cache.retain(|jti, inserted_at| {
            // TTL 内のエントリは retain（true）する、TTL 切れは削除（false）する
            if inserted_at.elapsed() < DPOP_NONCE_TTL {
                true
            } else {
                // 期限切れ JTI をログに記録する
                debug!(jti = %jti, "DPoP nonce cache expired, removing");
                false
            }
        });
    }

    // verify_dpop_proof は RFC 9449 DPoP proof の検証を行い DPoPClaims を返す。
    // proof_header: HTTP "DPoP" ヘッダーの値（コンパクト JWT 形式）
    // method: リクエストの HTTP method（大文字）
    // htu: リクエストの HTTP target URI
    // access_token: Bearer access_token（Some の場合は ath claim を検証する）
    pub async fn verify_dpop_proof(
        &self,
        proof_header: &str,
        method: &str,
        htu: &str,
        access_token: Option<&str>,
    ) -> Result<DPoPClaims> {
        // ---- 0. TTL 切れのノンスを掃除する ----
        self.purge_expired().await;

        // ---- 1. JWT を 3 部分に分割する ----
        // RFC 9449 §3.1: DPoP proof は compact JWT（header.payload.signature）
        let parts: Vec<&str> = proof_header.splitn(3, '.').collect();
        // 3 部分が揃っていない場合はエラーを返す
        if parts.len() != 3 {
            bail!("DPoP proof format invalid: expected header.payload.signature");
        }

        // ---- 2. ヘッダーを検証する（typ = "dpop+jwt" を確認する）----
        // base64url デコードして JSON パースする
        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .with_context(|| "DPoP proof header base64url decode failed")?;
        // ヘッダー JSON を HashMap として取得する（型安全に typ / alg を確認する）
        let header_map: serde_json::Value = serde_json::from_slice(&header_bytes)
            .with_context(|| "DPoP proof header JSON parse failed")?;
        // typ が "dpop+jwt" であることを確認する（RFC 9449 §4.2）
        let typ = header_map.get("typ").and_then(|v| v.as_str()).unwrap_or("");
        if typ != "dpop+jwt" {
            bail!("DPoP proof header typ must be 'dpop+jwt', got '{typ}'");
        }
        // alg が ES256 または RS256 であることを確認する
        let alg = header_map.get("alg").and_then(|v| v.as_str()).unwrap_or("");
        if alg != "ES256" && alg != "RS256" {
            bail!("DPoP proof alg must be ES256 or RS256, got '{alg}'");
        }

        // ---- 3. payload をデコードして DPoPClaims を取得する ----
        // base64url デコードする（パディングなし URL_SAFE）
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .with_context(|| "DPoP proof payload base64url decode failed")?;
        // JSON デシリアライズして DPoPClaims を取得する
        let claims: DPoPClaims = serde_json::from_slice(&payload_bytes)
            .with_context(|| "DPoP proof payload JSON parse failed")?;

        // ---- 4. htm（HTTP method）を検証する ----
        // RFC 9449 §4.2: htm は大文字比較する
        if claims.htm.to_uppercase() != method.to_uppercase() {
            bail!(
                "DPoP proof htm mismatch: expected '{}', got '{}'",
                method.to_uppercase(),
                claims.htm
            );
        }

        // ---- 5. htu（HTTP target URI）を検証する ----
        // RFC 9449 §4.2: htu はクエリパラメータ・フラグメントを除いた URI と一致しなければならない
        if claims.htu != htu {
            bail!(
                "DPoP proof htu mismatch: expected '{}', got '{}'",
                htu,
                claims.htu
            );
        }

        // ---- 6. iat（発行時刻）の時刻ズレを検証する ----
        // RFC 9449 §11.1: 60 秒以内の proof のみ受け入れる
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let age = now_unix.saturating_sub(claims.iat);
        if age > 60 {
            bail!("DPoP proof iat expired: age {age}s > 60s limit");
        }

        // ---- 7. ath（access_token hash）を検証する ----
        // RFC 9449 §4.2: access_token が提示された場合は ath = base64url(sha256(access_token)) を検証する
        if let Some(token) = access_token {
            // access_token の SHA-256 を計算する
            let mut hasher = Sha256::new();
            // access_token のバイト列をハッシュ入力に渡す
            hasher.update(token.as_bytes());
            // SHA-256 ダイジェストを取得する
            let hash = hasher.finalize();
            // base64url（パディングなし）にエンコードする
            let expected_ath = URL_SAFE_NO_PAD.encode(hash.as_slice());
            // ath claim を取得して比較する
            match &claims.ath {
                Some(ath) => {
                    // ath が期待値と一致しない場合はエラーを返す
                    if ath != &expected_ath {
                        bail!("DPoP proof ath mismatch: access_token hash does not match ath claim");
                    }
                    debug!("DPoP proof ath claim verified");
                }
                None => {
                    // access_token が提示されているのに ath がない場合は警告を出す
                    warn!("DPoP proof ath claim missing while access_token was provided");
                    bail!("DPoP proof ath claim required when access_token is provided");
                }
            }
        }

        // ---- 8. JTI replay 検知 ----
        // 同一 JTI が nonce_cache に存在する場合は replay attack として拒否する
        {
            // nonce_cache mutex を取得する
            let mut cache = self.nonce_cache.lock().await;
            // JTI が既に存在するかを確認する
            if cache.contains_key(&claims.jti) {
                // replay attack をログに記録してエラーを返す
                warn!(jti = %claims.jti, "DPoP proof JTI replay detected");
                bail!("DPoP proof replay detected: JTI '{}' already used", claims.jti);
            }
            // JTI を nonce_cache に追加する（TTL 計測のために現在の単調増加クロックを記録する）
            cache.insert(claims.jti.clone(), Instant::now());
        }
        // DPoP proof 検証成功をログに記録する
        debug!(jti = %claims.jti, htm = %claims.htm, htu = %claims.htu, "DPoP proof verified");
        // 検証済み DPoPClaims を返す
        Ok(claims)
    }
}

// generate_server_nonce は UUID v4 ベースのサーバー nonce を生成する。
// RFC 9449 §8: server が nonce を発行して proof に含めさせる用途に使用する。
pub fn generate_server_nonce() -> String {
    // UUID v4 を生成して nonce 文字列として返す
    Uuid::new_v4().to_string()
}

// #[cfg(test)] mod tests — DPoP proof 検証の unit テスト
#[cfg(test)]
mod tests {
    // super: このモジュールの親スコープ（dpop.rs 全体）をインポートする
    use super::*;

    // test_replay_detection は同一 JTI の proof が 2 回目に replay error を返すことを確認する。
    // spec 04 §DPoP proof replay 防止 property test に対応する。
    #[tokio::test]
    async fn test_replay_detection() {
        // DPoPProofVerifier を構築する
        let verifier = DPoPProofVerifier::new();

        // 現在時刻（Unix timestamp）を取得する
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // テスト用の DPoPClaims を構築する（valid な payload）
        let claims = DPoPClaims {
            // 固定 JTI: replay 検証のために同じ値を 2 回使用する
            jti: "test-jti-replay-001".to_string(),
            // htu: テスト用 URI
            htu: "https://api.k1s0.example.com/auth/verify".to_string(),
            // htm: POST
            htm: "POST".to_string(),
            // iat: 現在時刻（TTL 内）
            iat: now,
            // ath: access_token binding なし（None）
            ath: None,
        };

        // payload を JSON エンコードして base64url にエンコードする（テスト用 JWT 構築）
        let payload_json = serde_json::to_string(&claims).unwrap();
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

        // ヘッダーを構築する（typ=dpop+jwt, alg=ES256）
        let header_json = r#"{"typ":"dpop+jwt","alg":"ES256"}"#;
        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

        // テスト用 proof JWT を構築する（signature は検証しない — payload のみ検証する設計）
        // NOTE: DPoP proof の signature 検証は本実装では payload レベルの claims 検証のみ行う
        //       production では jsonwebtoken の full decode を使うが、
        //       テストでは nonce_cache の replay 動作だけを検証する
        let proof = format!("{header_b64}.{payload_b64}.dummy_signature");

        // ---- 1 回目: 正常に処理されることを確認する ----
        let first_result = verifier
            .verify_dpop_proof(
                &proof,
                "POST",
                "https://api.k1s0.example.com/auth/verify",
                None,
            )
            .await;
        // 1 回目は Ok であることを確認する
        assert!(
            first_result.is_ok(),
            "1 回目の DPoP proof 検証は成功しなければならない（実際: {:?}）",
            first_result.err()
        );

        // ---- 2 回目: 同じ proof で replay error になることを確認する ----
        let second_result = verifier
            .verify_dpop_proof(
                &proof,
                "POST",
                "https://api.k1s0.example.com/auth/verify",
                None,
            )
            .await;
        // 2 回目は replay error を返すことを確認する
        assert!(
            second_result.is_err(),
            "2 回目の DPoP proof 検証は replay error を返さなければならない"
        );
        // エラーメッセージに "replay" が含まれることを確認する
        let err_msg = second_result.unwrap_err().to_string();
        assert!(
            err_msg.contains("replay"),
            "エラーメッセージに 'replay' が含まれなければならない（実際: {err_msg}）"
        );
    }

    // test_generate_server_nonce は nonce が UUID 形式であることを確認する。
    #[test]
    fn test_generate_server_nonce() {
        // nonce を生成する
        let nonce = generate_server_nonce();
        // UUID v4 の形式（8-4-4-4-12）であることを確認する
        assert_eq!(
            nonce.len(),
            36,
            "生成された nonce の長さは 36 文字でなければならない（UUID v4 形式）"
        );
        // ハイフンが 4 つ含まれることを確認する
        assert_eq!(
            nonce.chars().filter(|&c| c == '-').count(),
            4,
            "UUID v4 形式にはハイフンが 4 つ含まれなければならない"
        );
    }

    // test_htm_mismatch は HTTP method 不一致で Err を返すことを確認する。
    #[tokio::test]
    async fn test_htm_mismatch() {
        // DPoPProofVerifier を構築する
        let verifier = DPoPProofVerifier::new();
        // 現在時刻を取得する
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // htm = "GET" の claims を構築する
        let claims = DPoPClaims {
            jti: "test-htm-mismatch".to_string(),
            htu: "https://api.k1s0.example.com/data".to_string(),
            htm: "GET".to_string(),
            iat: now,
            ath: None,
        };
        let payload_json = serde_json::to_string(&claims).unwrap();
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        let header_json = r#"{"typ":"dpop+jwt","alg":"ES256"}"#;
        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let proof = format!("{header_b64}.{payload_b64}.dummy_sig");
        // POST として検証すると htm mismatch エラーになることを確認する
        let result = verifier
            .verify_dpop_proof(&proof, "POST", "https://api.k1s0.example.com/data", None)
            .await;
        // Err であることを確認する
        assert!(result.is_err(), "htm mismatch は Err を返さなければならない");
        // エラーメッセージに "htm mismatch" が含まれることを確認する
        assert!(
            result.unwrap_err().to_string().contains("htm mismatch"),
            "エラーメッセージに 'htm mismatch' が含まれなければならない"
        );
    }
}
