// storage.rs — k1s0 tier1 Library backend: Object Storage L3 trait
// backend 専用カテゴリ（frontend には提供しない）。
// 公開 API に OSS 型（aws_sdk_s3::Client / opendal::Operator 等）を露出しない。
// 全 trait は Send + Sync を要求する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: オブジェクトメタデータのシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: ストレージ操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;

// ObjectMetadata はストレージオブジェクトのメタデータを表す Library 独自型。
// OSS 固有のメタデータ型（S3 HeadObjectOutput 等）は使わない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    // bucket: オブジェクトが属するバケット名（テナント ID が prefix として埋め込まれる）
    pub bucket: String,
    // key: オブジェクトキー（パス形式; テナント境界内のパス）
    pub key: String,
    // size_bytes: オブジェクトのバイトサイズ
    pub size_bytes: u64,
    // content_type: MIME タイプ（例: "application/octet-stream"）
    pub content_type: String,
    // etag: オブジェクトの ETag（一意性識別子）
    pub etag: String,
    // tenant_id: このオブジェクトが属するテナントの識別子
    pub tenant_id: String,
    // custom_metadata: ユーザー定義メタデータ（key-value ペア）
    pub custom_metadata: std::collections::HashMap<String, String>,
}

// UploadOptions はオブジェクトアップロード時のオプションを表す struct。
#[derive(Debug, Clone)]
pub struct UploadOptions {
    // content_type: MIME タイプ（省略時は "application/octet-stream"）
    pub content_type: String,
    // custom_metadata: ユーザー定義メタデータ
    pub custom_metadata: std::collections::HashMap<String, String>,
    // server_side_encryption: サーバー側暗号化を要求するかどうか
    pub server_side_encryption: bool,
}

// UploadOptions のデフォルト値
impl Default for UploadOptions {
    fn default() -> Self {
        // 汎用的なデフォルト設定
        Self {
            // content_type: application/octet-stream（汎用バイナリ）
            content_type: "application/octet-stream".to_string(),
            // custom_metadata: 空のメタデータマップ
            custom_metadata: std::collections::HashMap::new(),
            // server_side_encryption: true（暗号化を要求する）
            server_side_encryption: true,
        }
    }
}

// ListResult はオブジェクト一覧取得の結果を表す struct。
#[derive(Debug, Clone)]
pub struct ListResult {
    // objects: メタデータのリスト（ページ単位）
    pub objects: Vec<ObjectMetadata>,
    // next_continuation_token: 次ページのトークン（None は最終ページ）
    pub next_continuation_token: Option<String>,
}

// ObjectStorage は Object Storage の L3 抽象 trait。
// MinIO / S3 / GCS 等の OSS/サービスを実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    // upload はオブジェクトを書き込む（マルチパートは実装が透過的に処理する）。
    // auth_ctx はテナント境界の保証と audit ログに使用する。
    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        options: UploadOptions,
        auth_ctx: &AuthContext,
    ) -> Result<ObjectMetadata>;

    // download はオブジェクトのバイナリ本体を返す（大容量の場合は実装がストリーム化する）。
    // auth_ctx はアクセス制御と audit ログに使用する。
    async fn download(
        &self,
        bucket: &str,
        key: &str,
        auth_ctx: &AuthContext,
    ) -> Result<Vec<u8>>;

    // head はオブジェクトのメタデータのみを返す（本体は取得しない）。
    async fn head(
        &self,
        bucket: &str,
        key: &str,
        auth_ctx: &AuthContext,
    ) -> Result<Option<ObjectMetadata>>;

    // delete はオブジェクトを削除する（存在しないキーを削除してもエラーにならない）。
    async fn delete(&self, bucket: &str, key: &str, auth_ctx: &AuthContext) -> Result<()>;

    // list はプレフィックスに一致するオブジェクト一覧を返す（ページネーション対応）。
    async fn list(
        &self,
        bucket: &str,
        prefix: &str,
        continuation_token: Option<&str>,
        auth_ctx: &AuthContext,
    ) -> Result<ListResult>;

    // presign_get はオブジェクトの一時取得 URL を生成する（有効期間は HLC tick 数で指定）。
    // 生成した URL は auth_ctx のテナント境界内のオブジェクトにのみ有効。
    async fn presign_get(
        &self,
        bucket: &str,
        key: &str,
        ttl_ticks: u64,
        auth_ctx: &AuthContext,
    ) -> Result<String>;
}
