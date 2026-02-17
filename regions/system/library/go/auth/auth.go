// Package auth は JWT JWKS 検証と RBAC を提供するサーバー用認証ライブラリ。
//
// JWKS エンドポイントから公開鍵を取得し、JWT の署名検証を行う。
// Keycloak が発行する JWT Claims に準拠した認証・認可チェックを提供する。
//
// 使い方:
//
//	verifier := auth.NewJWKSVerifier(jwksURL, issuer, audience, 10*time.Minute)
//	claims, err := verifier.VerifyToken(ctx, tokenString)
//	if err != nil {
//	    // 認証エラー
//	}
//	if !auth.HasPermission(claims, "orders", "read") {
//	    // 権限なし
//	}
package auth
