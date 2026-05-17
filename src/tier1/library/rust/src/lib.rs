// lib.rs — k1s0 tier1 Library Rust エントリーポイント
// 05_鍵管理適合仕様.md / 04_認証適合仕様.md に基づく 4 言語等価強度 SDK の公開 API を定義する。
// 生 key bytes / 生 access_token は公開 API シグネチャに一切露出しない。

// key_handle モジュール: KeyClass enum + KeyHandle trait + OpenBaoKeyHandle struct
pub mod key_handle;
// auth_context モジュール: AuthClass enum + AuthContext struct
pub mod auth_context;
// repository モジュール: Repository<T> trait + PgRepository<T> 実装
pub mod repository;
