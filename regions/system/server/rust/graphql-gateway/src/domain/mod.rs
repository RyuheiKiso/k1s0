pub mod error;
pub mod loader;
pub mod model;
// ドメイン層のポートトレイト定義。依存性逆転の原則によりインフラ層への直接依存を排除する。
pub mod port;
