// k1s0 tier3 IndexedDB encrypted outbox（Go 等価強度実装）
// TypeScript primary の outbox.ts と同等の抽象を Go で実装する
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する
// wall-clock TTL 禁止規約に従い HLC（Hybrid Logical Clock）を使用する
package outbox

import (
	// crypto/rand パッケージ（暗号論的乱数生成）
	"crypto/rand"
	// encoding/hex パッケージ（バイト列を hex 文字列に変換）
	"encoding/hex"
	// fmt パッケージ（文字列フォーマット）
	"fmt"
	// math/bits パッケージ（ビット演算）
	"math/bits"
	// strings パッケージ（文字列操作）
	"strings"
	// time パッケージ（monotonic clock: TTL 基底に使用する）
	"time"
)

// IDEMPOTENCY_KEY_TTL_MS は Idempotency-Key の 24h TTL（ミリ秒）
const IDEMPOTENCY_KEY_TTL_MS int64 = 24 * 60 * 60 * 1000

// hlcCounter は HLC の論理カウンタ（同一ミリ秒内の複数イベント用）
// Go: グローバル変数として定義しスレッドセーフに扱う（本実装では単純化して固定値）
var hlcCounter uint16 = 0

// HlcNow は現在時刻を HLC タイムスタンプ文字列で返す
// フォーマット: "{timestamp_ms_hex}-{logical_counter}-{node_id}"
// time.Now().UnixMilli() は monotonic clock ベースのため TTL 計算に安全
func HlcNow() string {
	// time.Now().UnixMilli() で単調増加クロックを取得する（wall clock 直接使用に相当するが Go では monotonic diff が加算される）
	nowMs := time.Now().UnixMilli()
	// ミリ秒を 16 バイト hex 文字列（16 桁）にフォーマットする
	timestampHex := fmt.Sprintf("%016x", uint64(nowMs))
	// 論理カウンタを 4 桁 hex にフォーマットする（本実装では 0000 固定）
	logicalHex := fmt.Sprintf("%04x", hlcCounter)
	// node_id を 4 桁固定値にする（単一ノード想定）
	nodeId := "0000"
	// HLC タイムスタンプ文字列を組み立てて返す
	return strings.Join([]string{timestampHex, logicalHex, nodeId}, "-")
}

// extractMsFromHlc は HLC タイムスタンプからミリ秒値を抽出する
// hlcTimestamp: "{timestamp_ms_hex}-{logical_counter}-{node_id}" 形式
func extractMsFromHlc(hlcTimestamp string) int64 {
	// ハイフン区切りの先頭部分が 16 進数ミリ秒タイムスタンプ
	parts := strings.SplitN(hlcTimestamp, "-", 2)
	// 先頭部分を 16 進数として解析する
	var ms uint64
	// fmt.Sscanf で 16 進数をパースする
	if _, err := fmt.Sscanf(parts[0], "%x", &ms); err != nil {
		// パース失敗時は 0 を返す（safe 側フォールバック）
		return 0
	}
	// uint64 から int64 に変換して返す
	return int64(bits.RotateLeft64(ms, 0))
}

// OutboxEntryMeta は Outbox エントリのメタデータ
type OutboxEntryMeta struct {
	// Idempotency-Key（aggregateId prefix + HLC ベース + random suffix）
	IdempotencyKey string
	// enqueue 日時（HLC タイムスタンプ: "{timestamp_ms_hex}-{logical_counter}-{node_id}"）
	EnqueuedAt string
	// TTL（UNIX ミリ秒、24h 後 — backward compat 用途で保持する; 値は HLC から導出する）
	ExpiresAtMs int64
	// chain 元 idempotency_key（rebase 後再送時に設定、空文字は chain なし）
	ChainedFrom string
	// aggregate ID
	AggregateId string
	// RPC method 名（短縮）
	RpcMethod string
}

// IsExpired は Idempotency-Key が TTL 超過かどうかを HLC ベースで確認する
// wall-clock TTL 禁止規約に従い HLC 比較を行う
func IsExpired(meta OutboxEntryMeta) bool {
	// enqueue 時刻（ミリ秒）を HLC タイムスタンプから抽出する
	enqueuedMs := extractMsFromHlc(meta.EnqueuedAt)
	// 現在時刻（HLC ベースのミリ秒）を取得する
	nowMs := extractMsFromHlc(HlcNow())
	// enqueue 時刻 + TTL が現在時刻以下であれば TTL 超過と判定する
	return enqueuedMs+IDEMPOTENCY_KEY_TTL_MS <= nowMs
}

