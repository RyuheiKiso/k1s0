//! k1s0 認証/認可ライブラリ
//!
//! JWT/OIDC検証、ポリシー評価、監査ログの機能を提供する。
//!
//! # 機能
//!
//! - **JWT検証**: JWKSローテーション対応のJWT検証
//! - **ポリシー評価**: 柔軟な権限制御ポリシーエンジン
//! - **監査ログ**: 認証・認可・操作の監査ログ出力
//! - **ミドルウェア**: HTTP/gRPC用の認証ミドルウェア
//!
//! # 使用例
//!
//! ## JWT検証
//!
//! ```rust,no_run
//! use k1s0_auth::jwt::{JwtVerifier, JwtVerifierConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = JwtVerifierConfig::new("https://auth.example.com")
//!         .with_jwks_uri("https://auth.example.com/.well-known/jwks.json")
//!         .with_audience("my-api");
//!
//!     let verifier = JwtVerifier::new(config);
//!
//!     let claims = verifier.verify("eyJ...").await?;
//!     println!("User: {}", claims.sub);
//!     Ok(())
//! }
//! ```
//!
//! ## ポリシー評価
//!
//! ```rust
//! use k1s0_auth::policy::{PolicyEvaluator, PolicyBuilder, PolicySubject, Action, PolicyRequest, ResourceContext};
//!
//! #[tokio::main]
//! async fn main() {
//!     let evaluator = PolicyEvaluator::new();
//!
//!     // 管理者ルールを追加
//!     let rules = PolicyBuilder::new()
//!         .admin_rule("admin")
//!         .read_rule("user_read", "user", vec!["user"], 10)
//!         .build();
//!     evaluator.add_rules(rules).await;
//!
//!     // ポリシーを評価
//!     let subject = PolicySubject::new("user123").with_role("admin");
//!     let action = Action::new("user", "delete");
//!     let request = PolicyRequest {
//!         subject,
//!         action,
//!         resource: ResourceContext::default(),
//!     };
//!
//!     let result = evaluator.evaluate(&request).await;
//!     assert!(result.is_allowed());
//! }
//! ```
//!
//! ## 監査ログ
//!
//! ```rust
//! use k1s0_auth::audit::{AuditLogger, AuditActor, AuditEvent, AuditEventType, AuditResult};
//!
//! let logger = AuditLogger::with_default_sink("my-service");
//!
//! // 認証成功を記録
//! let actor = AuditActor::new("user123").with_ip_address("192.168.1.1");
//! logger.log_authentication_success(actor);
//! ```

pub mod audit;
pub mod error;
pub mod jwt;
pub mod middleware;
pub mod policy;

// 主要な型を再エクスポート
pub use audit::{AuditActor, AuditEvent, AuditEventType, AuditLogger, AuditResource, AuditResult};
pub use error::AuthError;
pub use jwt::{Claims, JwtVerifier, JwtVerifierConfig};
pub use middleware::{AuthContext, AuthMiddleware, AuthSkipMatcher};
pub use policy::{
    Action, PolicyBuilder, PolicyDecision, PolicyEvaluator, PolicyRequest, PolicyResult,
    PolicySubject, ResourceContext,
};
