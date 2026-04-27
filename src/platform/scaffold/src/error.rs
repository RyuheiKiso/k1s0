// 本ファイルは k1s0-scaffold engine のエラー型。
// CLI / Backstage UI の両経路で同じ型を返す。

use thiserror::Error;

// scaffold engine が返すエラーの統一表現。
#[derive(Debug, Error)]
pub enum ScaffoldError {
    // I/O 関連（read / write / mkdir / git）
    #[error("io: {0}")]
    Io(String),
    // template.yaml / 入力 JSON の構文エラー
    #[error("parse: {0}")]
    Parse(String),
    // 必須フィールド不足 / template 不在 等の論理エラー
    #[error("validation: {0}")]
    Validation(String),
    // Handlebars テンプレ展開失敗
    #[error("render: {0}")]
    Render(String),
}

impl From<ScaffoldError> for anyhow::Error {
    fn from(e: ScaffoldError) -> Self {
        anyhow::anyhow!(e)
    }
}
