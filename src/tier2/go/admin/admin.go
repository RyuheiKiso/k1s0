// k1s0 tier2 admin Go インターフェース定義
// Rust 実装（admin/src/admin_boundary.rs, admin/src/admin_operation.rs）と 4 言語等価強度を持つ Go 版
// すべての管理操作は AdminBoundaryGuard を通過してから実行する（設計方針 14）

// パッケージ名: admin
package admin

import (
	// context パッケージ: リクエストスコープに使用する
	"context"
	// errors パッケージ: エラー生成に使用する
	"errors"
	// fmt パッケージ: エラーメッセージのフォーマットに使用する
	"fmt"

	// uuid パッケージ: リクエスト / 呼び出し元識別子に使用する
	"github.com/google/uuid"
)

// AdminOperationKind: 管理境界を通過できる操作の識別子（業界中立語のみ）
// Rust の AdminOperation enum に対応する
type AdminOperationKind int

const (
	// TenantProvision: テナントプロビジョニング（新規テナントのリソース割り当て）
	TenantProvision AdminOperationKind = iota
	// TenantSuspend: テナント一時停止（すべての API アクセスを拒否状態にする）
	TenantSuspend
	// QuotaOverride: クォータ上限の一時的な上書き（緊急措置）
	QuotaOverride
	// UserGrant: ユーザーへの権限付与（デュアル承認を強制する）
	UserGrant
	// EmergencyAccess: 緊急アクセス（インシデント対応時のみ / デュアル承認必須）
	EmergencyAccess
)

// String: AdminOperationKind の文字列表現を返す（監査ログのイベント種別フィールドに使用する）
func (k AdminOperationKind) String() string {
	// パターンマッチで各操作の名前文字列を返す
	switch k {
	// テナントプロビジョニング操作名
	case TenantProvision:
		return "TenantProvision"
	// テナント停止操作名
	case TenantSuspend:
		return "TenantSuspend"
	// クォータ上書き操作名
	case QuotaOverride:
		return "QuotaOverride"
	// ユーザー権限付与操作名
	case UserGrant:
		return "UserGrant"
	// 緊急アクセス操作名
	case EmergencyAccess:
		return "EmergencyAccess"
	// 未知の操作種別
	default:
		return fmt.Sprintf("Unknown(%d)", int(k))
	}
}

// RequiresDualApproval: この操作がデュアル承認を必要とするか返す
// Rust の AdminOperation.requires_dual_approval() に対応する
func (k AdminOperationKind) RequiresDualApproval() bool {
	// EmergencyAccess と QuotaOverride は常にデュアル承認を要求する
	return k == EmergencyAccess || k == QuotaOverride
}

// AdminRequest: 管理操作リクエストのエンベロープ型
// Rust の AdminRequest 構造体に対応する
type AdminRequest struct {
	// リクエスト一意識別子（UUID v4 / 監査ログの相関 ID に使用する）
	RequestID uuid.UUID
	// 呼び出し元識別子（ユーザー ID またはサービスアカウント ID）
	CallerID uuid.UUID
	// 実行しようとしている管理操作種別
	OperationKind AdminOperationKind
	// 操作の正当化理由（監査ログに記録される）
	Justification string
	// 操作固有パラメータ（JSON シリアライズ形式で渡す）
	OperationPayloadJSON string
}

// AdminResponse: 管理操作レスポンスのエンベロープ型
// Rust の AdminResponse 構造体に対応する
type AdminResponse struct {
	// 対応するリクエスト識別子（相関追跡に使用する）
	RequestID uuid.UUID
	// 操作が成功したか
	Success bool
	// 操作結果メッセージ（成功時は完了詳細 / 失敗時はエラー詳細）
	Message string
	// 操作が監査ログに記録されたか（必ず true でなければならない）
	AuditRecorded bool
}

// AdminBoundaryErrorKind: 管理境界違反の種別
// Rust の AdminBoundaryError バリアントに対応する
type AdminBoundaryErrorKind int

const (
	// ErrKindInsufficientScope: 呼び出し元トークンが管理スコープを持っていない
	ErrKindInsufficientScope AdminBoundaryErrorKind = iota
	// ErrKindDualApprovalRequired: デュアル承認が必要な操作で承認が足りない
	ErrKindDualApprovalRequired
	// ErrKindTenantAccessDenied: 操作対象テナントへのアクセス権がない
	ErrKindTenantAccessDenied
)

// AdminBoundaryError: 管理境界違反エラー
// Rust の AdminBoundaryError に対応する
type AdminBoundaryError struct {
	// 違反種別
	Kind AdminBoundaryErrorKind
	// エラーメッセージ
	Msg string
}

// Error: error インターフェースを実装する（エラーメッセージを返す）
func (e *AdminBoundaryError) Error() string {
	// Msg フィールドをそのまま返す
	return e.Msg
}

// ErrInsufficientScope: スコープ不足エラーの変数（sentinel value として使用する）
var ErrInsufficientScope = errors.New("admin: 管理スコープなし")

// ErrDualApprovalRequired: デュアル承認未完了エラーの変数
var ErrDualApprovalRequired = errors.New("admin: デュアル承認未完了")

// ErrTenantAccessDenied: テナントアクセス拒否エラーの変数
var ErrTenantAccessDenied = errors.New("admin: テナントアクセス拒否")

// AdminBoundaryGuard: 管理操作の境界チェックを実装する Go インターフェース
// Rust の AdminBoundaryGuard トレイトに対応する（設計方針 14）
// Keycloak + Kyverno ポリシーと連動してスコープ検証を行う
// tenant_id は AuthContext から取得するため API 引数で受け取らない
type AdminBoundaryGuard interface {
	// CheckAdminScope: 管理操作の実行スコープを検証する
	// token: Bearer トークン（Keycloak JWT）
	// operationKind: 実行しようとしている管理操作種別
	// スコープ不足またはデュアル承認未完了の場合は *AdminBoundaryError を返す
	CheckAdminScope(ctx context.Context, token string, operationKind AdminOperationKind) error

	// ProcessRequest: 管理リクエストを処理する（スコープ検証後に呼び出す）
	// 操作の実行結果を *AdminResponse として返す
	// tenant_id は AuthContext から取得するため引数で受け取らない
	ProcessRequest(ctx context.Context, req *AdminRequest) (*AdminResponse, error)
}
