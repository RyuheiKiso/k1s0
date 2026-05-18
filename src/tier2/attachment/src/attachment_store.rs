// tier2 業務添付帳票資産: AttachmentStore トレイト（設計方針 28）
// テナント分離されたオブジェクトストレージへの添付ファイルアップロードを抽象化する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: AttachmentMetadata のシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子および添付ファイル識別子に使用する
use uuid::Uuid;

// AttachmentMetadata: アップロード完了後に返すメタデータ構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    // 添付ファイルの一意識別子（UUID v4）
    pub attachment_id: Uuid,
    // アップロード先テナント識別子（RLS で自動フィルタリングされる）
    pub tenant_id: Uuid,
    // ファイルの MIME タイプ（例: application/pdf / image/png）
    pub content_type: String,
    // アップロードされたファイルのバイトサイズ
    pub size_bytes: u64,
    // ストレージ内のオブジェクトキー（テナント分離プレフィックス付き）
    pub object_key: String,
    // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
    pub hlc_timestamp: u64,
    // エンベロープ暗号化で使用された DEK ハンドル（OpenBao Transit で管理する）
    pub dek_handle: String,
}

// AttachmentStore トレイト: テナント分離されたオブジェクトストレージへの操作を抽象化する
// 実装クラスは MinIO または S3 互換ストレージと連携する
pub trait AttachmentStore: Send + Sync {
    // テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
    // tenant_id: アップロード先テナント識別子（AuthContext から注入する / API 引数で受け取り禁止）
    // data: アップロードするファイルのバイト列
    // content_type: ファイルの MIME タイプ
    // アップロード完了後に AttachmentMetadata を返す
    fn upload(
        &self,
        tenant_id: Uuid,
        data: &[u8],
        content_type: &str,
    ) -> Result<AttachmentMetadata>;

    // 添付ファイルを取得する
    // tenant_id: 取得対象テナント識別子（RLS で自動フィルタリングされる）
    // attachment_id: 取得対象添付ファイル識別子
    // ファイルのバイト列を返す
    fn fetch(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<Vec<u8>>;

    // 添付ファイルを削除する（PII データ削除フローから呼び出される）
    // tenant_id: 削除対象テナント識別子
    // attachment_id: 削除対象添付ファイル識別子
    fn delete(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<()>;
}
