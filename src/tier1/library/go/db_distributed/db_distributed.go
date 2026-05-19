// db_distributed.go — k1s0 tier1 Library Go 実装: Relational Store / Distributed SQL の L1+ interface
// 14_分散SQL適合仕様.md §DistributedDbClient（CockroachDB / Spanner L1+ 深耕）に準拠する。
// Distributed SQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（pgxpool / spanner 等）を公開シグネチャに一切含まない。

// パッケージ名: db_distributed（tier1 Library の Distributed SQL API を提供する）
package db_distributed

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
	// db パッケージ: DbRow / DbRows / DbTx / DbTxIsoLevel を参照する
	"github.com/k1s0-io/k1s0/tier1/library/db"
)

// DistributedTxPriority はトランザクションの実行優先度を宣言する型。
// CockroachDB の transaction priority に準拠した Library 独自語彙とする。
type DistributedTxPriority string

const (
	// DistributedTxPriorityNormal: 通常優先度（デフォルト）
	DistributedTxPriorityNormal DistributedTxPriority = "normal"
	// DistributedTxPriorityHigh: 高優先度（コンテンション時に優先してコミットする）
	DistributedTxPriorityHigh DistributedTxPriority = "high"
	// DistributedTxPriorityLow: 低優先度（バックグラウンドジョブに使用する）
	DistributedTxPriorityLow DistributedTxPriority = "low"
)

// DistributedTxOptions は Distributed SQL のトランザクション開始オプションを宣言する型。
// DbTxOptions を継承しつつ、Distributed SQL 固有オプションを追加する。
type DistributedTxOptions struct {
	// IsoLevel: トランザクション分離レベル（Distributed SQL では Serializable 推奨）
	IsoLevel db.DbTxIsoLevel
	// ReadOnly: 読み取り専用トランザクションかどうか
	ReadOnly bool
	// Priority: Distributed SQL のトランザクション優先度
	Priority DistributedTxPriority
	// AsOfSystemTime: 過去の特定タイムスタンプを読む（AOST: CockroachDB 固有機能）
	// wall-clock TTL 禁止規約に準拠して HLC tick 値で指定する（0 = 最新を読む）。
	AsOfSystemTimeTick uint64
	// MaxRetries: 自動リトライ回数（楽観的ロック競合時に自動リトライする）
	MaxRetries int
}

// DistributedDbTx は Distributed SQL のアクティブなトランザクションを宣言する interface。
// DbTx を継承しつつ、Distributed SQL 固有メソッドを追加する。
type DistributedDbTx interface {
	// DbTx の全メソッドを継承する（QueryRow / Query / Exec / Commit / Rollback）
	db.DbTx

	// Savepoint は名前付きセーブポイントを作成する（ネストトランザクション用）。
	// name はセーブポイント名（英数字 + アンダースコアのみ許容する）。
	Savepoint(ctx context.Context, name string) error

	// RollbackToSavepoint はセーブポイントにロールバックする（部分ロールバック）。
	RollbackToSavepoint(ctx context.Context, name string) error

	// ReleaseSavepoint はセーブポイントをリリースする（セーブポイントを確定する）。
	ReleaseSavepoint(ctx context.Context, name string) error

	// Priority は現在のトランザクション優先度を返す。
	Priority() DistributedTxPriority
}

// DistributedDbClient は Relational Store / Distributed SQL の L1+ 抽象 interface を宣言する。
// CockroachDB / Google Cloud Spanner / YugabyteDB 等を抽象化する。
// OSS 型（pgxpool.Pool / spanner.Client 等）を引数・戻り値に一切含まない。
// 単一ノード PostgreSQL と区別するために DbClient を継承しない（別 interface として宣言する）。
type DistributedDbClient interface {
	// Begin はトランザクションを開始して DistributedDbTx を返す。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離に必須）。
	// opts は Distributed SQL 固有オプション（priority / AOST 等）。
	Begin(ctx context.Context, opts *DistributedTxOptions) (DistributedDbTx, error)

	// QueryRow は単一行クエリを autocommit モードで実行する。
	// ctx には AuthContext が伝播されている前提とする。
	QueryRow(ctx context.Context, query string, args ...any) db.DbRow

	// Query は複数行クエリを autocommit モードで実行する。
	// ctx には AuthContext が伝播されている前提とする。
	Query(ctx context.Context, query string, args ...any) (db.DbRows, error)

	// Exec は DML を autocommit モードで実行する。
	// ctx には AuthContext が伝播されている前提とする。
	Exec(ctx context.Context, query string, args ...any) (int64, error)

	// InTx はトランザクション内でコールバック fn を実行する（自動リトライ + 自動 COMMIT/ROLLBACK）。
	// fn が error を返した場合は自動 ROLLBACK 後に opts.MaxRetries まで自動リトライする。
	// fn が nil を返した場合は自動 COMMIT する。
	InTx(ctx context.Context, opts *DistributedTxOptions, fn func(ctx context.Context, tx DistributedDbTx) error) error

	// Ping は DB への接続確認を行う（health check 用途）。
	Ping(ctx context.Context) error

	// NodeInfo は接続先 Distributed SQL クラスターのノード情報を返す（診断用）。
	NodeInfo(ctx context.Context) (*DistributedDbNodeInfo, error)
}

// DistributedDbNodeInfo は Distributed SQL クラスターのノード情報を宣言する型。
// CockroachDB のクラスター情報を Library 独自語彙で表現する。
type DistributedDbNodeInfo struct {
	// NodeCount: クラスター内のノード数
	NodeCount int
	// Region: 接続先リージョン名（CockroachDB の locality 設定）
	Region string
	// Version: DB バージョン文字列
	Version string
	// IsReadOnly: 接続先ノードが読み取り専用かどうか（リードレプリカ接続時）
	IsReadOnly bool
}

// BulkInsertOptions は Distributed SQL への大量データ一括挿入オプションを宣言する型。
// CockroachDB の IMPORT / COPY 機能を Library 独自語彙で抽象化する。
type BulkInsertOptions struct {
	// BatchSize: 1 バッチあたりの行数（大きすぎるとトランザクションタイムアウトになる）
	BatchSize int
	// OnConflict: 主キー競合時の処理（"update" / "ignore" / "error"）
	OnConflict string
	// ReturnInserted: 挿入した行のデータを戻り値に含めるかどうか（RETURNING 句相当）
	ReturnInserted bool
}

// DistributedBulkClient は Distributed SQL への大量データ挿入に特化した interface を宣言する。
// COPY / IMPORT 等のバルク操作を Library 独自語彙で抽象化する。
type DistributedBulkClient interface {
	// BulkInsert は複数行を一括挿入する（バッチ分割は自動で行う）。
	// table は挿入先テーブル名（スキーマ付きの完全修飾名を推奨する）。
	// columns は挿入するカラム名スライス。
	// rows はカラム順に対応する値スライスのスライス（len(row) == len(columns) を保証する）。
	// 戻り値は挿入した行数。
	BulkInsert(ctx context.Context, table string, columns []string, rows [][]any, opts *BulkInsertOptions) (int64, error)

	// BulkUpsert は複数行を一括 UPSERT する（INSERT ON CONFLICT DO UPDATE 相当）。
	// conflictColumns は競合判定に使用するカラム名スライス（主キー / ユニークキー）。
	BulkUpsert(ctx context.Context, table string, columns []string, rows [][]any, conflictColumns []string, opts *BulkInsertOptions) (int64, error)
}
