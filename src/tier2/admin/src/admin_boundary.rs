// tier2 管理境界ガード（設計方針 14 / AdminBoundaryGuard トレイト定義）
// すべての管理操作はこのトレイトを実装したガードを通過してから実行する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: AdminRequest / AdminResponse のシリアライズに使用する
use serde::{Deserialize, Serialize};
// thiserror: 管理境界違反エラー型の定義に使用する
use thiserror::Error;
// uuid: リクエスト識別子および呼び出し元識別子に使用する
use uuid::Uuid;
// AdminOperation 列挙型をインポートする
use crate::admin_operation::AdminOperation;

// 管理境界違反エラー型（AdminBoundaryGuard が返すエラー）
#[derive(Debug, Error)]
pub enum AdminBoundaryError {
    // 呼び出し元トークンが管理スコープを持っていない
    #[error("管理スコープなし: {0}")]
    InsufficientScope(String),
    // デュアル承認が必要な操作で承認が足りない
    #[error("デュアル承認未完了: 操作 {operation} には承認が {required} 件必要")]
    DualApprovalRequired {
        // 承認不足の操作名
        operation: String,
        // 必要な承認件数
        required: u32,
    },
    // 操作対象テナントへのアクセス権がない
    #[error("テナントアクセス拒否: {0}")]
    TenantAccessDenied(String),
}

// AdminRequest: 管理操作リクエストのエンベロープ型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRequest {
    // リクエスト一意識別子（UUID v4 / 監査ログの相関 ID に使用する）
    pub request_id: Uuid,
    // 呼び出し元識別子（ユーザー ID またはサービスアカウント ID）
    pub caller_id: Uuid,
    // 実行しようとしている管理操作
    pub operation: AdminOperation,
    // 操作の正当化理由（監査ログに記録される）
    pub justification: String,
}

// AdminResponse: 管理操作レスポンスのエンベロープ型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminResponse {
    // 対応するリクエスト識別子（相関追跡に使用する）
    pub request_id: Uuid,
    // 操作が成功したか
    pub success: bool,
    // 操作結果メッセージ（成功時は完了詳細 / 失敗時はエラー詳細）
    pub message: String,
    // 操作が監査ログに記録されたか（必ず true でなければならない）
    pub audit_recorded: bool,
}

// AdminBoundaryGuard トレイト: 管理操作の境界チェックを実装する契約
// 実装クラスは Keycloak + Kyverno ポリシーと連動してスコープ検証を行う
pub trait AdminBoundaryGuard: Send + Sync {
    // 管理操作の実行スコープを検証する
    // token: Bearer トークン（KeyCloak JWT）
    // operation: 実行しようとしている管理操作
    // スコープ不足またはデュアル承認未完了の場合は AdminBoundaryError を返す
    fn check_admin_scope(
        &self,
        token: &str,
        operation: &AdminOperation,
    ) -> Result<(), AdminBoundaryError>;

    // 管理リクエストを処理する（スコープ検証後に呼び出す）
    // 操作の実行結果を AdminResponse として返す
    fn process_request(&self, request: &AdminRequest) -> Result<AdminResponse>;
}
