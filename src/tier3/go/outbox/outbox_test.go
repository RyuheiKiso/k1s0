// k1s0 tier3 Go outbox プロパティテスト
// outbox.go の各関数が期待通りの性質を持つことを検証する
package outbox

import (
	// strings パッケージ（文字列操作）
	"strings"
	// testing パッケージ（Go 標準テストフレームワーク）
	"testing"
	// time パッケージ（スリープ等に使用）
	"time"
)

// TestHlcNowFormat は HlcNow() が正しいフォーマットを返すことを確認する
func TestHlcNowFormat(t *testing.T) {
	// HLC タイムスタンプを生成する
	hlc := HlcNow()
	// ハイフン区切りで 3 部分に分かれることを確認する
	parts := strings.Split(hlc, "-")
	// パーツ数が 3 であることを確認する
	if len(parts) != 3 {
		// フォーマットが不正な場合はテスト失敗
		t.Errorf("HlcNow() フォーマット不正: got %q, parts=%d, want 3", hlc, len(parts))
	}
	// timestamp_ms_hex が 16 桁であることを確認する
	if len(parts[0]) != 16 {
		// 16 桁でない場合はテスト失敗
		t.Errorf("HlcNow() timestamp_ms_hex が 16 桁でない: got %q (len=%d)", parts[0], len(parts[0]))
	}
	// logical_counter が 4 桁であることを確認する
	if len(parts[1]) != 4 {
		// 4 桁でない場合はテスト失敗
		t.Errorf("HlcNow() logical_counter が 4 桁でない: got %q (len=%d)", parts[1], len(parts[1]))
	}
	// node_id が 4 桁であることを確認する
	if len(parts[2]) != 4 {
		// 4 桁でない場合はテスト失敗
		t.Errorf("HlcNow() node_id が 4 桁でない: got %q (len=%d)", parts[2], len(parts[2]))
	}
}

// TestHlcNowMonotonicity は HlcNow() が単調増加することを確認する
func TestHlcNowMonotonicity(t *testing.T) {
	// 最初の HLC タイムスタンプを取得する
	hlc1 := HlcNow()
	// 少し待機して次の HLC タイムスタンプを取得する
	time.Sleep(2 * time.Millisecond)
	// 2 回目の HLC タイムスタンプを取得する
	hlc2 := HlcNow()
	// 最初の HLC のミリ秒値を抽出する
	ms1 := extractMsFromHlc(hlc1)
	// 2 回目の HLC のミリ秒値を抽出する
	ms2 := extractMsFromHlc(hlc2)
	// ms2 >= ms1 であることを確認する（単調増加）
	if ms2 < ms1 {
		// 単調増加でない場合はテスト失敗
		t.Errorf("HlcNow() 単調増加違反: ms1=%d > ms2=%d", ms1, ms2)
	}
}

// TestGenerateIdempotencyKeyUniqueness は GenerateIdempotencyKey() が一意な key を生成することを確認する
func TestGenerateIdempotencyKeyUniqueness(t *testing.T) {
	// 100 回生成して全て異なることを確認する（プロパティテスト）
	keys := make(map[string]struct{}, 100)
	for i := 0; i < 100; i++ {
		// Idempotency-Key を生成する
		key := GenerateIdempotencyKey("aggregate1", "create")
		// 既に同じ key が生成されていた場合はテスト失敗
		if _, exists := keys[key]; exists {
			t.Errorf("GenerateIdempotencyKey() 重複 key: %q", key)
		}
		// 生成した key を記録する
		keys[key] = struct{}{}
	}
}

// TestGenerateIdempotencyKeyPrefix は GenerateIdempotencyKey() が正しい prefix を含むことを確認する
func TestGenerateIdempotencyKeyPrefix(t *testing.T) {
	// aggregateId の先頭 8 文字を prefix とした key が生成されることを確認する
	key := GenerateIdempotencyKey("aggregate123456789", "createMethod")
	// key が "aggregat_crea" で始まることを確認する
	if !strings.HasPrefix(key, "aggregat_crea") {
		// prefix が不正な場合はテスト失敗
		t.Errorf("GenerateIdempotencyKey() prefix 不正: got %q, want prefix 'aggregat_crea'", key)
	}
}

// TestCreateOutboxMetaExpiresAtMs は CreateOutboxMeta() の ExpiresAtMs が TTL 後になることを確認する
func TestCreateOutboxMetaExpiresAtMs(t *testing.T) {
	// メタデータを生成する
	meta := CreateOutboxMeta("agg-001", "create", "")
	// ExpiresAtMs が 0 より大きいことを確認する
	if meta.ExpiresAtMs <= 0 {
		// ExpiresAtMs が不正な場合はテスト失敗
		t.Errorf("CreateOutboxMeta() ExpiresAtMs <= 0: got %d", meta.ExpiresAtMs)
	}
	// EnqueuedAt から抽出したミリ秒 + TTL が ExpiresAtMs と一致することを確認する
	enqueuedMs := extractMsFromHlc(meta.EnqueuedAt)
	// 期待する ExpiresAtMs を計算する
	expectedExpires := enqueuedMs + IDEMPOTENCY_KEY_TTL_MS
	// 一致しない場合はテスト失敗
	if meta.ExpiresAtMs != expectedExpires {
		t.Errorf("CreateOutboxMeta() ExpiresAtMs 不一致: got %d, want %d", meta.ExpiresAtMs, expectedExpires)
	}
}

