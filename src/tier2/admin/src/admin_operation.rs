// tier2 管理操作列挙型（設計方針 14 / 管理境界）
// すべての管理操作は AdminBoundaryGuard を通過してから実行する

// serde: AdminOperation のシリアライズ / デシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: 操作対象テナント / ユーザーの識別子に使用する
use uuid::Uuid;

// AdminOperation: 管理境界を通過できる操作の網羅的列挙
// 業界中立語のみ使用する（業界固有語禁止規約準拠）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdminOperation {
    // テナントプロビジョニング（新規テナントのリソース割り当て）
    TenantProvision {
        // プロビジョニング対象テナント識別子
        tenant_id: Uuid,
        // 割り当てるクォータクラス名
        quota_class: String,
    },
    // テナント一時停止（すべての API アクセスを拒否状態にする）
    TenantSuspend {
        // 停止対象テナント識別子
        tenant_id: Uuid,
        // 停止理由（監査ログに記録される）
        reason: String,
    },
    // クォータ上限の一時的な上書き（通常承認フローを経ない緊急措置）
    QuotaOverride {
        // クォータ上書き対象テナント識別子
        tenant_id: Uuid,
        // 上書きするクォータ項目名
        quota_key: String,
        // 新しいクォータ上限値
        new_limit: i64,
    },
    // ユーザーへの権限付与（AdminBoundaryGuard が二重承認を強制する）
    UserGrant {
        // 権限付与対象ユーザー識別子
        user_id: Uuid,
        // 付与するロール名
        role: String,
        // 付与対象テナント識別子
        tenant_id: Uuid,
    },
    // 緊急アクセス（インシデント対応時のみ / デュアル承認必須）
    EmergencyAccess {
        // 緊急アクセス要求者の識別子
        requestor_id: Uuid,
        // アクセス対象テナント識別子
        target_tenant_id: Uuid,
        // 緊急アクセスの理由（インシデント ID を含む）
        incident_id: String,
    },
}

impl AdminOperation {
    // 操作名の文字列表現を返す（監査ログのイベント種別フィールドに使用する）
    pub fn operation_name(&self) -> &'static str {
        // パターンマッチで各操作の名前文字列を返す
        match self {
            // テナントプロビジョニング操作名
            AdminOperation::TenantProvision { .. } => "TenantProvision",
            // テナント停止操作名
            AdminOperation::TenantSuspend { .. } => "TenantSuspend",
            // クォータ上書き操作名
            AdminOperation::QuotaOverride { .. } => "QuotaOverride",
            // ユーザー権限付与操作名
            AdminOperation::UserGrant { .. } => "UserGrant",
            // 緊急アクセス操作名
            AdminOperation::EmergencyAccess { .. } => "EmergencyAccess",
        }
    }

    // この操作がデュアル承認を必要とするか返す
    pub fn requires_dual_approval(&self) -> bool {
        // EmergencyAccess と QuotaOverride は常にデュアル承認を要求する
        matches!(
            self,
            AdminOperation::EmergencyAccess { .. } | AdminOperation::QuotaOverride { .. }
        )
    }
}
