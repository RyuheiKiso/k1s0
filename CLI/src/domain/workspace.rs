use std::path::PathBuf;

use super::error::WorkspaceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePath(PathBuf);

impl WorkspacePath {
    pub fn new(raw: &str) -> Result<Self, WorkspaceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(WorkspaceError::EmptyPath);
        }
        let path = PathBuf::from(trimmed);
        if !path.is_absolute() {
            return Err(WorkspaceError::NotAbsolute(trimmed.to_string()));
        }
        Ok(Self(path))
    }

    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        assert_eq!(WorkspacePath::new(""), Err(WorkspaceError::EmptyPath));
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert_eq!(WorkspacePath::new("   "), Err(WorkspaceError::EmptyPath));
    }

    #[test]
    fn relative_path_is_rejected() {
        let result = WorkspacePath::new("relative/path");
        assert_eq!(
            result,
            Err(WorkspaceError::NotAbsolute("relative/path".to_string()))
        );
    }

    #[test]
    fn absolute_path_is_accepted() {
        let result = WorkspacePath::new(r"C:\Users\test\workspace");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string_lossy(),
            r"C:\Users\test\workspace"
        );
    }

    #[test]
    fn leading_trailing_whitespace_is_trimmed() {
        let result = WorkspacePath::new(r"  C:\Users\test  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), r"C:\Users\test");
    }
}
