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
	// github.com/k1s0/hlc-lib-go: HLC クロック（wall-clock TTL 禁止規律に従い time.Now() の代替として使用する）
	github.com/k1s0/hlc-lib-go v0.0.0-00010101000000-000000000000
)

// ローカル path への replace ディレクティブ（HLC lib は src/client/hlc_lib/go に配置されている）
replace github.com/k1s0/hlc-lib-go => ../../../client/hlc_lib/go

require (
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/jackc/pgpassfile v1.0.0 // indirect
	github.com/jackc/pgservicefile v0.0.0-20240606120523-5a60cdf6a761 // indirect
	github.com/jackc/puddle/v2 v2.2.2 // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	github.com/stretchr/testify v1.9.0 // indirect
	golang.org/x/crypto v0.31.0 // indirect
	golang.org/x/sync v0.10.0 // indirect
	golang.org/x/text v0.21.0 // indirect
)
