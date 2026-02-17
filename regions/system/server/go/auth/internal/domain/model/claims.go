package model

// TokenClaims は JWT トークンのクレームを表す。
type TokenClaims struct {
	Sub               string                       `json:"sub"`
	Iss               string                       `json:"iss"`
	Aud               string                       `json:"aud"`
	Exp               int64                        `json:"exp"`
	Iat               int64                        `json:"iat"`
	Jti               string                       `json:"jti"`
	Typ               string                       `json:"typ"`
	Azp               string                       `json:"azp"`
	Scope             string                       `json:"scope"`
	PreferredUsername  string                       `json:"preferred_username"`
	Email             string                       `json:"email"`
	RealmAccess       RealmAccess                  `json:"realm_access"`
	ResourceAccess    map[string]ClientRoles       `json:"resource_access"`
	TierAccess        []string                     `json:"tier_access"`
}

// RealmAccess はレルムロール情報を表す。
type RealmAccess struct {
	Roles []string `json:"roles"`
}

// ClientRoles はクライアント固有のロール情報を表す。
type ClientRoles struct {
	Roles []string `json:"roles"`
}
