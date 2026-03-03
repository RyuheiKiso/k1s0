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
}
