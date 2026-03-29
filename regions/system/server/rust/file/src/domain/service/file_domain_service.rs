pub struct FileDomainService;

impl FileDomainService {
    pub const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;

    pub fn validate_upload_size(size_bytes: u64, max_file_size_bytes: u64) -> Result<(), String> {
        if size_bytes > max_file_size_bytes {
            return Err(format!(
                "file size exceeds limit: {} > {}",
                size_bytes, max_file_size_bytes
            ));
        }
        Ok(())
    }

    pub fn can_access_tenant_resource(resource_tenant_id: &str, requester_tenant_id: &str) -> bool {
        !requester_tenant_id.is_empty() && resource_tenant_id == requester_tenant_id
    }

    /// storage_path からテナントIDを抽出する。
    /// storage_path は `{tenant_id}/{filename}` 形式で構成されているため、
    /// 最初の '/' より前のセグメントがテナントIDとなる。
    /// MED-03: 抽出したセグメントが '..' や空文字でないことを検証し、
    /// パストラバーサルを利用したテナント境界突破を防止する。
    pub fn tenant_id_from_storage_path(storage_path: &str) -> Option<&str> {
        storage_path
            .split('/')
            .next()
            // 空文字・".."・"." を含む無効なセグメントを拒否する
            .filter(|s| !s.is_empty() && *s != ".." && *s != ".")
    }
}
