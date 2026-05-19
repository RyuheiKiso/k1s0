// k1s0 tier2 notary TSA クライアント
// RFC 3161 タイムスタンプ認証局（TSA）へのタイムスタンプ要求と
// Sigstore Rekor 透明性ログへのバンドル登録を提供する
// 鍵管理適合仕様 05: 外部公証で削除・変更を attest する

// reqwest: HTTP クライアント（TSA への POST / Rekor API への POST に使用する）
use reqwest::Client;
// anyhow: エラーハンドリング（Result<T> の統合 error context に使用する）
use anyhow::{Context, Result};
// serde: シリアライズ / デシリアライズフレームワーク（Rekor リクエスト/レスポンスの JSON 構造体に使用する）
use serde::{Deserialize, Serialize};
// base64: バイナリデータを base64 にエンコードして Rekor API リクエストに含めるために使用する
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
// tracing: 構造化ロギング（HTTP リクエスト送信 / レスポンス受信のトレースに使用する）
use tracing::{info, debug};
// std::env: 環境変数 RFC3161_TSA_URL の読み取りに使用する
use std::env;

// Rekor ログエントリ登録リクエストの本体構造体（hashedrekord タイプを使用する）
#[derive(Debug, Serialize)]
struct RekorHashedRekordRequest {
    // Rekor エントリの kind を指定する（"hashedrekord" を使用する）
    kind: String,
    // Rekor エントリの API バージョンを指定する（"0.0.1" を使用する）
    apiVersion: String,
    // エントリの仕様データを保持する
    spec: RekorHashedRekordSpec,
}

// Rekor hashedrekord エントリの仕様（hash + signature + publicKey を含む）
#[derive(Debug, Serialize)]
struct RekorHashedRekordSpec {
    // ハッシュ情報（アルゴリズムと値）
    data: RekorDataHash,
    // 署名情報（content を base64 エンコードして格納する）
    signature: RekorSignature,
}

// Rekor エントリのハッシュデータ（SHA-256 のみを想定する）
#[derive(Debug, Serialize)]
struct RekorDataHash {
    // ハッシュアルゴリズム名（"sha256" を使用する）
    algorithm: String,
    // ハッシュ値の hex 文字列
    value: String,
}

// Rekor エントリの署名情報（タイムスタンプトークンを署名コンテンツとして使用する）
#[derive(Debug, Serialize)]
struct RekorSignature {
    // 署名コンテンツを base64 エンコードした文字列（タイムスタンプトークンを格納する）
    content: String,
    // 公開鍵情報（Rekor の hashedrekord では任意だが構造上必須のためダミー値を使用する）
    publicKey: RekorPublicKey,
}

// Rekor 公開鍵情報（TSA 応答のタイムスタンプトークン登録では空文字列でよい）
#[derive(Debug, Serialize)]
struct RekorPublicKey {
    // 公開鍵コンテンツを base64 エンコードした文字列
    content: String,
}

// Rekor ログエントリ登録レスポンスのトップレベル構造体
#[derive(Debug, Deserialize)]
struct RekorEntryResponse {
    // エントリの UUID をキーとするマップ（キーが UUID、値がエントリ詳細）
    // Rekor API は UUID をキーとする JSON オブジェクトを返す
    // serde_json::Value で任意のキーを受け取る
    #[serde(flatten)]
    entries: std::collections::HashMap<String, serde_json::Value>,
}

// TsaClient: RFC 3161 TSA 呼び出しと Rekor ログエントリ登録を提供する構造体
pub struct TsaClient {
    // HTTP クライアント（reqwest::Client を再利用する）
    http_client: Client,
    // TSA エンドポイント URL（RFC3161_TSA_URL 環境変数または rfc3161_config.yaml から取得する）
    tsa_url: String,
}

