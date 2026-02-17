package model

import "time"

// User は Keycloak ユーザーを表すドメインエンティティ。
type User struct {
	ID            string              `json:"id"`
	Username      string              `json:"username"`
	Email         string              `json:"email"`
	FirstName     string              `json:"first_name"`
	LastName      string              `json:"last_name"`
	Enabled       bool                `json:"enabled"`
	EmailVerified bool                `json:"email_verified"`
	CreatedAt     time.Time           `json:"created_at"`
	Attributes    map[string][]string `json:"attributes,omitempty"`
}
