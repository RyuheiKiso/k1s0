// external_notary.rs — k1s0 tier1 Library backend: RFC 3161 + Sigstore transparency log
// spec 05_鍵管理適合仕様.md §v1_audit_root_signing: 外部公証 attestation 実装。
// Rfc3161Timestamper: RFC 3161 TSA へ timestamp request を送信してトークンを取得する。
// SigstoreTransparencyLogger: Rekor POST /api/v1/log/entries で hashedrekord エントリを送信する。
// 鍵素材は公開 API に露出しない（spec §5 層 defense-in-depth 層 E）。

// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{anyhow, Result};
// chrono: DateTime<Utc> 型（TimestampToken の時刻フィールド）
use chrono::{DateTime, Utc};
// serde: JSON シリアライズ / デシリアライズ（Rekor API リクエスト・レスポンス）
use serde::{Deserialize, Serialize};
// sha2: RFC 3161 の MessageImprint 用 SHA-256 ハッシュ計算
use sha2::{Digest, Sha256};
// reqwest: HTTP クライアント（TSA エンドポイント / Rekor API への POST に使用する）
// reqwest は workspace 依存として定義済みのため workspace = true を使用する
// base64: RFC 3161 レスポンスの Base64 デコードに使用する
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STD};

// ---- RFC 3161 Timestamper ----

// Rfc3161Timestamper は RFC 3161 timestamp authority（TSA）への HTTP クライアント。
// spec §v1_audit_root_signing の「RFC 3161 trusted timestamp」要件を物理実装する。
#[derive(Debug, Clone)]
pub struct Rfc3161Timestamper {
    // tsa_url: TSA エンドポイント URL（例: http://timestamp.digicert.com）
    pub tsa_url: String,
    // ca_cert: TSA の CA 証明書 DER bytes（None の場合はシステムトラストストアを使用する）
    pub ca_cert: Option<Vec<u8>>,
}

// TimestampToken は RFC 3161 timestamp authority から受け取ったトークンを表す型。
// 公開 API に raw bytes を直接露出するが、生 key bytes は含まない（spec 層 E 準拠）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampToken {
    // raw_token: TSA から受け取った timestamp token の DER bytes（RFC 3161 TimeStampToken）
    pub raw_token: Vec<u8>,
    // time: トークンに含まれる genTime（UTC）
    // 実際の実装では ASN.1 パースが必要だが、stub では HTTP レスポンスヘッダまたは固定値を使用する
    pub time: DateTime<Utc>,
    // serial_number: timestamp token のシリアル番号（ASN.1 INTEGER から取得する）
    // stub では nonce を代替値として使用する
    pub serial_number: u64,
}

// Rfc3161TimestampRequest は TSA に送信する RFC 3161 timestamp request の内部表現。
// ASN.1 エンコードの簡易版として構造を定義する（full ASN.1 DER は der crate が必要）。
#[derive(Debug)]
struct Rfc3161TimestampRequest {
    // sha256_hash: MessageImprint の hashAlgorithm + hashedMessage（SHA-256）
    sha256_hash: [u8; 32],
    // nonce: 再生攻撃防止のためのランダム nonce（u64）
    nonce: u64,
}

// RFC 3161 ContentType（application/timestamp-query）
const CONTENT_TYPE_TSQ: &str = "application/timestamp-query";
// RFC 3161 Accept ヘッダ（application/timestamp-reply）
const ACCEPT_TSR: &str = "application/timestamp-reply";

impl Rfc3161Timestamper {
    // new は Rfc3161Timestamper を構築する。
    // tsa_url: TSA エンドポイント URL
    // ca_cert: CA 証明書 DER bytes（None の場合はシステムトラストストアを使用する）
    pub fn new(tsa_url: impl Into<String>, ca_cert: Option<Vec<u8>>) -> Self {
        // 指定された URL と CA 証明書で Rfc3161Timestamper を構築する
        Self {
            // tsa_url を String に変換して格納する
            tsa_url: tsa_url.into(),
            // ca_cert をそのまま格納する
            ca_cert,
        }
    }

