// auth.rs — k1s0 tier1 Library core: Authentication/Authorization L3 facade
// 04_認証適合仕様.md §v1 auth_class セット（5 class）に準拠する。
// auth_context.rs の AuthClass / AuthContext をこのモジュールから re-export する。
// 追加で AuthVerifier trait を定義する（token 検証抽象）。
// 公開 API に生 access_token / JWT claims 型を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;

// auth_context.rs から AuthClass / AuthContext を re-export する
// （後方互換のため crate::auth_context も維持する）
pub use crate::auth_context::{AuthClass, AuthContext};

// AuthVerificationResult は token 検証の結果を表す Library 独自型。
// 検証成功時は AuthContext を返し、失敗時はエラー理由を返す。
// 生 JWT claims オブジェクト（jsonwebtoken::Claims 等）は返さない。
#[derive(Debug)]
pub enum AuthVerificationResult {
    // Valid: 検証成功（AuthContext を伴う）
    Valid(AuthContext),
    // Invalid: 検証失敗（理由メッセージ付き; 生 token を含まない）
    Invalid(String),
    // StepUpRequired: step_up challenge が必要（resource ラベル付き）
    StepUpRequired { resource: String },
}

// ScopeRequirement は認可チェックに使用するスコープ要件を表す struct。
#[derive(Debug, Clone)]
pub struct ScopeRequirement {
    // required_scopes: この要件を満たすために必要なスコープ（AND 条件）
    pub required_scopes: Vec<String>,
    // any_of_scopes: required_scopes が空の場合に代替するスコープ（OR 条件）
    pub any_of_scopes: Vec<String>,
}

// ScopeRequirement のファクトリメソッド群
impl ScopeRequirement {
    // all はスコープをすべて持つことを要件とする（AND 条件）
    pub fn all(scopes: Vec<String>) -> Self {
        // required_scopes に設定し any_of_scopes は空にする
        Self {
            required_scopes: scopes,
            any_of_scopes: vec![],
        }
    }

    // any はいずれかのスコープを持つことを要件とする（OR 条件）
    pub fn any(scopes: Vec<String>) -> Self {
        // any_of_scopes に設定し required_scopes は空にする
        Self {
            required_scopes: vec![],
            any_of_scopes: scopes,
        }
    }
}

// AuthVerifier は token 検証と認可チェックを担う L3 trait。
// 公開 API に生 access_token / JWT ライブラリ型を露出しない。
#[async_trait]
pub trait AuthVerifier: Send + Sync {
    // verify は opaque な token 文字列を受け取り、AuthVerificationResult を返す。
    // 生 token 文字列は戻り値に含まれない（AuthContext に変換して返す）。
    async fn verify(&self, opaque_token: &str) -> Result<AuthVerificationResult>;

    // check_scope は AuthContext とスコープ要件を受け取り、認可可否を返す。
    // AuthContext の scopes フィールドを評価する（OSS RBAC ライブラリに依存しない）。
    fn check_scope(&self, ctx: &AuthContext, requirement: &ScopeRequirement) -> bool;

    // refresh は opaque な refresh_token を受け取り、新しい AuthContext を返す。
    // 生 refresh_token は戻り値に含まれない。
    async fn refresh(&self, opaque_refresh_token: &str) -> Result<AuthVerificationResult>;
}

// check_scope_static は AuthVerifier を持たない文脈でのスコープチェック。
// AuthContext と ScopeRequirement を受け取り、認可可否を返す純粋関数。
pub fn check_scope_static(ctx: &AuthContext, requirement: &ScopeRequirement) -> bool {
    // is_valid=false の場合は認可を拒否する
    if !ctx.is_valid {
        return false;
    }
    // required_scopes が空でない場合は AND チェックを行う
    if !requirement.required_scopes.is_empty() {
        // required_scopes の全スコープが ctx.scopes に含まれているか確認する
        return requirement
            .required_scopes
            .iter()
            .all(|s| ctx.scopes.contains(s));
    }
    // any_of_scopes が空でない場合は OR チェックを行う
    if !requirement.any_of_scopes.is_empty() {
        // any_of_scopes のいずれかのスコープが ctx.scopes に含まれているか確認する
        return requirement
            .any_of_scopes
            .iter()
            .any(|s| ctx.scopes.contains(s));
    }
    // 要件が空の場合は認可を許可する（スコープ制限なし）
    true
}
