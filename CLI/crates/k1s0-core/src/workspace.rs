/// ワークスペースルート検出ユーティリティ。
///
/// `regions/` ディレクトリと `infra/helm/services/` ディレクトリの両方が存在する
/// 最も近い祖先ディレクトリをワークスペースルートとして返す。
use std::path::{Path, PathBuf};

/// 指定パスから上位に遡り、k1s0 ワークスペースルートを検出する。
///
/// `regions/` と `infra/helm/services/` の両方が存在するディレクトリを返す。
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|ancestor| {
        let has_regions = ancestor.join("regions").is_dir();
        let has_helm = ancestor
            .join("infra")
            .join("helm")
            .join("services")
            .is_dir();
        if has_regions && has_helm {
            Some(ancestor.to_path_buf())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_workspace_root_found() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("regions")).unwrap();
        std::fs::create_dir_all(root.join("infra/helm/services")).unwrap();

        let nested = root.join("regions/service/task");
        std::fs::create_dir_all(&nested).unwrap();

        let result = find_workspace_root(&nested);
        assert_eq!(result, Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_workspace_root_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = find_workspace_root(tmp.path());
        assert!(result.is_none());
    }
}
