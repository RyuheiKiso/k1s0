// concurrency_guard.go — aggregate_id 単位で max_one_in_flight を強制する ConcurrencyGuard（Go 等価強度実装）
// TypeScript の withAggregateExclusivity と等価の抽象を Go で実装する
// spec 11 §per-aggregate write 並行 1 件以下: layers.yaml invariant の max_one_in_flight_per_aggregate を物理化する
package state

import (
	// context パッケージ: キャンセル / タイムアウト制御に使用する
	"context"
	// errors パッケージ: エラー定義に使用する
	"errors"
	// sync パッケージ: sync.Map / sync.Mutex による concurrency 制御に使用する
	"sync"
)

// ErrAggregateInFlight は対象 aggregate が既に処理中であることを示すエラー
// withAggregateExclusivity と同様、先行処理の完了を待機するため通常は返されない
var ErrAggregateInFlight = errors.New("aggregate is already in flight")

// aggregateLock は aggregate 単位の排他制御に使用するエントリ
type aggregateLock struct {
	// aggregate 単位の Mutex（goroutine 間の排他制御に使用する）
	mu sync.Mutex
	// この lock を参照している goroutine 数（ゼロになったら sync.Map から削除する）
	refCount int
}

// ConcurrencyGuard は aggregate_id 単位で max_one_in_flight を実現するガード構造体
// TypeScript の inFlightMap に対応する aggregate 単位 Mutex マップを sync.Map で管理する
type ConcurrencyGuard struct {
	// aggregate_id → *aggregateLock のマップ（sync.Map で goroutine-safe に管理する）
	locks sync.Map
	// locks マップ自体の変更（refCount 操作）を保護する Mutex
	mu sync.Mutex
}

// WithAggregateExclusivity は aggregateID で指定した aggregate に対して fn を排他的に実行する
// 先行処理が存在する場合は先行処理の完了まで待機してから fn を実行する（TypeScript と等価）
// ctx: キャンセル / タイムアウトのコンテキスト
// aggregateID: 排他制御の対象 aggregate の識別子
// fn: aggregate に対して排他的に実行する処理
func (g *ConcurrencyGuard) WithAggregateExclusivity(ctx context.Context, aggregateID string, fn func() error) error {
	// aggregate 単位の lock エントリを取得または生成する
	entry := g.getOrCreateLock(aggregateID)
	// 取得した lock エントリの refCount をデクリメントして不要なら削除する（defer で確実に実行する）
	defer g.releaseLock(aggregateID, entry)
	// context のキャンセルチェック（lock 取得前に cancel 済みの場合は早期リターンする）
	select {
	// context がキャンセル済みの場合はエラーを返す
	case <-ctx.Done():
		// context エラーをそのまま返す（Deadline exceeded / Canceled）
		return ctx.Err()
	// context がキャンセル済みでない場合は続行する
	default:
	}
	// aggregate 単位の Mutex を lock する（先行処理が完了するまで待機する）
	// TypeScript の await current.catch(() => undefined) + inFlightMap.set に対応する
	entry.mu.Lock()
	// fn 完了後に Mutex を unlock する（次の待機者を解放する）
	defer entry.mu.Unlock()
	// context のキャンセルチェック（lock 取得後に cancel された場合も検出する）
	select {
	// context がキャンセル済みの場合はエラーを返す（Mutex unlock は defer で行う）
	case <-ctx.Done():
		// context エラーをそのまま返す
		return ctx.Err()
	// context がキャンセル済みでない場合は fn を実行する
	default:
	}
	// 排他的な処理を実行する（エラーはそのまま呼び出し元に返す）
	return fn()
}

// getOrCreateLock は aggregateID に対応する aggregateLock を取得または生成する
// refCount をインクリメントして lock エントリを保持する（releaseLock でデクリメントする）
func (g *ConcurrencyGuard) getOrCreateLock(aggregateID string) *aggregateLock {
	// locks マップの変更を保護する Mutex を lock する
	g.mu.Lock()
	// defer で Mutex を unlock する
	defer g.mu.Unlock()
	// 既存の lock エントリを取得する
	if val, ok := g.locks.Load(aggregateID); ok {
		// 既存エントリの refCount をインクリメントする
		entry := val.(*aggregateLock)
		// この goroutine がエントリを参照していることを記録する
		entry.refCount++
		// 既存の lock エントリを返す
		return entry
	}
	// 新規の lock エントリを生成する（refCount = 1 で初期化する）
	entry := &aggregateLock{refCount: 1}
	// sync.Map に登録する
	g.locks.Store(aggregateID, entry)
	// 新規の lock エントリを返す
	return entry
}

// releaseLock は aggregateID に対応する aggregateLock の refCount をデクリメントし
// refCount がゼロになった場合は sync.Map から削除してメモリリークを防止する
func (g *ConcurrencyGuard) releaseLock(aggregateID string, entry *aggregateLock) {
	// locks マップの変更を保護する Mutex を lock する
	g.mu.Lock()
	// defer で Mutex を unlock する
	defer g.mu.Unlock()
	// refCount をデクリメントする
	entry.refCount--
	// refCount がゼロになった場合は sync.Map からエントリを削除する
	if entry.refCount == 0 {
		// 不要なエントリを削除してメモリリークを防止する
		g.locks.Delete(aggregateID)
	}
}

// IsAggregateInFlight は aggregateID で指定した aggregate が現在 in-flight かどうかを返す
// TypeScript の isAggregateInFlight と等価の確認関数
func (g *ConcurrencyGuard) IsAggregateInFlight(aggregateID string) bool {
	// sync.Map に aggregateID のエントリが存在するか確認する
	_, ok := g.locks.Load(aggregateID)
	// エントリが存在する場合は in-flight（true）を返す
	return ok
}
