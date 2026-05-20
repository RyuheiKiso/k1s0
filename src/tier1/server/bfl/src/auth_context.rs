// auth_context.rs — k1s0 tier1 bfl: AuthClass / AuthContext の canonical 再エクスポート
// 独自定義を廃止し、k1s0-tier1-library の canonical 実装を参照する。
// CLAUDE.md「重複実装は drift リスクで禁止」規律の物理化。
// bfl 固有の呼び出しは main.rs 側で canonical シグネチャに合わせる。

// k1s0-tier1-library の AuthClass を canonical 実装から再エクスポートする
pub use k1s0_tier1_library::auth_context::AuthClass;
// k1s0-tier1-library の AuthContext を canonical 実装から再エクスポートする
pub use k1s0_tier1_library::auth_context::AuthContext;
