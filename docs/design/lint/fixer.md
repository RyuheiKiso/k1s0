# 自動修正（Fixer）

← [Lint 設計書](./)

---

## API

```rust
impl Fixer {
    /// 新しい Fixer を作成
    pub fn new(base_path: &Path) -> Self;

    /// 違反を修正する
    pub fn fix(&self, violation: &Violation) -> Option<FixResult>;

    /// ルールが自動修正可能かどうか
    pub fn is_fixable(rule: RuleId) -> bool;
}
```

## 修正可能なルール

| ルール | 修正内容 |
|--------|---------|
| K010 | ディレクトリを作成 |
| K011 | 空ファイルを作成 |

## FixResult

```rust
pub struct FixResult {
    /// 修正したファイルパス
    pub path: PathBuf,
    /// 修正の説明
    pub description: String,
    /// 成功したかどうか
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
}
```
