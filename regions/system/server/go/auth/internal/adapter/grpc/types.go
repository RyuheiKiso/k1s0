package grpc

// proto 生成コードが未生成のため、auth_service.proto と common/types.proto に
// 対応する Go 構造体を手動定義する。buf generate 後にこのファイルは生成コードに置き換える。

// --- Common Types ---

// Pagination はページネーションパラメータ。
type Pagination struct {
	Page     int32 `json:"page"`
	PageSize int32 `json:"page_size"`
}

// PaginationResult はページネーション結果。
type PaginationResult struct {
	TotalCount int32 `json:"total_count"`
	Page       int32 `json:"page"`
	PageSize   int32 `json:"page_size"`
	HasNext    bool  `json:"has_next"`
}

// Timestamp は protobuf Timestamp 互換型。
type Timestamp struct {
	Seconds int64 `json:"seconds"`
	Nanos   int32 `json:"nanos"`
}

// --- ValidateToken ---

// ValidateTokenRequest はトークン検証リクエスト。
type ValidateTokenRequest struct {
	Token string `json:"token"`
}

// ValidateTokenResponse はトークン検証レスポンス。
type ValidateTokenResponse struct {
	Valid        bool              `json:"valid"`
	Claims       *PbTokenClaims    `json:"claims,omitempty"`
	ErrorMessage string            `json:"error_message,omitempty"`
}

// PbTokenClaims は proto の TokenClaims に対応する構造体。
type PbTokenClaims struct {
	Sub              string                    `json:"sub"`
	Iss              string                    `json:"iss"`
	Aud              string                    `json:"aud"`
	Exp              int64                     `json:"exp"`
	Iat              int64                     `json:"iat"`
	Jti              string                    `json:"jti"`
	PreferredUsername string                   `json:"preferred_username"`
	Email            string                    `json:"email"`
	RealmAccess      *PbRealmAccess            `json:"realm_access,omitempty"`
	ResourceAccess   map[string]*PbClientRoles `json:"resource_access,omitempty"`
	TierAccess       []string                  `json:"tier_access,omitempty"`
}

// PbRealmAccess は proto の RealmAccess に対応する構造体。
type PbRealmAccess struct {
	Roles []string `json:"roles"`
}

// PbClientRoles は proto の ClientRoles に対応する構造体。
type PbClientRoles struct {
	Roles []string `json:"roles"`
}

// --- User ---

// GetUserRequest はユーザー情報取得リクエスト。
type GetUserRequest struct {
	UserId string `json:"user_id"`
}

// GetUserResponse はユーザー情報取得レスポンス。
type GetUserResponse struct {
	User *PbUser `json:"user"`
}

// PbUser は proto の User に対応する構造体。
type PbUser struct {
	Id            string                  `json:"id"`
	Username      string                  `json:"username"`
	Email         string                  `json:"email"`
	FirstName     string                  `json:"first_name"`
	LastName      string                  `json:"last_name"`
	Enabled       bool                    `json:"enabled"`
	EmailVerified bool                    `json:"email_verified"`
	CreatedAt     *Timestamp              `json:"created_at,omitempty"`
	Attributes    map[string]*PbStringList `json:"attributes,omitempty"`
}

// PbStringList は proto の StringList に対応する構造体。
type PbStringList struct {
	Values []string `json:"values"`
}

// --- ListUsers ---

// ListUsersRequest はユーザー一覧取得リクエスト。
type ListUsersRequest struct {
	Pagination *Pagination `json:"pagination,omitempty"`
	Search     string      `json:"search,omitempty"`
	Enabled    *bool       `json:"enabled,omitempty"`
}

// ListUsersResponse はユーザー一覧取得レスポンス。
type ListUsersResponse struct {
	Users      []*PbUser         `json:"users"`
	Pagination *PaginationResult `json:"pagination,omitempty"`
}

// --- Roles ---

// GetUserRolesRequest はユーザーロール取得リクエスト。
type GetUserRolesRequest struct {
	UserId string `json:"user_id"`
}

// GetUserRolesResponse はユーザーロール取得レスポンス。
type GetUserRolesResponse struct {
	UserId      string                `json:"user_id"`
	RealmRoles  []*PbRole             `json:"realm_roles"`
	ClientRoles map[string]*PbRoleList `json:"client_roles"`
}

// PbRole は proto の Role に対応する構造体。
type PbRole struct {
	Id          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
}

// PbRoleList は proto の RoleList に対応する構造体。
type PbRoleList struct {
	Roles []*PbRole `json:"roles"`
}

// --- Permission ---

// CheckPermissionRequest はパーミッション確認リクエスト。
type CheckPermissionRequest struct {
	UserId     string   `json:"user_id"`
	Permission string   `json:"permission"`
	Resource   string   `json:"resource"`
	Roles      []string `json:"roles"`
}

// CheckPermissionResponse はパーミッション確認レスポンス。
type CheckPermissionResponse struct {
	Allowed bool   `json:"allowed"`
	Reason  string `json:"reason,omitempty"`
}

// --- Audit Log ---

// RecordAuditLogRequest は監査ログ記録リクエスト。
type RecordAuditLogRequest struct {
	EventType string            `json:"event_type"`
	UserId    string            `json:"user_id"`
	IpAddress string            `json:"ip_address"`
	UserAgent string            `json:"user_agent"`
	Resource  string            `json:"resource"`
	Action    string            `json:"action"`
	Result    string            `json:"result"`
	Metadata  map[string]string `json:"metadata"`
}

// RecordAuditLogResponse は監査ログ記録レスポンス。
type RecordAuditLogResponse struct {
	Id         string     `json:"id"`
	RecordedAt *Timestamp `json:"recorded_at,omitempty"`
}

// SearchAuditLogsRequest は監査ログ検索リクエスト。
type SearchAuditLogsRequest struct {
	Pagination *Pagination `json:"pagination,omitempty"`
	UserId     string      `json:"user_id,omitempty"`
	EventType  string      `json:"event_type,omitempty"`
	From       *Timestamp  `json:"from,omitempty"`
	To         *Timestamp  `json:"to,omitempty"`
	Result     string      `json:"result,omitempty"`
}

// SearchAuditLogsResponse は監査ログ検索レスポンス。
type SearchAuditLogsResponse struct {
	Logs       []*PbAuditLog     `json:"logs"`
	Pagination *PaginationResult `json:"pagination,omitempty"`
}

// PbAuditLog は proto の AuditLog に対応する構造体。
type PbAuditLog struct {
	Id         string            `json:"id"`
	EventType  string            `json:"event_type"`
	UserId     string            `json:"user_id"`
	IpAddress  string            `json:"ip_address"`
	UserAgent  string            `json:"user_agent"`
	Resource   string            `json:"resource"`
	Action     string            `json:"action"`
	Result     string            `json:"result"`
	Metadata   map[string]string `json:"metadata"`
	RecordedAt *Timestamp        `json:"recorded_at,omitempty"`
}