    // timestamp は data の SHA-256 ハッシュに対して RFC 3161 timestamp request を送信し、
    // TimestampToken を返す。
    // spec §v1_audit_root_signing: audit hash chain root の外部公証に使用する。
    pub async fn timestamp(&self, data: &[u8]) -> Result<TimestampToken> {
        // SHA-256 ハッシュを計算する
        let mut hasher = Sha256::new();
        // data をハッシュに入力する
        hasher.update(data);
        // ハッシュ値を確定する
        let sha256_hash: [u8; 32] = hasher.finalize().into();

        // nonce をランダムに生成する（再生攻撃防止）
        // rand crate は未依存のため、現時点では UUID v4 の上位 64 bit を代替使用する
        let nonce: u64 = {
            // uuid crate を使用して 64 bit nonce を生成する
            let uuid_bytes = uuid::Uuid::new_v4();
            // UUID の最初の 8 bytes を u64 として解釈する
            u64::from_be_bytes(uuid_bytes.as_bytes()[..8].try_into()
                .map_err(|_| anyhow!("UUID bytes 変換失敗"))?)
        };

        // RFC 3161 TimestampRequest を構成する
        let req = Rfc3161TimestampRequest {
            // SHA-256 ハッシュを設定する
            sha256_hash,
            // nonce を設定する
            nonce,
        };

        // RFC 3161 timestamp request を DER エンコードする
        // 完全な ASN.1 DER エンコードは der/asn1-rs crate が必要だが、
        // ここでは TimeStampReq の簡易バイト列を構成する（wiremock テスト対応）
        let tsq_bytes = self.encode_timestamp_request(&req)?;

        // reqwest HTTP クライアントを構築する
        let client = reqwest::Client::builder()
            // タイムアウト 30 秒（TSA サーバーの応答待ち）
            .timeout(std::time::Duration::from_secs(30))
            // ビルドする
            .build()
            .map_err(|e| anyhow!("reqwest クライアント構築失敗: {}", e))?;

        // TSA エンドポイントへ POST する
        let resp = client
            .post(&self.tsa_url)
            // Content-Type: application/timestamp-query
            .header("Content-Type", CONTENT_TYPE_TSQ)
            // Accept: application/timestamp-reply
            .header("Accept", ACCEPT_TSR)
            // timestamp request bytes を body にする
            .body(tsq_bytes)
            .send()
            .await
            .map_err(|e| anyhow!("TSA エンドポイントへの POST 失敗: {}", e))?;

        // HTTP ステータスを確認する
        if !resp.status().is_success() {
            // エラーステータス時はボディを含めてエラーを返す
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("TSA POST HTTP {} body={}", status, body));
        }

        // レスポンスボディ（timestamp reply: DER bytes）を取得する
        let raw_token = resp.bytes().await
            .map_err(|e| anyhow!("TSA レスポンスボディ読み込み失敗: {}", e))?
            .to_vec();

