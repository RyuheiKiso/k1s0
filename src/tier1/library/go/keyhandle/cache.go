// cache.go — k1s0 tier1 Library Go 実装: KeyValue / Cache の L3 interface
// 08_キャッシュ適合仕様.md §CacheClient（OSS 中立 L3）に準拠する。
// Redis / Memcached 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// wall-clock TTL 禁止規約に準拠して HLC Duration のみを TTL として受け付ける。

// パッケージ名: keyhandle（tier1 Library の KeyValue / Cache API を提供する）
package keyhandle

import (
	// context: context.Context（非同期操作に使用する）
	"context"
)

// CacheTTL は HLC ベースの TTL を宣言する型。
// wall-clock TTL 禁止規約に準拠して論理クロック差分 (logical ticks) で表現する。
// 実装側は HLC の hlc_lib を参照して変換する（SystemTime::now() 等の直接使用を禁止する）。
type CacheTTL struct {
	// LogicalTicks: HLC 論理クロック差分（tick 単位）
	// 1 tick = 実装固有の物理時間（通常はマイクロ秒単位）
	LogicalTicks uint64
}

// CacheSetOptions は CacheClient.Set / CacheClient.SetNX に渡すオプションを宣言する型。
type CacheSetOptions struct {
	// TTL: HLC ベースのキャッシュ有効期限（nil = 無期限キャッシュ）
	// wall-clock TTL 禁止規約に準拠して HLC Duration のみを許容する。
	TTL *CacheTTL
	// IfNotExists: true の場合はキーが存在しない場合のみ設定する（SET NX 相当）
	IfNotExists bool
	// IfExists: true の場合はキーが既に存在する場合のみ更新する（SET XX 相当）
	IfExists bool
}

// CacheScanCursor はキャッシュスキャン（SCAN 相当）のカーソルを宣言する型。
// OSS の redis scan cursor 型を露出せず Library 独自語彙で表現する。
type CacheScanCursor struct {
	// Cursor: スキャン位置カーソル（0 = スキャン開始 / 終了）
	Cursor uint64
	// Done: スキャンが完了したかどうか（Cursor==0 かつ Done==true で全スキャン完了）
	Done bool
}

// CacheClient は KeyValue / Cache の L3 抽象 interface を宣言する。
// Redis / Memcached / DragonflyDB 等 OSS を透過的に切り替え可能にする。
// OSS 型（redis.Client / memcache.Client 等）を引数・戻り値に一切含まない。
type CacheClient interface {
	// Get はキーに対応する値を返す。
	// キーが存在しない場合は nil, nil を返す（エラーと区別する）。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Get(ctx context.Context, tenantID string, key string) ([]byte, error)

	// Set はキーと値のペアをキャッシュに保存する。
	// opts == nil の場合は無期限キャッシュとして保存する。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Set(ctx context.Context, tenantID string, key string, value []byte, opts *CacheSetOptions) error

	// Delete はキーをキャッシュから削除する。
	// キーが存在しない場合はエラーを返さない（idempotent 操作）。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Delete(ctx context.Context, tenantID string, key string) error

	// Exists はキーが存在するかどうかを返す。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Exists(ctx context.Context, tenantID string, key string) (bool, error)

	// GetMany は複数キーの値を一括取得する（MGET 相当）。
	// 戻り値は keys と同順の値スライスで、存在しないキーは nil とする。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	GetMany(ctx context.Context, tenantID string, keys []string) ([][]byte, error)

	// SetMany は複数キーと値のペアを一括保存する（MSET 相当）。
	// pairs はキーと値のペアマップ、opts は全ペアに適用する共通オプション。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	SetMany(ctx context.Context, tenantID string, pairs map[string][]byte, opts *CacheSetOptions) error

	// DeleteMany は複数キーを一括削除する（DEL 相当）。
	// 存在しないキーは無視する（idempotent 操作）。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	DeleteMany(ctx context.Context, tenantID string, keys []string) error

	// Increment はキーの数値を delta だけアトミックに加算して新しい値を返す。
	// キーが存在しない場合は 0 を初期値として delta を加算する（INCRBY 相当）。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Increment(ctx context.Context, tenantID string, key string, delta int64) (int64, error)
}

// CacheLockOptions は分散ロック（RedLock / Fencing Token）のオプションを宣言する型。
// wall-clock TTL 禁止規約に準拠して HLC ベースのロック有効期限のみを受け付ける。
type CacheLockOptions struct {
	// TTL: HLC ベースのロック有効期限（必須: 無期限ロックは禁止する）
	TTL CacheTTL
	// RetryCount: ロック取得リトライ回数（0 = リトライなし）
	RetryCount int
	// FencingToken: Fencing Token 機能を有効にするかどうか（防止 stale write）
	FencingToken bool
}

// CacheLock はキャッシュ分散ロックの L3 抽象 interface を宣言する。
// RedLock / Redisson / etcd lease 等を隠蔽する。
type CacheLock interface {
	// Acquire はロックを取得する（取得できなかった場合はエラーを返す）。
	// name はロック名（tenantID 内で一意な識別子）。
	// tenantID を prefix とした名前空間で tenant 分離を保証する。
	Acquire(ctx context.Context, tenantID string, name string, opts CacheLockOptions) (string, error)

	// Release はロックを解放する（token は Acquire で返されたトークン）。
	// token が不一致の場合はロックを解放せずにエラーを返す（Fencing Token 保護）。
	Release(ctx context.Context, tenantID string, name string, token string) error

	// Refresh はロックの有効期限を延長する（long-running 処理用）。
	// token は Acquire で返されたトークン、opts.TTL は延長後の新しい有効期限。
	Refresh(ctx context.Context, tenantID string, name string, token string, opts CacheLockOptions) error
}
