// src/lib.rs — bfl security test ライブラリのルート
// Phase C: tier1 bfl 認可境界の failure-path・property test を集約する。
// テスト対象: dpop.rs / oidc.rs / spire_workload.rs / spire_revoke.rs

// テストヘルパーモジュール: JWT compact format を構築するユーティリティ
pub mod jwt_helpers;
