pub mod app;
pub mod claims;
pub mod download_stat;
pub mod platform;
pub mod version;

// H-02 監査対応: 各エンティティをサブモジュールとして宣言するのみとし、
// re-export は使用箇所で直接パスを指定する形に変更（unused_imports 警告対応）
