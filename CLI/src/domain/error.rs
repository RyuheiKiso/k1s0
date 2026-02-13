use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    EmptyPath,
    NotAbsolute(String),
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "ワークスペースパスが空です"),
            Self::NotAbsolute(path) => {
                write!(f, "絶対パスを指定してください: {path}")
            }
        }
    }
}

impl std::error::Error for WorkspaceError {}
