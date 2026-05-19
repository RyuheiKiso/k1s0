// storage.rs — k1s0 tier1 Library: オブジェクトストレージ L1+ facade trait
// S3 互換 API 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// put / get / delete の 3 操作で基本的なオブジェクト管理を提供する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// ObjectStorage は S3 互換オブジェクトストレージを L1+ ラップする facade trait。
// 公開 API シグネチャに OSS 型（aws_sdk_s3::Client 等）を一切含まない。
// bucket / key の 2 軸でオブジェクトを識別する（S3 互換の命名規約に準拠する）。
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    // put はオブジェクトをバケットにアップロードする。
    // bucket はバケット名（例: "k1s0-artifacts" / "k1s0-backups"）。
    // key はオブジェクトキー（例: "tenant-001/file.pdf"）。
    // data はアップロードするバイト列。
    // mime_type は Content-Type ヘッダー（例: "application/pdf" / "image/png"）。
    async fn put(&self, bucket: &str, key: &str, data: Vec<u8>, mime_type: &str) -> crate::Result<()>;

    // get は指定バケットとキーのオブジェクトをダウンロードする。
    // オブジェクトが存在しない場合はエラーを返す（Option ではなく Result を使用する）。
    async fn get(&self, bucket: &str, key: &str) -> crate::Result<Vec<u8>>;

    // delete は指定バケットとキーのオブジェクトを削除する。
    // オブジェクトが存在しない場合はエラーにならない（idempotent な操作）。
    async fn delete(&self, bucket: &str, key: &str) -> crate::Result<()>;
}