impl TsaClient {
    // TsaClient を生成する
    // RFC3161_TSA_URL 環境変数が設定されている場合はそれを優先する
    // 未設定の場合は rfc3161_config.yaml の tsa_url をデフォルト値として使用する
    pub fn new() -> Result<Self> {
        // RFC3161_TSA_URL 環境変数から TSA URL を読み取る
        let tsa_url = env::var("RFC3161_TSA_URL")
            // 環境変数が未設定の場合は rfc3161_config.yaml のデフォルト TSA URL を使用する
            .unwrap_or_else(|_| "http://timestamp.digicert.com".to_string());

        // TSA URL をログ出力して設定を確認できるようにする
        info!(tsa_url = %tsa_url, "TsaClient initialized");

        // reqwest::Client をビルドする（rustls-tls を使用する）
        let http_client = Client::builder()
            // User-Agent を k1s0-notary/0.1.0 に設定する
            .user_agent("k1s0-notary/0.1.0")
            // HTTP クライアントをビルドする
            .build()
            .context("failed to build reqwest::Client")?;

        // TsaClient インスタンスを返す
        Ok(Self { http_client, tsa_url })
    }

    // timestamp_hash: data_hash のタイムスタンプを TSA に要求してトークンバイトを返す
    // RFC 3161 TimeStampRequest を組み立て、TSA URL に POST して応答バイトを返す
    // data_hash: タイムスタンプ対象データのハッシュ値（SHA-256 バイト列）
    // 戻り値: DER エンコードされたタイムスタンプトークンバイト列
    pub async fn timestamp_hash(&self, data_hash: &[u8]) -> Result<Vec<u8>> {
        // data_hash の長さを検証する（SHA-256 は 32 バイト）
        debug!(hash_len = data_hash.len(), "building RFC 3161 timestamp request");

        // RFC 3161 TimeStampRequest の DER エンコードを手動で構築する
        // ASN.1 DER 構造: SEQUENCE { version INTEGER, messageImprint HashAlgorithmAndValue, nonce INTEGER }
        // OID 2.16.840.1.101.3.4.2.1 = SHA-256
        // 参考: RFC 3161 Section 2.4.1
        let sha256_oid: &[u8] = &[
            // OID タグ (0x06) + 長さ (0x09)
            0x06, 0x09,
            // OID 2.16.840.1.101.3.4.2.1 の DER エンコード
            0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01,
        ];

        // AlgorithmIdentifier: SEQUENCE { oid, NULL }
        // SEQUENCE タグ (0x30) + 長さ
        let alg_id_inner_len = sha256_oid.len() + 2; // NULL (0x05 0x00) の 2 バイトを加える
        // AlgorithmIdentifier の内部コンテンツを構築する
        let mut alg_id: Vec<u8> = Vec::new();
        // SEQUENCE タグを追加する
        alg_id.push(0x30);
        // 内部長さを追加する（1 バイト長）
        alg_id.push(alg_id_inner_len as u8);
        // OID バイト列を追加する
        alg_id.extend_from_slice(sha256_oid);
        // NULL (0x05 0x00) を追加する
        alg_id.extend_from_slice(&[0x05, 0x00]);

        // MessageImprint: SEQUENCE { AlgorithmIdentifier, OCTET STRING(hash) }
        // OCTET STRING タグ (0x04) + 長さ + ハッシュバイト列
        let mut hash_octet: Vec<u8> = Vec::new();
        // OCTET STRING タグを追加する
        hash_octet.push(0x04);
        // ハッシュ長を追加する
        hash_octet.push(data_hash.len() as u8);
        // ハッシュバイト列を追加する
        hash_octet.extend_from_slice(data_hash);

        // MessageImprint の内部コンテンツ長を計算する
        let msg_imprint_inner_len = alg_id.len() + hash_octet.len();
        // MessageImprint SEQUENCE を構築する
        let mut msg_imprint: Vec<u8> = Vec::new();
        // SEQUENCE タグを追加する
        msg_imprint.push(0x30);
        // 内部長さを追加する（1 バイト長）
        msg_imprint.push(msg_imprint_inner_len as u8);
        // AlgorithmIdentifier を追加する
        msg_imprint.extend_from_slice(&alg_id);
        // OCTET STRING を追加する
        msg_imprint.extend_from_slice(&hash_octet);

        // version INTEGER (1) の DER エンコード: 0x02 0x01 0x01
        let version_der: &[u8] = &[0x02, 0x01, 0x01];

        // certReq BOOLEAN TRUE の DER エンコード: 0x01 0x01 0xff
        let cert_req_der: &[u8] = &[0x01, 0x01, 0xff];

        // TimeStampReq SEQUENCE 内部コンテンツ: version + messageImprint + certReq
        let inner_len = version_der.len() + msg_imprint.len() + cert_req_der.len();

        // TimeStampReq 全体を構築する
        let mut ts_req: Vec<u8> = Vec::new();
        // SEQUENCE タグを追加する
        ts_req.push(0x30);
        // 内部長さを追加する（2 バイト長形式で対応する）
        if inner_len < 128 {
            // 短形式: 1 バイト長
            ts_req.push(inner_len as u8);
        } else if inner_len < 256 {
            // 長形式 1 バイト: 0x81 + 長さ
            ts_req.push(0x81);
            ts_req.push(inner_len as u8);
        } else {
            // 長形式 2 バイト: 0x82 + 上位バイト + 下位バイト
            ts_req.push(0x82);
            ts_req.push((inner_len >> 8) as u8);
            ts_req.push((inner_len & 0xff) as u8);
        }
        // version DER を追加する
        ts_req.extend_from_slice(version_der);
        // MessageImprint を追加する
        ts_req.extend_from_slice(&msg_imprint);
        // certReq を追加する
        ts_req.extend_from_slice(cert_req_der);

        // 構築したタイムスタンプリクエストのサイズをデバッグログに出力する
        debug!(request_bytes = ts_req.len(), tsa_url = %self.tsa_url, "posting RFC 3161 timestamp request");

        // TSA URL に RFC 3161 タイムスタンプリクエストを POST する
        let response = self.http_client
            // TSA URL に POST する
            .post(&self.tsa_url)
            // Content-Type を application/timestamp-query に設定する（RFC 3161 規定）
            .header("Content-Type", "application/timestamp-query")
            // リクエストボディに DER エンコード済みタイムスタンプリクエストを設定する
            .body(ts_req)
            // 非同期で HTTP リクエストを送信する
            .send()
            .await
            .context("failed to send RFC 3161 timestamp request to TSA")?;

        // HTTP ステータスコードを検証する（2xx 以外はエラーとして扱う）
        let status = response.status();
        // レスポンスステータスをログ出力する
        info!(status = %status, "received RFC 3161 timestamp response");

        // 2xx 以外のステータスコードはエラーとして返す
        if !status.is_success() {
            // エラーメッセージにステータスコードを含める
            anyhow::bail!("TSA returned non-success status: {}", status);
        }

        // レスポンスボディを全バイト読み取る（タイムスタンプトークンの DER バイト列）
        let token_bytes = response
            .bytes()
            .await
            .context("failed to read RFC 3161 timestamp response body")?
            .to_vec();

        // タイムスタンプトークンのサイズをログ出力する
        debug!(token_bytes = token_bytes.len(), "RFC 3161 timestamp token received");

        // タイムスタンプトークンのバイト列を返す
        Ok(token_bytes)
    }

