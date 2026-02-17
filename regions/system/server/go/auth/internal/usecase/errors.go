package usecase

import "errors"

var (
	// ErrInvalidToken はトークンが無効な場合のエラー。
	ErrInvalidToken = errors.New("invalid token")

	// ErrInvalidIssuer は issuer が一致しない場合のエラー。
	ErrInvalidIssuer = errors.New("invalid issuer")

	// ErrInvalidAudience は audience が一致しない場合のエラー。
	ErrInvalidAudience = errors.New("invalid audience")

	// ErrTokenExpired はトークンの有効期限が切れている場合のエラー。
	ErrTokenExpired = errors.New("token expired")

	// ErrUserNotFound はユーザーが見つからない場合のエラー。
	ErrUserNotFound = errors.New("user not found")
)