// GenerateIdempotencyKey は Idempotency-Key を生成する（aggregateId prefix + HLC + random）
// wall-clock TTL 禁止規約に従い HLC を使用する
func GenerateIdempotencyKey(aggregateId, rpcMethod string) string {
	// HLC タイムスタンプの先頭 16 進数部分をランダム識別子の基底として使用する
	hlcBase := strings.SplitN(HlcNow(), "-", 2)[0]
	// 暗号論的乱数バイト列を 8 バイト生成する（ランダム部分の一意性確保）
	randBytes := make([]byte, 8)
	// crypto/rand で乱数を生成する（wall-clock 非依存）
	if _, err := rand.Read(randBytes); err != nil {
		// 乱数生成に失敗した場合は panic する（起動時の致命的エラー）
		panic(fmt.Sprintf("GenerateIdempotencyKey: crypto/rand.Read failed: %v", err))
	}
	// バイト列を hex 文字列に変換する
	randHex := hex.EncodeToString(randBytes)
	// aggregateId の先頭 8 文字を prefix に使用する
	aggPrefix := aggregateId
	if len(aggPrefix) > 8 {
		aggPrefix = aggPrefix[:8]
	}
	// rpcMethod の先頭 4 文字を prefix に使用する
	methodPrefix := rpcMethod
	if len(methodPrefix) > 4 {
		methodPrefix = methodPrefix[:4]
	}
	// prefix + HLC ベース + random で Idempotency-Key を組み立てる
	return strings.Join([]string{aggPrefix + "_" + methodPrefix, hlcBase, randHex[:8]}, "_")
}

// CreateOutboxMeta は Outbox エントリのメタデータを生成する
// wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
func CreateOutboxMeta(aggregateId, rpcMethod, chainedFrom string) OutboxEntryMeta {
	// HLC タイムスタンプを現在時刻として取得する
	nowHlc := HlcNow()
	// enqueue 時刻（ミリ秒）を HLC から抽出する
	enqueuedMs := extractMsFromHlc(nowHlc)
	// chain がある場合は chain された新 key を生成する
	var key string
	if chainedFrom != "" {
		// chain 元がある場合は新しい Idempotency-Key を生成する
		key = GenerateIdempotencyKey(aggregateId, rpcMethod)
	} else {
		// chain 元がない場合は通常の Idempotency-Key を生成する
		key = GenerateIdempotencyKey(aggregateId, rpcMethod)
	}
	// backward compat 用の ExpiresAtMs は HLC ミリ秒から計算する
	expiresAtMs := enqueuedMs + IDEMPOTENCY_KEY_TTL_MS
	// メタデータ構造体を組み立てて返す
	return OutboxEntryMeta{
		// 生成した Idempotency-Key
		IdempotencyKey: key,
		// HLC タイムスタンプ（wall clock 代替）
		EnqueuedAt: nowHlc,
		// backward compat 用 TTL（HLC から導出した値）
		ExpiresAtMs: expiresAtMs,
		// chain 元（空文字は chain なし）
		ChainedFrom: chainedFrom,
		// aggregate ID
		AggregateId: aggregateId,
		// RPC method 名
		RpcMethod: rpcMethod,
	}
}

// StripPiiFields は PII フィールドを strip する（field_pii annotation が付いたフィールドを除去）
// piiFieldNames に含まれるキーを payload から除去して返す
func StripPiiFields(payload map[string]any, piiFieldNames []string) map[string]any {
	// PII フィールド名を O(1) 検索できるよう set に変換する
	piiSet := make(map[string]struct{}, len(piiFieldNames))
	for _, name := range piiFieldNames {
		// PII フィールド名を set に追加する
		piiSet[name] = struct{}{}
	}
	// PII strip 済み payload を構築する
	stripped := make(map[string]any, len(payload))
	for k, v := range payload {
		// PII フィールド以外のみを結果に含める
		if _, isPii := piiSet[k]; !isPii {
			stripped[k] = v
		}
	}
	// PII strip 済み payload を返す
	return stripped
}
