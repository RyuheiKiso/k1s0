# エラーハンドリング

← [CLI 設計書](./)

## エラー型

```rust
pub struct CliError {
    /// エラーの種類
    pub kind: CliErrorKind,
    /// エラーメッセージ
    pub message: String,
    /// 対象（ファイルパス等）
    pub target: Option<String>,
    /// ヒント
    pub hint: Option<String>,
}

pub enum CliErrorKind {
    /// IO エラー
    Io,
    /// 衝突（ファイル/ディレクトリが既に存在）
    Conflict,
    /// バリデーションエラー
    Validation,
    /// manifest が見つからない
    ManifestNotFound,
    /// テンプレートが見つからない
    TemplateNotFound,
    /// 内部エラー
    Internal,
}
```

## 終了コード

```rust
pub enum ExitCode {
    /// 成功
    Success = 0,
    /// 一般的なエラー
    Error = 1,
    /// バリデーションエラー（lint 失敗等）
    ValidationError = 2,
}
```