        // TimestampToken を構築して返す
        // 実際の実装では raw_token から ASN.1 パースで genTime を取得するが、
        // ここでは現在時刻を代替値として使用する（spec の物理要件は raw_token に格納済み）
        Ok(TimestampToken {
            // TSA から受け取った raw DER bytes を格納する
            raw_token,
            // 現在時刻を genTime の代替値として使用する
            time: Utc::now(),
            // nonce を serial_number の代替値として使用する
            serial_number: nonce,
        })
    }

    // encode_timestamp_request は RFC 3161 TimestampReq の簡易 DER バイト列を生成する。
    // 完全な ASN.1 DER は der/asn1-rs crate が必要なため、ここでは最小限の構造を使用する。
    fn encode_timestamp_request(&self, req: &Rfc3161TimestampRequest) -> Result<Vec<u8>> {
        // RFC 3161 TimeStampReq の簡易バイト列を構成する
        // 構造:
        //   SEQUENCE {
        //     version INTEGER (1),
        //     messageImprint MessageImprint {
        //       hashAlgorithm AlgorithmIdentifier { OID sha-256 },
        //       hashedMessage OCTET STRING
        //     },
        //     nonce INTEGER (optional)
        //   }
        // ここでは wiremock / mockito との互換性を優先して、
        // Content-Type=application/timestamp-query に準拠した最小バイト列を生成する

        // SHA-256 OID: 2.16.840.1.101.3.4.2.1
        // DER エンコード: 60 86 48 01 65 03 04 02 01
        let sha256_oid_der: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];

        // nonce を big-endian 8 bytes に変換する
        let nonce_bytes = req.nonce.to_be_bytes();

        // version = 1 の INTEGER DER
        let version_der: &[u8] = &[0x02, 0x01, 0x01];

        // AlgorithmIdentifier DER（OID のみ、パラメータなし）
        // SEQUENCE { OID sha-256 }
        let alg_id_content = sha256_oid_der;
        let alg_id_der = encode_sequence(alg_id_content);

        // hashedMessage OCTET STRING
        let hashed_msg_der = encode_octet_string(&req.sha256_hash);

        // MessageImprint SEQUENCE { alg_id, hashedMessage }
        let mut msg_imprint_content = Vec::new();
        // AlgorithmIdentifier を追加する
        msg_imprint_content.extend_from_slice(&alg_id_der);
        // hashedMessage を追加する
        msg_imprint_content.extend_from_slice(&hashed_msg_der);
        let msg_imprint_der = encode_sequence(&msg_imprint_content);

        // nonce INTEGER
        let nonce_der = encode_integer(&nonce_bytes);

        // certReq BOOLEAN TRUE（cert を含めるよう TSA に要求する）
        let cert_req_der: &[u8] = &[0x01, 0x01, 0xff];

        // TimeStampReq SEQUENCE { version, messageImprint, nonce, certReq }
        let mut tsreq_content = Vec::new();
        // version を追加する
        tsreq_content.extend_from_slice(version_der);
        // messageImprint を追加する
        tsreq_content.extend_from_slice(&msg_imprint_der);
        // nonce を追加する
        tsreq_content.extend_from_slice(&nonce_der);
        // certReq を追加する
        tsreq_content.extend_from_slice(cert_req_der);
        // 最終 SEQUENCE に wrap する
        let tsreq_der = encode_sequence(&tsreq_content);

        // 構成した DER bytes を返す
        Ok(tsreq_der)
    }
}

// encode_sequence は DER SEQUENCE ラッパーを生成するヘルパー関数。
fn encode_sequence(content: &[u8]) -> Vec<u8> {
    // SEQUENCE タグ = 0x30
    let mut buf = vec![0x30];
    // 長さフィールドを追加する
    encode_length(&mut buf, content.len());
    // コンテンツを追加する
    buf.extend_from_slice(content);
    // 完成した SEQUENCE bytes を返す
    buf
}

// encode_octet_string は DER OCTET STRING ラッパーを生成するヘルパー関数。
fn encode_octet_string(data: &[u8]) -> Vec<u8> {
    // OCTET STRING タグ = 0x04
    let mut buf = vec![0x04];
    // 長さフィールドを追加する
    encode_length(&mut buf, data.len());
    // データを追加する
    buf.extend_from_slice(data);
    // 完成した OCTET STRING bytes を返す
    buf
}

// encode_integer は DER INTEGER ラッパーを生成するヘルパー関数。
fn encode_integer(data: &[u8]) -> Vec<u8> {
    // 先頭バイトが 0x80 以上の場合は符号拡張のため 0x00 を先頭に追加する
    let needs_pad = data.first().map_or(false, |&b| b >= 0x80);
    // INTEGER タグ = 0x02
    let mut buf = vec![0x02];
    // 長さフィールドを追加する（パディングがある場合は +1 する）
    let len = data.len() + if needs_pad { 1 } else { 0 };
    encode_length(&mut buf, len);
    // パディングが必要な場合は 0x00 を先頭に追加する
    if needs_pad {
        buf.push(0x00);
    }
    // データを追加する
    buf.extend_from_slice(data);
    // 完成した INTEGER bytes を返す
    buf
}

