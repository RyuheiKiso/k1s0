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
	// strings パッケージ（文字列操作）
	"strings"
	// k1s0-hlc: wall-clock TTL 禁止規律に従い HLC（Hybrid Logical Clock）を使用する
	// src/CLAUDE.md §wall-clock TTL 禁止: time.Now() は HLC lib 内部のみ許可
	hlc "github.com/k1s0/hlc-lib-go"
)

// IDEMPOTENCY_KEY_TTL_MS は Idempotency-Key の 24h TTL（ミリ秒）
const IDEMPOTENCY_KEY_TTL_MS int64 = 24 * 60 * 60 * 1000

// globalHlcClock はプロセス全体で共有する HLC クロック（スレッドセーフ）
// src/CLAUDE.md §wall-clock TTL 禁止: time.Now() は HLC lib 内部のみ許可
var globalHlcClock *hlc.HlcClock

// init は package 初期化時に HLC クロックを生成する
func init() {
	// 環境変数 HLC_NODE_ID から node_id を取得して HLC クロックを生成する
	globalHlcClock = hlc.NewHlcClockFromEnv()
}

// HlcNow は共有 HLC クロックから現在のタイムスタンプを生成して文字列に変換する
// wall-clock TTL 禁止規約に従い HLC lib（k1s0-hlc）のみが time.Now() を呼ぶ
func HlcNow() string {
	// k1s0-hlc のグローバルクロックから Tick して HlcTimestamp を取得する
	ts := globalHlcClock.Tick()
	// HLC タイムスタンプを compact 文字列（"{wall_ms_hex_16}-{logical_04x}-{node_04x}"）に変換する
	return ts.FormatCompact()
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
	// uint64 から int64 に変換して返す（bits.RotateLeft64(x,0) は恒等変換のため直接変換する）
	return int64(ms)
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

// GenerateIdempotencyKey は Idempotency-Key を生成する
// フォーマット: "{tenantId}_{ulidHex}_{methodHash}" — docs/04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md §idempotency_key
// tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由）
// aggregateId: 集約識別子（方法 hash の一部に使用する）
// rpcMethod: RPC method 名（short hash の生成に使用する）
func GenerateIdempotencyKey(tenantId, aggregateId, rpcMethod string) string {
	// ULID 相当の識別子を HLC ベースで生成する（timestamp_ms_hex + random）
	hlcBase := strings.SplitN(HlcNow(), "-", 2)[0]
	// 暗号論的乱数バイト列を 8 バイト生成する（ULID のランダム部分）
	randBytes := make([]byte, 8)
	// crypto/rand で乱数を生成する（wall-clock 非依存）
	if _, err := rand.Read(randBytes); err != nil {
		// 乱数生成に失敗した場合は panic する（起動時の致命的エラー）
		panic(fmt.Sprintf("GenerateIdempotencyKey: crypto/rand.Read failed: %v", err))
	}
	// バイト列を hex 文字列に変換する
	randHex := hex.EncodeToString(randBytes)
	// ULID 相当: HLC タイムスタンプ hex + random hex で 24 文字の識別子を生成する
	ulidHex := hlcBase + randHex[:8]
	// rpcMethod の先頭 4 文字を method hash として使用する（短縮識別子）
	methodHash := rpcMethod
	if len(methodHash) > 4 {
		methodHash = methodHash[:4]
	}
	// tenantId prefix + ulid + method hash の形式で Idempotency-Key を組み立てる
	// フォーマット: "{tenantId}_{ulidHex}_{methodHash}" — spec §idempotency_key 準拠
	return strings.Join([]string{tenantId, ulidHex, methodHash}, "_")
}

// CreateOutboxMeta は Outbox エントリのメタデータを生成する
// tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
// wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
func CreateOutboxMeta(tenantId, aggregateId, rpcMethod, chainedFrom string) OutboxEntryMeta {
	// HLC タイムスタンプを現在時刻として取得する
	nowHlc := HlcNow()
	// enqueue 時刻（ミリ秒）を HLC から抽出する
	enqueuedMs := extractMsFromHlc(nowHlc)
	// Idempotency-Key を生成する（tenantId prefix + ULID + methodHash — spec §idempotency_key 準拠）
	key := GenerateIdempotencyKey(tenantId, aggregateId, rpcMethod)
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
