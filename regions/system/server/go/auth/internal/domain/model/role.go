package model

// Role はロールを表す。
type Role struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
}

// Permission はロールに紐づくパーミッションを表す。
type Permission struct {
	Role       string `json:"role"`
	Permission string `json:"permission"` // read, write, delete, admin
	Resource   string `json:"resource"`
}
