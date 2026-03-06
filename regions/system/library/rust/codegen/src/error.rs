use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("template rendering failed for `{template}`: {source}")]
    Template {
        template: String,
        source: tera::Error,
    },

    #[error("I/O error at `{}`: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("proto parse error: {0}")]
    ProtoParse(String),

    #[error("cargo update error: {0}")]
    CargoUpdate(String),
}