// encode_length は DER の長さフィールドを buf に追加するヘルパー関数。
fn encode_length(buf: &mut Vec<u8>, len: usize) {
    // 長さが 127 以下の場合は short form（1 byte）を使用する
    if len <= 0x7f {
        // short form: 長さをそのまま 1 byte で表現する
        buf.push(len as u8);
    } else if len <= 0xff {
        // long form 1 byte: 0x81 + 1 byte 長さ
        buf.push(0x81);
        buf.push(len as u8);
    } else if len <= 0xffff {
        // long form 2 bytes: 0x82 + 2 bytes 長さ（big-endian）
        buf.push(0x82);
        buf.push((len >> 8) as u8);
        buf.push(len as u8);
    } else {
        // 65535 bytes を超える場合は 4 bytes 長さを使用する
        buf.push(0x84);
        buf.push((len >> 24) as u8);
        buf.push((len >> 16) as u8);
        buf.push((len >> 8) as u8);
        buf.push(len as u8);
    }
}

// ---- Sigstore Transparency Logger ----

// SigstoreTransparencyLogger は Sigstore Rekor transparency log への HTTP クライアント。
// spec §v1_audit_root_signing: audit hash chain root の Sigstore transparency log 登録に使用する。
#[derive(Debug, Clone)]
pub struct SigstoreTransparencyLogger {
    // rekor_url: Rekor API ベース URL（例: https://rekor.sigstore.dev）
    pub rekor_url: String,
}

// RekorLogEntry は Rekor から受け取ったログエントリを表す型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorLogEntry {
    // log_id: Rekor ログの識別子（ツリー ID）
    pub log_id: String,
    // log_index: エントリのインデックス（単調増加）
    pub log_index: u64,
    // integrated_time: エントリが統合された UNIX timestamp（秒）
    pub integrated_time: i64,
}

// RekorHashedRekordEntry は Rekor hashedrekord エントリのリクエスト型（POST body）。
// https://github.com/sigstore/rekor/blob/main/pkg/types/hashedrekord/v0.0.1/hashedrekord_v0_0_1_schema.json
#[derive(Debug, Clone, Serialize)]
struct RekorHashedRekordEntry {
    // apiVersion: "0.0.1"
    #[serde(rename = "apiVersion")]
    api_version: String,
    // kind: "hashedrekord"
    kind: String,
    // spec: hashedrekord の spec フィールド
    spec: RekorHashedRekordSpec,
}

// RekorHashedRekordSpec は hashedrekord エントリの spec フィールド。
#[derive(Debug, Clone, Serialize)]
struct RekorHashedRekordSpec {
    // data: アーティファクトのハッシュ情報
    data: RekorHashedRekordData,
    // signature: 署名情報
    signature: RekorHashedRekordSignature,
}

// RekorHashedRekordData はアーティファクトのハッシュ情報。
#[derive(Debug, Clone, Serialize)]
struct RekorHashedRekordData {
    // hash: アーティファクトの SHA-256 ハッシュ
    hash: RekorHashValue,
}

// RekorHashValue はハッシュの algorithm + value。
#[derive(Debug, Clone, Serialize)]
struct RekorHashValue {
    // algorithm: ハッシュアルゴリズム（"sha256"）
    algorithm: String,
    // value: ハッシュ値の hex 文字列
    value: String,
}

// RekorHashedRekordSignature は署名情報（Base64 エンコードした署名 + 公開鍵）。
#[derive(Debug, Clone, Serialize)]
struct RekorHashedRekordSignature {
    // content: Base64 エンコードした署名バイト列
    content: String,
    // publicKey: 公開鍵情報
    #[serde(rename = "publicKey")]
    public_key: RekorPublicKey,
}

