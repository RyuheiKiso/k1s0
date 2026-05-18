// auth.rs — k1s0 tier1 Library frontend: Authentication/Authorization L3 thin wrapper
// frontend 向けの認証 / 認可 thin wrapper を定義する。
// core::auth の型を再利用し、frontend 固有のヘルパーを追加する。
// frontend では token refresh / step_up initiation のみを提供し、
// token 検証は backend で行う（frontend は trust-on-first-use しない）。
// 公開 API に生 access_token / JWT claims 型を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;

// core::auth の型を frontend 向けに re-export する
// （frontend も AuthContext / AuthClass を使用するため）
pub use crate::core::auth::{AuthClass, AuthContext, AuthVerificationResult, ScopeRequirement};
// check_scope_static は frontend でも使用するため re-export する
pub use crate::core::auth::check_scope_static;

// FrontendTokenBundle は frontend が保持するトークン束を表す Library 独自型。
// 生 access_token / refresh_token を文字列として保持するが、
// このモジュール外に露出しない（opaque wrapper として扱う）。
// serde の Serialize を実装しない（ストレージへのシリアライズを防ぐ）。
#[derive(Debug)]
pub struct FrontendTokenBundle {
    // opaque_access_token: アクセストークン（frontend に一時保持を許可するが公開しない）
    opaque_access_token: String,
    // opaque_refresh_token: リフレッシュトークン（frontend に一時保持を許可するが公開しない）
    opaque_refresh_token: Option<String>,
    // auth_ctx: 検証済み AuthContext（backend から受け取る）
    pub auth_ctx: AuthContext,
}

// FrontendTokenBundle のコンストラクタと操作メソッド
impl FrontendTokenBundle {
    // new は opaque トークンと AuthContext から FrontendTokenBundle を生成する
    pub fn new(
        opaque_access_token: String,
        opaque_refresh_token: Option<String>,
        auth_ctx: AuthContext,
    ) -> Self {
        // フィールドを設定する（opaque トークンは外部に露出しない）
        Self {
            opaque_access_token,
            opaque_refresh_token,
            auth_ctx,
        }
    }

    // with_access_token はクロージャにアクセストークンを渡して処理させる。
    // クロージャ外にトークン文字列が漏れないようにするための設計。
    pub fn with_access_token<F, T>(&self, f: F) -> T
    where
        // クロージャはトークン文字列を受け取り T を返す
        F: FnOnce(&str) -> T,
    {
        // トークン文字列をクロージャに渡す（所有権を移さない）
        f(&self.opaque_access_token)
    }

    // has_refresh_token はリフレッシュトークンが存在するかどうかを返す
    pub fn has_refresh_token(&self) -> bool {
        // リフレッシュトークンの有無を返す
        self.opaque_refresh_token.is_some()
    }

    // with_refresh_token はクロージャにリフレッシュトークンを渡して処理させる。
    // リフレッシュトークンが存在しない場合は None を渡す。
    pub fn with_refresh_token<F, T>(&self, f: F) -> T
    where
        // クロージャはリフレッシュトークンの Option<&str> を受け取り T を返す
        F: FnOnce(Option<&str>) -> T,
    {
        // リフレッシュトークンをクロージャに渡す（所有権を移さない）
        f(self.opaque_refresh_token.as_deref())
    }
}

// FrontendAuthClient は frontend 向けの認証クライアント L3 trait。
// token refresh / step_up initiation / scope check のみを提供する。
// token 検証（verify）は含まない（frontend は backend に検証を委譲する）。
#[async_trait]
pub trait FrontendAuthClient: Send + Sync {
    // refresh はリフレッシュトークンを使って新しいトークン束を取得する。
    // 新しい FrontendTokenBundle を返す（生トークン文字列は返さない）。
    async fn refresh(&self, bundle: &FrontendTokenBundle) -> Result<FrontendTokenBundle>;

    // initiate_step_up は step_up challenge を開始する。
    // 返した challenge_url にユーザーをリダイレクトする（SPA のリダイレクト先 URL）。
    // 生トークンは返さない（challenge_url は PKCE state を含む短命 URL）。
    async fn initiate_step_up(
        &self,
        resource: &str,
        auth_ctx: &AuthContext,
    ) -> Result<String>;

    // check_scope は AuthContext とスコープ要件を受け取り、認可可否を返す。
    // frontend でのボタン表示制御等に使用する（API 認可チェックは backend で行う）。
    fn check_scope(&self, ctx: &AuthContext, requirement: &ScopeRequirement) -> bool;
}