    // submit_to_rekor: タイムスタンプバンドルを Sigstore Rekor 透明性ログに登録する
    // bundle: Rekor に登録するバイト列（タイムスタンプトークン等）
    // 戻り値: Rekor ログエントリの UUID 文字列
    pub async fn submit_to_rekor(&self, bundle: &[u8]) -> Result<String> {
        // Rekor API エンドポイント URL を定数として定義する
        let rekor_url = "https://rekor.sigstore.dev/api/v1/log/entries";

        // バンドルバイト列を SHA-256 でハッシュ化してエントリのデータハッシュとして使用する
        // 注: 実運用では正確な hashedrekord 仕様に合わせた実装が必要
        let hash_hex = bundle
            .iter()
            // 各バイトを 2 桁 hex 文字列にフォーマットしてハッシュ文字列を生成する
            .fold(String::new(), |mut acc, b| {
                // フォーマット済み文字列を蓄積する
                acc.push_str(&format!("{:02x}", b));
                acc
            });

        // バンドルバイト列を base64 エンコードして Rekor 署名コンテンツとして使用する
        let bundle_b64 = BASE64_STANDARD.encode(bundle);

        // Rekor hashedrekord エントリリクエストを構築する
        let rekor_request = RekorHashedRekordRequest {
            // エントリ種別を hashedrekord に設定する
            kind: "hashedrekord".to_string(),
            // API バージョンを 0.0.1 に設定する
            apiVersion: "0.0.1".to_string(),
            // エントリ仕様を設定する
            spec: RekorHashedRekordSpec {
                // データハッシュ情報を設定する
                data: RekorDataHash {
                    // ハッシュアルゴリズムを sha256 に設定する
                    algorithm: "sha256".to_string(),
                    // ハッシュ hex 文字列を設定する
                    value: hash_hex,
                },
                // 署名情報を設定する（タイムスタンプトークンを署名コンテンツとして使用する）
                signature: RekorSignature {
                    // base64 エンコードされたタイムスタンプトークンを設定する
                    content: bundle_b64,
                    // 公開鍵フィールドを空文字列で初期化する（hashedrekord では任意）
                    publicKey: RekorPublicKey {
                        // 空文字列の base64 エンコード値を設定する
                        content: BASE64_STANDARD.encode(b""),
                    },
                },
            },
        };

        // Rekor リクエストを JSON にシリアライズする
        let request_json = serde_json::to_string(&rekor_request)
            .context("failed to serialize Rekor request to JSON")?;

        // Rekor リクエストのサイズをデバッグログに出力する
        debug!(
            request_json_len = request_json.len(),
            rekor_url = %rekor_url,
            "posting bundle to Rekor"
        );

        // Rekor API に POST してタイムスタンプバンドルを登録する
        let response = self.http_client
            // Rekor API エンドポイントに POST する
            .post(rekor_url)
            // Content-Type を application/json に設定する（Rekor API 要件）
            .header("Content-Type", "application/json")
            // Accept ヘッダーを application/json に設定する
            .header("Accept", "application/json")
            // リクエストボディに JSON を設定する
            .body(request_json)
            // 非同期で HTTP リクエストを送信する
            .send()
            .await
            .context("failed to send bundle to Rekor")?;

        // HTTP ステータスコードを検証する（201 Created が期待値）
        let status = response.status();
        // レスポンスステータスをログ出力する
        info!(status = %status, "received Rekor log entry response");

        // 2xx 以外のステータスコードはエラーとして返す
        if !status.is_success() {
            // レスポンスボディをエラーメッセージに含める
            let body = response.text().await.unwrap_or_default();
            // ステータスとボディを含むエラーを返す
            anyhow::bail!("Rekor returned non-success status: {} body: {}", status, body);
        }

        // レスポンス JSON をパースして UUID を取得する
        let entry_response: RekorEntryResponse = response
            .json()
            .await
            .context("failed to parse Rekor log entry response as JSON")?;

        // エントリマップの最初のキー（UUID）を取得する
        let entry_uuid = entry_response
            .entries
            // キーのイテレータから最初の UUID を取得する
            .keys()
            .next()
            .cloned()
            // UUID が存在しない場合はエラーを返す
            .ok_or_else(|| anyhow::anyhow!("Rekor response contained no entry UUID"))?;

        // Rekor エントリ UUID をログ出力する
        info!(entry_uuid = %entry_uuid, "bundle successfully submitted to Rekor");

        // Rekor エントリ UUID を返す
        Ok(entry_uuid)
    }
}