// TestCreateOutboxMetaEnqueuedAtHlcFormat は CreateOutboxMeta() の EnqueuedAt が HLC フォーマットであることを確認する
func TestCreateOutboxMetaEnqueuedAtHlcFormat(t *testing.T) {
	// メタデータを生成する
	meta := CreateOutboxMeta("agg-001", "create", "")
	// EnqueuedAt がハイフン区切りで 3 部分に分かれることを確認する
	parts := strings.Split(meta.EnqueuedAt, "-")
	// パーツ数が 3 であることを確認する
	if len(parts) != 3 {
		// フォーマットが不正な場合はテスト失敗
		t.Errorf("CreateOutboxMeta() EnqueuedAt フォーマット不正: got %q, parts=%d, want 3", meta.EnqueuedAt, len(parts))
	}
}

// TestIsExpiredFalseForFreshEntry は生成直後のエントリが TTL 超過でないことを確認する
func TestIsExpiredFalseForFreshEntry(t *testing.T) {
	// 生成直後のメタデータを作成する
	meta := CreateOutboxMeta("agg-001", "create", "")
	// 生成直後は TTL 超過でないことを確認する
	if IsExpired(meta) {
		// 生成直後に TTL 超過と判定された場合はテスト失敗
		t.Errorf("IsExpired() 生成直後のエントリが TTL 超過と判定された: EnqueuedAt=%q", meta.EnqueuedAt)
	}
}

// TestIsExpiredTrueForOldEntry は 25h 前のエントリが TTL 超過であることを確認する
func TestIsExpiredTrueForOldEntry(t *testing.T) {
	// 25h 前の HLC タイムスタンプを生成する（TTL=24h を超過させる）
	pastMs := time.Now().UnixMilli() - (25 * 60 * 60 * 1000)
	// 過去のタイムスタンプを HLC フォーマットに変換する
	pastHlc := hlcFromMs(pastMs)
	// 古いエントリのメタデータを手動で組み立てる
	meta := OutboxEntryMeta{
		// Idempotency-Key は任意の値を使用する
		IdempotencyKey: "test-key",
		// 25h 前の HLC タイムスタンプを設定する
		EnqueuedAt: pastHlc,
		// ExpiresAtMs は pastMs + TTL（過去の値）
		ExpiresAtMs: pastMs + IDEMPOTENCY_KEY_TTL_MS,
		// aggregate ID
		AggregateId: "agg-001",
		// RPC method 名
		RpcMethod: "create",
	}
	// 25h 前のエントリは TTL 超過と判定されることを確認する
	if !IsExpired(meta) {
		// TTL 超過でない場合はテスト失敗
		t.Errorf("IsExpired() 25h 前のエントリが TTL 超過と判定されなかった: EnqueuedAt=%q", meta.EnqueuedAt)
	}
}

// hlcFromMs は Unix ミリ秒値から HLC タイムスタンプ文字列を生成するヘルパー
// テスト専用のヘルパー関数（production コードには含めない）
func hlcFromMs(ms int64) string {
	// ミリ秒を 16 桁 hex にフォーマットする
	return strings.Join([]string{
		// 16 桁 hex タイムスタンプ
		strings.ToLower(strings.TrimLeft(func() string {
			const hex = "0123456789abcdef"
			b := make([]byte, 16)
			v := uint64(ms)
			for i := 15; i >= 0; i-- {
				b[i] = hex[v&0xf]
				v >>= 4
			}
			return string(b)
		}(), "")),
		// logical_counter（テスト用固定値）
		"0000",
		// node_id（テスト用固定値）
		"0000",
	}, "-")
}

// TestStripPiiFields は StripPiiFields() が PII フィールドを正しく除去することを確認する
func TestStripPiiFields(t *testing.T) {
	// テスト用 payload を作成する
	payload := map[string]any{
		// PII フィールド（除去対象）
		"email": "test@example.com",
		// PII フィールド（除去対象）
		"name": "テストユーザー",
		// 非 PII フィールド（保持対象）
		"order_id": "ORD-001",
		// 非 PII フィールド（保持対象）
		"amount": 1000,
	}
	// PII フィールド名リスト
	piiFields := []string{"email", "name"}
	// PII strip を実行する
	stripped := StripPiiFields(payload, piiFields)
	// email が除去されていることを確認する
	if _, exists := stripped["email"]; exists {
		t.Errorf("StripPiiFields() email が除去されていない")
	}
	// name が除去されていることを確認する
	if _, exists := stripped["name"]; exists {
		t.Errorf("StripPiiFields() name が除去されていない")
	}
	// order_id が保持されていることを確認する
	if _, exists := stripped["order_id"]; !exists {
		t.Errorf("StripPiiFields() order_id が除去されてしまった")
	}
	// amount が保持されていることを確認する
	if _, exists := stripped["amount"]; !exists {
		t.Errorf("StripPiiFields() amount が除去されてしまった")
	}
}

// TestCreateOutboxMetaWithChainedFrom は ChainedFrom が設定される場合のメタデータ生成を確認する
func TestCreateOutboxMetaWithChainedFrom(t *testing.T) {
	// chain 元 key を設定してメタデータを生成する
	meta := CreateOutboxMeta("agg-001", "update", "original-key-001")
	// ChainedFrom が設定されていることを確認する
	if meta.ChainedFrom != "original-key-001" {
		// ChainedFrom が不正な場合はテスト失敗
		t.Errorf("CreateOutboxMeta() ChainedFrom 不正: got %q, want 'original-key-001'", meta.ChainedFrom)
	}
	// IdempotencyKey が ChainedFrom と異なることを確認する（新しい key が生成された）
	if meta.IdempotencyKey == meta.ChainedFrom {
		// 同じ key が生成された場合はテスト失敗
		t.Errorf("CreateOutboxMeta() IdempotencyKey が ChainedFrom と同じ: %q", meta.IdempotencyKey)
	}
}
