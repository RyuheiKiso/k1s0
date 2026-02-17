package repository

// UserPostgresRepository は PostgreSQL ベースのユーザーリポジトリ。
// 現時点では Keycloak Admin API 経由のため、
// adapter/gateway/keycloak_client.go がこの役割を担う。
// 将来的にユーザー情報のキャッシュ層が必要な場合に使用する。