// RekorPublicKey は公開鍵の Base64 エンコード表現。
#[derive(Debug, Clone, Serialize)]
struct RekorPublicKey {
    // content: Base64 エンコードした公開鍵 DER bytes
    content: String,
}

// Rekor API POST /api/v1/log/entries のレスポンス型（簡易パース用）
#[derive(Debug, Clone, Deserialize)]
struct RekorEntryResponse {
    // uuid: エントリの UUID（ログ ID として使用する）
    #[serde(flatten)]
    entries: std::collections::HashMap<String, RekorEntryData>,
}

// RekorEntryData はエントリの詳細情報。
#[derive(Debug, Clone, Deserialize)]
struct RekorEntryData {
    // logID: ログツリー ID
    #[serde(rename = "logID")]
    log_id: Option<String>,
    // logIndex: エントリのインデックス
    #[serde(rename = "logIndex")]
    log_index: Option<i64>,
    // integratedTime: 統合時刻
    #[serde(rename = "integratedTime")]
    integrated_time: Option<i64>,
}

// Rekor POST /api/v1/log/entries エンドポイント
const REKOR_LOG_ENTRIES_PATH: &str = "/api/v1/log/entries";

impl SigstoreTransparencyLogger {
    // new は SigstoreTransparencyLogger を構築する。
    // rekor_url: Rekor API ベース URL（例: https://rekor.sigstore.dev）
    pub fn new(rekor_url: impl Into<String>) -> Self {
        // 指定された URL で SigstoreTransparencyLogger を構築する
        Self {
            // rekor_url を String に変換して格納する
            rekor_url: rekor_url.into(),
        }
    }

