// セッションドメイン層のモジュール定義。
// SessionError は crate::error モジュールで一元管理するため、domain/error は持たない。
pub mod entity;
pub mod repository;
pub mod service;
