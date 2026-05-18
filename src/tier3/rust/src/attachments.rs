// k1s0 tier3 Rust 添付ファイルストア trait（TypeScript 等価強度実装）
// チャンクアップロード / MIME 検査 / ハッシュチェーン整合性を Rust trait として定義する
// 非同期 trait は async-trait マクロを使用せず標準 Future で表現する

// 標準ライブラリの非同期 Future をインポートする
use std::future::Future;
// Pin: 非同期 Future を安全にスタックから移動させないために使用する
use std::pin::Pin;

// --------- 定数 ---------

// デフォルトのチャンクサイズ（4 MiB）
pub const DEFAULT_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;

// 許可する MIME タイプ一覧（&str スライス）
pub const ALLOWED_MIME_TYPES: &[&str] = &[
    // PDF 文書
    "application/pdf",
    // Microsoft Excel（新形式）
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    // Microsoft Word（新形式）
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    // CSV テキスト
    "text/csv",
    // プレーンテキスト
    "text/plain",
    // JPEG 画像
    "image/jpeg",
    // PNG 画像
    "image/png",
];

// --------- 型定義 ---------

// 添付ファイルのメタデータ型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentMeta {
    // ファイル名
    pub file_name: String,
    // MIME タイプ
    pub mime_type: String,
    // ファイルサイズ（バイト）
    pub size_bytes: u64,
    // ファイル全体の SHA-256 ハッシュ（hex 文字列）
    pub sha256_hex: String,
}

// チャンクデータ型
#[derive(Debug, Clone)]
pub struct AttachmentChunk {
    // チャンク番号（0 始まり）
    pub chunk_index: u32,
    // チャンクのバイナリデータ
    pub data: Vec<u8>,
    // このチャンクの SHA-256 ハッシュ（hex 文字列）
    pub chunk_sha256_hex: String,
}

// アップロード結果型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentUploadResult {
    // 付与された添付ファイル ID（サーバー生成 UUID）
    pub attachment_id: String,
    // アップロード完了時刻（ISO 8601）
    pub completed_at: String,
    // サーバー側で計算されたファイル全体ハッシュ
    pub server_sha256_hex: String,
}

// 動的エラー型（Box<dyn std::error::Error + Send + Sync> の短縮形）
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

// --------- MIME 検査関数 ---------

// MIME タイプが許可リストに含まれているか検査する
pub fn check_mime_type(mime_type: &str) -> bool {
    // 許可 MIME タイプ一覧に含まれているか線形探索する
    ALLOWED_MIME_TYPES.contains(&mime_type)
}

// --------- AttachmentStore trait ---------

// AttachmentStore: 添付ファイルの保存・取得・削除を抽象化する trait
// 非同期メソッドは Pin<Box<dyn Future>> を返す（async-trait マクロ不使用）
pub trait AttachmentStore: Send + Sync {
    // アップロードセッションを開始する（multipart upload init 相当）
    // メタデータを受け取り、アップロード ID を返す
    fn init_upload(
        &self,
        meta: AttachmentMeta,
    ) -> Pin<Box<dyn Future<Output = Result<String, BoxError>> + Send + '_>>;

    // チャンクをアップロードする（チャンク番号とデータを受け取る）
    fn upload_chunk(
        &self,
        upload_id: &str,
        chunk: AttachmentChunk,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + '_>>;

    // 全チャンクのアップロード完了を通知してアップロード結果を返す
    fn complete_upload(
        &self,
        upload_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<AttachmentUploadResult, BoxError>> + Send + '_>>;

    // 指定した添付ファイル ID のメタデータを取得する
    fn get_meta(
        &self,
        attachment_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<AttachmentMeta, BoxError>> + Send + '_>>;

    // 指定した添付ファイル ID を論理削除する
    fn delete(
        &self,
        attachment_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + '_>>;
}

// --------- テスト ---------

#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // check_mime_type が許可 MIME タイプを正しく判定するかテストする
    #[test]
    fn test_check_mime_type_allowed() {
        // PDF は許可されているので true を返すことを確認する
        assert!(check_mime_type("application/pdf"));
        // JPEG は許可されているので true を返すことを確認する
        assert!(check_mime_type("image/jpeg"));
    }

    // check_mime_type が拒否 MIME タイプを正しく判定するかテストする
    #[test]
    fn test_check_mime_type_denied() {
        // 実行可能ファイルは拒否されているので false を返すことを確認する
        assert!(!check_mime_type("application/x-executable"));
        // ZIP は拒否されているので false を返すことを確認する
        assert!(!check_mime_type("application/zip"));
    }
}