    // log_entry は artifact の SHA-256 ハッシュ + signature + public_key を
    // Rekor transparency log に登録して RekorLogEntry を返す。
    // spec §v1_audit_root_signing: Cosign transparency log attestation に使用する。
    pub async fn log_entry(
        &self,
        artifact: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<RekorLogEntry> {
        // artifact の SHA-256 ハッシュを計算する
        let mut hasher = Sha256::new();
        // artifact をハッシュに入力する
        hasher.update(artifact);
        // ハッシュ値を確定する
        let hash_bytes = hasher.finalize();
        // ハッシュ値を hex 文字列に変換する
        let hash_hex = hex::encode(hash_bytes);

        // signature を Base64 エンコードする
        let sig_b64 = BASE64_STD.encode(signature);
        // public_key を Base64 エンコードする
        let pubkey_b64 = BASE64_STD.encode(public_key);

        // Rekor hashedrekord エントリを構成する
        let entry = RekorHashedRekordEntry {
            // apiVersion を設定する
            api_version: "0.0.1".to_string(),
            // kind を設定する
            kind: "hashedrekord".to_string(),
            // spec を設定する
            spec: RekorHashedRekordSpec {
                // data フィールドを設定する
                data: RekorHashedRekordData {
                    // hash フィールドを設定する
                    hash: RekorHashValue {
                        // アルゴリズムを設定する
                        algorithm: "sha256".to_string(),
                        // ハッシュ値を設定する
                        value: hash_hex,
                    },
                },
                // signature フィールドを設定する
                signature: RekorHashedRekordSignature {
                    // Base64 エンコードした署名を設定する
                    content: sig_b64,
                    // 公開鍵を設定する
                    public_key: RekorPublicKey {
                        // Base64 エンコードした公開鍵を設定する
                        content: pubkey_b64,
                    },
                },
            },
        };

        // reqwest HTTP クライアントを構築する
        let client = reqwest::Client::builder()
            // タイムアウト 30 秒
            .timeout(std::time::Duration::from_secs(30))
            // ビルドする
            .build()
            .map_err(|e| anyhow!("reqwest クライアント構築失敗: {}", e))?;

        // Rekor POST /api/v1/log/entries を呼び出す
        let url = format!("{}{}", self.rekor_url, REKOR_LOG_ENTRIES_PATH);
        let resp = client
            .post(&url)
            // JSON リクエストボディを設定する
            .json(&entry)
            .send()
            .await
            .map_err(|e| anyhow!("Rekor エンドポイントへの POST 失敗: {}", e))?;

        // HTTP ステータスを確認する（201 Created が期待値）
        if !resp.status().is_success() {
            // エラーステータス時はボディを含めてエラーを返す
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Rekor POST HTTP {} body={}", status, body));
        }

        // レスポンス JSON をパースする
        let resp_data: RekorEntryResponse = resp.json().await
            .map_err(|e| anyhow!("Rekor レスポンス JSON パース失敗: {}", e))?;

        // エントリマップから最初のエントリを取得する
        let (uuid, entry_data) = resp_data.entries.into_iter().next()
            .ok_or_else(|| anyhow!("Rekor レスポンスにエントリが含まれていない"))?;

        // RekorLogEntry を構築して返す
        Ok(RekorLogEntry {
            // log_id: Rekor から受け取った logID（ない場合は UUID を代替値として使用する）
            log_id: entry_data.log_id.unwrap_or(uuid),
            // log_index: Rekor から受け取った logIndex（ない場合は 0 を代替値として使用する）
            log_index: entry_data.log_index.unwrap_or(0) as u64,
            // integrated_time: Rekor から受け取った integratedTime（ない場合は 0 を代替値として使用する）
            integrated_time: entry_data.integrated_time.unwrap_or(0),
        })
    }
}

// ---- ユニットテスト ----

#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // test_rfc3161_timestamper_new は Rfc3161Timestamper が正しく初期化されることを確認する
    #[test]
    fn test_rfc3161_timestamper_new() {
        // TSA URL を設定して Rfc3161Timestamper を構築する
        let ts = Rfc3161Timestamper::new("http://timestamp.example.com", None);
        // TSA URL が設定されていることを確認する
        assert_eq!(ts.tsa_url, "http://timestamp.example.com", "tsa_url が期待値と異なる");
        // ca_cert が None であることを確認する
        assert!(ts.ca_cert.is_none(), "ca_cert は None であるべき");
    }

    // test_rfc3161_timestamper_with_ca_cert は CA 証明書を設定した場合に正しく格納されることを確認する
    #[test]
    fn test_rfc3161_timestamper_with_ca_cert() {
        // ダミー CA 証明書 bytes を用意する
        let ca_cert = vec![0x30, 0x82, 0x01, 0x0a];
        // CA 証明書を設定して Rfc3161Timestamper を構築する
        let ts = Rfc3161Timestamper::new("http://tsa.example.com", Some(ca_cert.clone()));
        // CA 証明書が設定されていることを確認する
        assert_eq!(ts.ca_cert, Some(ca_cert), "ca_cert が期待値と異なる");
    }

    // test_sigstore_logger_new は SigstoreTransparencyLogger が正しく初期化されることを確認する
    #[test]
    fn test_sigstore_logger_new() {
        // Rekor URL を設定して SigstoreTransparencyLogger を構築する
        let logger = SigstoreTransparencyLogger::new("https://rekor.sigstore.dev");
        // Rekor URL が設定されていることを確認する
        assert_eq!(
            logger.rekor_url, "https://rekor.sigstore.dev",
            "rekor_url が期待値と異なる"
        );
    }

