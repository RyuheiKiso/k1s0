package serviceauth

import (
	"context"
	"fmt"
	"strings"
)

// ServiceClaims はサービストークンのクレーム。
type ServiceClaims struct {
	// Subject はサービスのサブジェクト（クライアント ID）。
	Subject string
	// Issuer はトークン発行者 URL。
	Issuer string
	// Audience はトークンの対象オーディエンス。
	Audience []string
	// ServiceAccountId はサービスアカウント ID（オプション）。
	ServiceAccountId string
}

// SpiffeId は SPIFFE/SPIRE のワークロードアイデンティティ。
// フォーマット: spiffe://<trust-domain>/ns/<namespace>/sa/<service-account>
type SpiffeId struct {
	// TrustDomain は信頼ドメイン。
	TrustDomain string
	// Namespace は Kubernetes ネームスペース。
	Namespace string
	// ServiceAccount は Kubernetes サービスアカウント名。
	ServiceAccount string
}

// String は SPIFFE ID の URI 文字列表現を返す。
func (s *SpiffeId) String() string {
	return fmt.Sprintf("spiffe://%s/ns/%s/sa/%s", s.TrustDomain, s.Namespace, s.ServiceAccount)
}

// ParseSpiffeId は SPIFFE ID 文字列をパースする。
func ParseSpiffeId(uri string) (*SpiffeId, error) {
	if !strings.HasPrefix(uri, "spiffe://") {
		return nil, fmt.Errorf("invalid SPIFFE ID: must start with spiffe://")
	}
	rest := strings.TrimPrefix(uri, "spiffe://")
	parts := strings.SplitN(rest, "/", 2)
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid SPIFFE ID format: %s", uri)
	}
	trustDomain := parts[0]
	path := "/" + parts[1]

	segments := strings.Split(strings.TrimPrefix(path, "/"), "/")
	// segments: ["ns", "<ns>", "sa", "<sa>"]
	if len(segments) < 4 || segments[0] != "ns" || segments[2] != "sa" {
		return nil, fmt.Errorf("invalid SPIFFE ID path (expected /ns/<ns>/sa/<sa>): %s", path)
	}

	return &SpiffeId{
		TrustDomain:    trustDomain,
		Namespace:      segments[1],
		ServiceAccount: segments[3],
	}, nil
}

// ServiceAuthClient はサービス間認証クライアントインターフェース。
type ServiceAuthClient interface {
	// GetToken は OAuth2 Client Credentials フローで新しいトークンを取得する。
	GetToken(ctx context.Context) (*ServiceToken, error)
	// GetCachedToken はキャッシュからトークンを返す（期限切れなら再取得）。
	// 返り値は Bearer トークン文字列 ("Bearer <token>")。
	GetCachedToken(ctx context.Context) (string, error)
	// ValidateSpiffeId は SPIFFE ID を検証し、期待ネームスペースと一致するか確認する。
	ValidateSpiffeId(spiffeId string, expectedNamespace string) (*SpiffeId, error)
}
