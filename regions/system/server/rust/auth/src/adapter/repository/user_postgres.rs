// UserRepository の PostgreSQL 実装。
// ただし、ユーザー情報は Keycloak が管理するため、
// このリポジトリは将来的なキャッシュ層として使用する。
// 現時点では Keycloak Admin API 経由での取得が主で、
// infrastructure::keycloak_client に UserRepository トレイト実装がある。
