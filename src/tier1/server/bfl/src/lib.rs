// lib.rs — bfl ライブラリターゲット
// 外部テストクレート（tests/bfl_security）から公開モジュールを参照できるよう再エクスポートする。
// バイナリ (main.rs) と並存するライブラリターゲットとして定義する。

// dpop モジュール: RFC 9449 DPoP proof 検証（DPoPProofVerifier / DPoPClaims）
pub mod dpop;
// oidc モジュール: OIDC JWT 検証（JwkCache / OidcClaims 等）
pub mod oidc;
// spire_workload モジュール: SPIRE Workload API クライアント（SVID attestation）
pub mod spire_workload;
// spire_revoke モジュール: SPIRE SVID revocation チェック
pub mod spire_revoke;
// emergency_break_glass モジュール: M-of-N break-glass 認可
pub mod emergency_break_glass;
// token_exchange モジュール: RFC 8693 token exchange 検証
pub mod token_exchange;
// openbao モジュール: OpenBao Transit API クライアント
pub mod openbao;
// auth_context モジュール: AuthContext 型定義
pub mod auth_context;
