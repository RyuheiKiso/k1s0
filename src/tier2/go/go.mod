// k1s0 tier2 Go モジュール: テナントコンテキスト（4 GUC 注入）の Go 実装
module github.com/k1s0/tier2

// Go バージョン: 1.23 を使用する
go 1.23.0

// ツールチェーンバージョンを固定する
toolchain go1.23.4

// 直接依存関係
require (
	// github.com/google/uuid: UUID 生成（tenant_id / actor_id に使用する）
	github.com/google/uuid v1.6.0
	// github.com/jackc/pgx/v5: PostgreSQL クライアント（実 transaction に使用する）
	// stdlib サブパッケージを使って database/sql 互換インターフェースを提供する
	github.com/jackc/pgx/v5 v5.7.2
)

require (
	github.com/jackc/pgpassfile v1.0.0 // indirect
	github.com/jackc/pgservicefile v0.0.0-20240606120523-5a60cdf6a761 // indirect
	github.com/jackc/puddle/v2 v2.2.2 // indirect
	golang.org/x/crypto v0.31.0 // indirect
	golang.org/x/sync v0.10.0 // indirect
	golang.org/x/text v0.21.0 // indirect
)