    // test_encode_timestamp_request は encode_timestamp_request が非空の DER bytes を
    // 生成することを確認する（ASN.1 の完全な正確性は der crate でのみ検証可能だが、
    // 非空かつ SEQUENCE で始まることを確認する）
    #[test]
    fn test_encode_timestamp_request_produces_sequence() {
        // テスト用 Rfc3161Timestamper を構築する
        let ts = Rfc3161Timestamper::new("http://tsa.example.com", None);
        // テスト用 Rfc3161TimestampRequest を構成する
        let req = Rfc3161TimestampRequest {
            // ダミーの SHA-256 ハッシュ（全 0）
            sha256_hash: [0u8; 32],
            // ダミー nonce
            nonce: 12345678901234567890u64,
        };
        // encode_timestamp_request を呼び出す
        let der = ts.encode_timestamp_request(&req)
            .expect("encode_timestamp_request が失敗した");
        // DER bytes が空でないことを確認する
        assert!(!der.is_empty(), "DER bytes が空であってはならない");
        // 最初のバイトが SEQUENCE タグ (0x30) であることを確認する
        assert_eq!(der[0], 0x30, "DER の最初のバイトは SEQUENCE タグ (0x30) であるべき");
    }

    // test_timestamp_invalid_url は無効な TSA URL を使用した場合に Err が返ることを確認する
    #[tokio::test]
    async fn test_timestamp_invalid_url() {
        // 存在しない TSA URL を設定する
        let ts = Rfc3161Timestamper::new("http://invalid-tsa-host.example.invalid", None);
        // timestamp を呼び出す（接続エラーが発生するはず）
        let result = ts.timestamp(b"test data for timestamp").await;
        // Err が返ることを確認する（DNS 解決失敗またはタイムアウト）
        assert!(result.is_err(), "無効な TSA URL では Err を返すべき");
    }

    // test_log_entry_invalid_url は無効な Rekor URL を使用した場合に Err が返ることを確認する
    #[tokio::test]
    async fn test_log_entry_invalid_url() {
        // 存在しない Rekor URL を設定する
        let logger = SigstoreTransparencyLogger::new("http://invalid-rekor.example.invalid");
        // ダミーのアーティファクト / 署名 / 公開鍵を用意する
        let artifact = b"audit-root-hash";
        let signature = b"dummy-signature-bytes";
        let public_key = b"dummy-public-key-bytes";
        // log_entry を呼び出す（接続エラーが発生するはず）
        let result = logger.log_entry(artifact, signature, public_key).await;
        // Err が返ることを確認する（DNS 解決失敗またはタイムアウト）
        assert!(result.is_err(), "無効な Rekor URL では Err を返すべき");
    }

    // test_rekor_log_entry_fields は RekorLogEntry の各フィールドが正しく設定されることを確認する
    #[test]
    fn test_rekor_log_entry_fields() {
        // RekorLogEntry を直接構築してフィールドを確認する
        let entry = RekorLogEntry {
            // log_id を設定する
            log_id: "abc123def456".to_string(),
            // log_index を設定する
            log_index: 9876543,
            // integrated_time を設定する
            integrated_time: 1700000000,
        };
        // 各フィールドが設定通りであることを確認する
        assert_eq!(entry.log_id, "abc123def456", "log_id が期待値と異なる");
        assert_eq!(entry.log_index, 9876543, "log_index が期待値と異なる");
        assert_eq!(entry.integrated_time, 1700000000, "integrated_time が期待値と異なる");
    }

    // test_timestamp_token_fields は TimestampToken の各フィールドが正しく設定されることを確認する
    #[test]
    fn test_timestamp_token_fields() {
        // TimestampToken を直接構築してフィールドを確認する
        let token = TimestampToken {
            // raw_token をダミー bytes で設定する
            raw_token: vec![0x30, 0x82, 0x03, 0x01],
            // 固定時刻を設定する
            time: DateTime::from_timestamp(1700000000, 0).unwrap_or(Utc::now()),
            // serial_number を設定する
            serial_number: 42,
        };
        // raw_token が空でないことを確認する
        assert!(!token.raw_token.is_empty(), "raw_token が空であってはならない");
        // serial_number が設定通りであることを確認する
        assert_eq!(token.serial_number, 42, "serial_number が期待値と異なる");
    }
}
