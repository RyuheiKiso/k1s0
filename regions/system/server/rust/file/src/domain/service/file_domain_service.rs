pub struct FileDomainService;

impl FileDomainService {
    pub const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;

    /// STATIC-HIGH-003 監査対応: アップロード可能なコンテンツタイプの許可リスト。
    /// 許可リスト以外のコンテンツタイプは拒否し、任意ファイル種別のアップロードを防止する。
    pub const ALLOWED_CONTENT_TYPES: &'static [&'static str] = &[
        "application/pdf",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "text/plain",
        "text/csv",
        "application/json",
        "application/zip",
        "application/gzip",
        "application/octet-stream",
    ];

    /// STATIC-HIGH-003 監査対応: 指定されたコンテンツタイプが許可リストに含まれるかを確認する。
    /// "Content-Type: image/png; charset=utf-8" のようなパラメーター付きでも正しく動作するよう
    /// セミコロン区切りでベースタイプのみを抽出して比較する。
    #[must_use]
    pub fn is_allowed_content_type(content_type: &str) -> bool {
        let base_type = content_type.split(';').next().unwrap_or("").trim();
        Self::ALLOWED_CONTENT_TYPES.contains(&base_type)
    }

    pub fn validate_upload_size(size_bytes: u64, max_file_size_bytes: u64) -> Result<(), String> {
        if size_bytes > max_file_size_bytes {
            return Err(format!(
                "file size exceeds limit: {size_bytes} > {max_file_size_bytes}"
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn can_access_tenant_resource(resource_tenant_id: &str, requester_tenant_id: &str) -> bool {
        !requester_tenant_id.is_empty() && resource_tenant_id == requester_tenant_id
    }

    /// `storage_path` からテナントIDを抽出する。
    /// `storage_path` は `{tenant_id}/{filename}` 形式で構成されているため、
    /// 最初の '/' より前のセグメントがテナントIDとなる。
    /// MED-03: 抽出したセグメントが '..' や空文字でないことを検証し、
    /// パストラバーサルを利用したテナント境界突破を防止する。
    #[must_use]
    pub fn tenant_id_from_storage_path(storage_path: &str) -> Option<&str> {
        storage_path
            .split('/')
            .next()
            // 空文字・".."・"." を含む無効なセグメントを拒否する
            .filter(|s| !s.is_empty() && *s != ".." && *s != ".")
    }
}
