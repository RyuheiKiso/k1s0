pub mod claims;
pub mod dependency;
pub mod health;
pub mod scorecard;
pub mod service;
pub mod service_doc;
pub mod team;

// H-02 監査対応: 各エンティティをサブモジュールとして宣言するのみとし、
// re-export は使用箇所で直接パスを指定する形に変更（unused_imports 警告対応）
