// db.go — k1s0 tier1 Library Go 実装: Relational Store / Single-leader の L1+ interface
// 13_リレーショナルDB適合仕様.md §DbClient（PostgreSQL L1+ 深耕）に準拠する。
// PostgreSQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と RLS を強制する。
// OSS 型（pgxpool.Pool / pgx.Conn 等）を公開シグネチャに一切含まない。

// パッケージ名: keyhandle（tier1 Library の Relational Store API を提供する）
package keyhandle

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
)

// DbRow は DB クエリ結果の単一行を宣言する型。
// OSS の pgx.Row / sql.Row を露出せず Library 独自語彙で表現する。
// Scan でフィールドを取り出す（カラム順に対応する dest ポインターを渡す）。
type DbRow interface {
	// Scan はクエリ結果の各カラムを dest に格納する。
	// dest はカラム順のポインタースライス（pgx.Row.Scan と同一シグネチャ）。
	// 行が存在しない場合は ErrNoRows 相当のエラーを返す。
	Scan(dest ...any) error
}

// DbRows は DB クエリ結果の複数行イテレーターを宣言する型。
// OSS の pgx.Rows / sql.Rows を露出せず Library 独自語彙で表現する。
type DbRows interface {
	// Next は次の行にカーソルを移動する（行が存在する場合 true を返す）。
	Next() bool

	// Scan は現在行の各カラムを dest に格納する。
	Scan(dest ...any) error

	// Err は反復処理中に発生したエラーを返す（Next が false を返した後に呼び出す）。
	Err() error

	// Close は DbRows を必ずクローズする（goroutine / 接続リーク防止のため必ず defer で呼び出す）。
	Close()
}

// DbTxIsoLevel はトランザクション分離レベルを宣言する型。
// PostgreSQL の isolation level に準拠した Library 独自語彙とする。
type DbTxIsoLevel string

const (
	// DbTxIsoReadCommitted: Read Committed（デフォルト: RLS と組み合わせて tenant 分離を保証する）
	DbTxIsoReadCommitted DbTxIsoLevel = "read_committed"
	// DbTxIsoRepeatableRead: Repeatable Read（整合性スナップショット読み取り）
	DbTxIsoRepeatableRead DbTxIsoLevel = "repeatable_read"
	// DbTxIsoSerializable: Serializable（SSI: 完全な直列化保証）
	DbTxIsoSerializable DbTxIsoLevel = "serializable"
)

// DbTxOptions はトランザクション開始オプションを宣言する型。
type DbTxOptions struct {
	// IsoLevel: トランザクション分離レベル（デフォルト: DbTxIsoReadCommitted）
	IsoLevel DbTxIsoLevel
	// ReadOnly: 読み取り専用トランザクションかどうか（true = BEGIN READ ONLY）
	ReadOnly bool
	// DeferConstraints: 制約チェックを DEFERRED にするかどうか（外部キー制約の遅延チェック）
	DeferConstraints bool
}

// DbTx はアクティブなトランザクションを宣言する interface。
// OSS の pgx.Tx / sql.Tx を露出せず Library 独自語彙で表現する。
// 全ての DB 操作は AuthContext を伝播して RLS を機能させる。
type DbTx interface {
	// QueryRow は単一行クエリを実行して DbRow を返す。
	// query はプリペアドクエリの SQL テンプレート（$1 / $2 等のプレースホルダー形式）。
	// 生 SQL 文字列受付 API を禁止する（sprintf 等での SQL 組み立て禁止）。
	QueryRow(ctx context.Context, query string, args ...any) DbRow

	// Query は複数行クエリを実行して DbRows を返す。
	// 使用後は必ず DbRows.Close() を呼び出す（goroutine リーク防止）。
	Query(ctx context.Context, query string, args ...any) (DbRows, error)

	// Exec は DML / DDL を実行して変更された行数を返す。
	// query はプリペアドクエリの SQL テンプレート（生 SQL 文字列組み立て禁止）。
	Exec(ctx context.Context, query string, args ...any) (int64, error)

	// Commit はトランザクションをコミットする。
	Commit(ctx context.Context) error

	// Rollback はトランザクションをロールバックする（エラー / パニック時は defer で呼び出す）。
	Rollback(ctx context.Context) error
}

// DbClient は Relational Store / Single-leader の L1+ 抽象 interface を宣言する。
// PostgreSQL の full API を Library 独自語彙で表現する。
// OSS 型（pgxpool.Pool / pgx.Conn 等）を引数・戻り値に一切含まない。
// ctx に AuthContext が含まれることを強制する（RLS / GUC 設定のため必須）。
type DbClient interface {
	// Begin はトランザクションを開始して DbTx を返す。
	// ctx には AuthContext が伝播されている前提とする（RLS / GUC 設定に必須）。
	// 実装側は BEGIN 後に AuthContext.ToGucSetters() で GUC を SET LOCAL する。
	Begin(ctx context.Context, opts *DbTxOptions) (DbTx, error)

	// QueryRow は単一行クエリを autocommit モードで実行する（短命クエリ用）。
	// ctx には AuthContext が伝播されている前提とする（RLS 機能保証のため）。
	QueryRow(ctx context.Context, query string, args ...any) DbRow

	// Query は複数行クエリを autocommit モードで実行する（短命クエリ用）。
	// ctx には AuthContext が伝播されている前提とする（RLS 機能保証のため）。
	Query(ctx context.Context, query string, args ...any) (DbRows, error)

	// Exec は DML を autocommit モードで実行する（短命 DML 用）。
	// ctx には AuthContext が伝播されている前提とする（RLS 機能保証のため）。
	Exec(ctx context.Context, query string, args ...any) (int64, error)

	// InTx はトランザクション内でコールバック fn を実行する（COMMIT / ROLLBACK は自動管理）。
	// fn が error を返した場合は自動 ROLLBACK する。
	// fn が nil を返した場合は自動 COMMIT する。
	// ctx には AuthContext が伝播されている前提とする（RLS 機能保証のため）。
	InTx(ctx context.Context, opts *DbTxOptions, fn func(ctx context.Context, tx DbTx) error) error

	// Ping は DB への接続確認を行う（health check 用途）。
	Ping(ctx context.Context) error

	// Stats は接続プールの統計情報を返す（監視 / メトリクス用途）。
	Stats(ctx context.Context) *DbPoolStats
}

// DbPoolStats は接続プールの統計情報を宣言する型。
// OSS の pgxpool.Stat を露出せず Library 独自語彙で表現する。
type DbPoolStats struct {
	// TotalConnections: 接続プールの総接続数
	TotalConnections int32
	// IdleConnections: アイドル状態の接続数
	IdleConnections int32
	// AcquiredConnections: 使用中の接続数
	AcquiredConnections int32
	// WaitCount: 接続待ちリクエスト数
	WaitCount int64
	// MaxConnections: 接続プールの最大接続数設定値
	MaxConnections int32
}

// ErrDbNoRows は単一行クエリで行が見つからない場合のエラー。
// OSS の pgx.ErrNoRows / sql.ErrNoRows を Library 語彙でラップする。
// errors.Is(err, ErrDbNoRows) でチェックする。
var ErrDbNoRows = dbNoRowsSentinel{}

// dbNoRowsSentinel は ErrDbNoRows の sentinel 型（unexported: 外部からの直接生成を禁止する）。
type dbNoRowsSentinel struct{}

// Error は error interface を実装する。
func (dbNoRowsSentinel) Error() string {
	// 行が見つからないエラーメッセージを返す
	return "db: no rows in result set"
}

// Is は errors.Is でのマッチングをサポートする。
func (dbNoRowsSentinel) Is(target error) bool {
	// 同一型かどうかを確認する
	_, ok := target.(dbNoRowsSentinel)
	return ok
}
