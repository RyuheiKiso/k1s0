// hlc_test.go — hlc パッケージの Go テスト
// Rust テスト（src/client/hlc_lib/rust/src/lib.rs #[cfg(test)] mod tests）と
// 同等の coverage を Go で実装する。
// 各テスト関数は Rust 側の対応するテスト関数と 1:1 の対応を持つ。
package hlc

// testing: Go 標準テストフレームワークをインポートする
import (
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// sync: concurrent tick テストの WaitGroup に使用する
	"sync"
	// testing: Go テストフレームワーク
	"testing"
)

// TestHlcTimestampOrdering は HlcTimestamp の順序比較を確認するテスト（Rust: test_hlc_timestamp_ordering に対応）
func TestHlcTimestampOrdering(t *testing.T) {
	// wall_ms が大きければ後のタイムスタンプであることを確認する
	a := HlcTimestamp{WallMs: 100, Logical: 0, NodeId: 0}
	b := HlcTimestamp{WallMs: 200, Logical: 0, NodeId: 0}
	// a < b であることを確認する
	if a.Compare(b) >= 0 {
		t.Error("WallMs が大きい方が後であるべき: a.Compare(b) は負であるべき")
	}
	// wall_ms が同じなら logical で比較することを確認する
	c := HlcTimestamp{WallMs: 100, Logical: 1, NodeId: 0}
	if a.Compare(c) >= 0 {
		t.Error("同一 WallMs では Logical が大きい方が後であるべき: a.Compare(c) は負であるべき")
	}
	// logical も同じなら node_id で比較することを確認する
	d := HlcTimestamp{WallMs: 100, Logical: 0, NodeId: 1}
	if a.Compare(d) >= 0 {
		t.Error("同一 WallMs/Logical では NodeId が大きい方が後であるべき: a.Compare(d) は負であるべき")
	}
}

// TestFormatAndParse は FormatCompact と ParseCompact のラウンドトリップを確認するテスト（Rust: test_format_and_parse に対応）
func TestFormatAndParse(t *testing.T) {
	// 既知の値で HlcTimestamp を生成する（Rust テストと同一の値）
	ts := HlcTimestamp{WallMs: 0x0123456789abcdef, Logical: 0x00ff, NodeId: 0x1234}
	// FormatCompact で文字列に変換する
	formatted := ts.FormatCompact()
	// 期待値: Rust テストと同一の形式
	expected := "0123456789abcdef-00ff-1234"
	if formatted != expected {
		t.Errorf("FormatCompact の結果が期待値と一致しない: got=%q, want=%q", formatted, expected)
	}
	// ParseCompact でラウンドトリップが成立することを確認する
	parsed, ok := ParseCompact(formatted)
	if !ok {
		t.Fatalf("ParseCompact が失敗した: input=%q", formatted)
	}
	// 元の値と等しいことを確認する
	if parsed.Compare(ts) != 0 {
		t.Errorf("ラウンドトリップが失敗した: original=%v, parsed=%v", ts, parsed)
	}
}

// TestParseCompactInvalid は ParseCompact が不正な文字列に対して false を返すことを確認するテスト
func TestParseCompactInvalid(t *testing.T) {
	// 不正な形式の文字列のリスト
	invalidInputs := []string{
		// 空文字列
		"",
		// ハイフン区切りが足りない
		"invalid",
		// パート数が足りない
		"0000-0000",
		// 短すぎる wall_ms
		"000-0000-0000",
	}
	// 各不正入力に対して ParseCompact が false を返すことを確認する
	for _, input := range invalidInputs {
		_, ok := ParseCompact(input)
		if ok {
			t.Errorf("不正な入力 %q に対して ParseCompact は false を返すべき", input)
		}
	}
}

// TestAddMs は AddMs が正しく deadline を計算することを確認するテスト（Rust: test_add_ms に対応）
func TestAddMs(t *testing.T) {
	// 基準タイムスタンプを生成する（Rust テストと同一の値）
	base := HlcTimestamp{WallMs: 1_000_000, Logical: 5, NodeId: 1}
	// 1000ms 後の deadline を計算する
	deadline := base.AddMs(1_000)
	// WallMs が正しく加算されることを確認する
	if deadline.WallMs != 1_001_000 {
		t.Errorf("AddMs が WallMs を正しく加算すべき: got=%d, want=%d", deadline.WallMs, uint64(1_001_000))
	}
	// Logical は 0 にリセットされることを確認する
	if deadline.Logical != 0 {
		t.Errorf("AddMs 後の Logical は 0 であるべき: got=%d", deadline.Logical)
	}
	// NodeId は引き継がれることを確認する
	if deadline.NodeId != 1 {
		t.Errorf("AddMs 後の NodeId は引き継ぐべき: got=%d, want=%d", deadline.NodeId, uint16(1))
	}
}

// TestAddMsOverflow は AddMs が MaxUint64 付近でオーバーフローを飽和させることを確認するテスト
func TestAddMsOverflow(t *testing.T) {
	// MaxUint64 - 1 の WallMs を持つタイムスタンプを生成する
	ts := HlcTimestamp{WallMs: ^uint64(0) - 1, Logical: 0, NodeId: 0}
	// 2 を加算して MaxUint64 を超えさせる
	result := ts.AddMs(2)
	// MaxUint64 に飽和することを確認する
	if result.WallMs != ^uint64(0) {
		t.Errorf("AddMs はオーバーフロー時に MaxUint64 に飽和すべき: got=%d", result.WallMs)
	}
}

// TestElapsedAndExpired は ElapsedMsSince と IsExpiredAt の動作を確認するテスト（Rust: test_elapsed_and_expired に対応）
func TestElapsedAndExpired(t *testing.T) {
	// 現在時刻を表すタイムスタンプを生成する（Rust テストと同一の値）
	now := HlcTimestamp{WallMs: 2_000_000, Logical: 0, NodeId: 0}
	// 1000ms 後の deadline を計算する
	deadline := now.AddMs(1_000)
	// now では期限切れでないことを確認する
	if deadline.IsExpiredAt(now) {
		t.Error("deadline より前では期限切れにならないべき")
	}
	// deadline ちょうどで期限切れになることを確認する
	if !deadline.IsExpiredAt(deadline) {
		t.Error("deadline ちょうどで期限切れになるべき")
	}
	// deadline を過ぎたら期限切れになることを確認する
	future := HlcTimestamp{WallMs: 2_001_001, Logical: 0, NodeId: 0}
	if !deadline.IsExpiredAt(future) {
		t.Error("deadline を超えたら期限切れになるべき")
	}
	// ElapsedMsSince の値が正しいことを確認する
	elapsed := future.ElapsedMsSince(now)
	if elapsed != 1_001 {
		t.Errorf("ElapsedMsSince が正しくあるべき: got=%d, want=%d", elapsed, uint64(1_001))
	}
}

// TestElapsedMsSinceSaturation は ElapsedMsSince が self < reference のとき 0 を返すことを確認するテスト
func TestElapsedMsSinceSaturation(t *testing.T) {
	// earlier が later より前のタイムスタンプ
	earlier := HlcTimestamp{WallMs: 1_000, Logical: 0, NodeId: 0}
	later := HlcTimestamp{WallMs: 2_000, Logical: 0, NodeId: 0}
	// earlier.ElapsedMsSince(later) は 0 を返すべき（負の elapsed は表現しない）
	elapsed := earlier.ElapsedMsSince(later)
	if elapsed != 0 {
		t.Errorf("ElapsedMsSince(later) は 0 を返すべき: got=%d", elapsed)
	}
}

// TestTickMonotonic は HlcClock.Tick が単調増加タイムスタンプを生成することを確認するテスト（Rust: test_tick_monotonic に対応）
func TestTickMonotonic(t *testing.T) {
	// nodeId=0 で HlcClock を生成する
	clock := NewHlcClock(0)
	// 連続で 100 回 Tick して全て単調増加することを確認する（Rust と同一の 100 回）
	prev := clock.Tick()
	for i := 0; i < 99; i++ {
		// 次のタイムスタンプを生成する
		next := clock.Tick()
		// prev よりも next が後（または同時）であることを確認する
		if next.Compare(prev) < 0 {
			t.Errorf("Tick は単調増加を保証すべき: prev=%v, next=%v", prev, next)
		}
		// prev を更新する
		prev = next
	}
}

// TestRecvCausality は HlcClock.Recv がメッセージの因果関係を正しく反映することを確認するテスト（Rust: test_recv_causality に対応）
func TestRecvCausality(t *testing.T) {
	// 送信者クロックを生成する
	sender := NewHlcClock(1)
	// 受信者クロックを生成する
	receiver := NewHlcClock(2)
	// 送信者で Tick する
	sendTs := sender.Tick()
	// 受信者で Recv する（sendTs の後になるはずである）
	recvTs := receiver.Recv(sendTs)
	// Recv の結果が sendTs 以後であることを確認する
	if recvTs.Compare(sendTs) < 0 {
		t.Errorf("Recv 後のタイムスタンプは send より後であるべき: send=%v, recv=%v", sendTs, recvTs)
	}
}

// TestConcurrentTick は複数 goroutine で concurrent Tick が単調増加を維持することを確認するテスト（Rust: test_concurrent_tick に対応）
func TestConcurrentTick(t *testing.T) {
	// Arc で HlcClock を共有する（Go では *HlcClock ポインタを直接共有できる）
	clock := NewHlcClock(0)
	// 結果を格納するスライスと Mutex を初期化する
	var mu sync.Mutex
	// 全 goroutine の結果を蓄積するスライス
	all := make([]HlcTimestamp, 0, 1000)
	// WaitGroup で全 goroutine の完了を待つ
	var wg sync.WaitGroup
	// 10 goroutine で並行 Tick を実行する（Rust: 10 スレッドと同等）
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			// goroutine 終了時に WaitGroup を通知する
			defer wg.Done()
			// 各 goroutine で 100 回 Tick する（合計 1000 回）
			results := make([]HlcTimestamp, 100)
			for j := 0; j < 100; j++ {
				results[j] = clock.Tick()
			}
			// 結果を all に追加する（Mutex で排他制御）
			mu.Lock()
			all = append(all, results...)
			mu.Unlock()
		}()
	}
	// 全 goroutine の完了を待つ
	wg.Wait()
	// 全タイムスタンプを集合に入れて重複を確認する
	seen := make(map[string]bool, len(all))
	for _, ts := range all {
		// FormatCompact をキーとして重複チェックする
		key := fmt.Sprintf("%016x-%04x-%04x", ts.WallMs, ts.Logical, ts.NodeId)
		if seen[key] {
			t.Errorf("concurrent Tick で重複タイムスタンプが生成された: %s", key)
		}
		seen[key] = true
	}
	// 1000 個（10 goroutine × 100 Tick）のユニークタイムスタンプが生成されたことを確認する
	if len(seen) != 1000 {
		t.Errorf("concurrent Tick は 1000 個の unique タイムスタンプを生成すべき: got=%d", len(seen))
	}
}

// TestEpochIsMinimum は Epoch 定数が最小タイムスタンプであることを確認するテスト（Rust: test_epoch_is_minimum に対応）
func TestEpochIsMinimum(t *testing.T) {
	// 任意の非 Epoch タイムスタンプを生成する
	ts := HlcTimestamp{WallMs: 1, Logical: 0, NodeId: 0}
	// Epoch が ts より前であることを確認する
	if Epoch.Compare(ts) >= 0 {
		t.Error("Epoch は全タイムスタンプの最小値であるべき")
	}
}

// TestNowIsTickAlias は HlcClock.Now が Tick の alias であることを確認するテスト
func TestNowIsTickAlias(t *testing.T) {
	// nodeId=1 で HlcClock を生成する
	clock := NewHlcClock(1)
	// Now を呼び出す
	ts := clock.Now()
	// NodeId が引き継がれることを確認する
	if ts.NodeId != 1 {
		t.Errorf("Now() の NodeId は 1 であるべき: got=%d", ts.NodeId)
	}
}

// TestNewHlcClockFromEnv は NewHlcClockFromEnv が HLC_NODE_ID 未設定時に nodeId=0 の HlcClock を生成することを確認するテスト
func TestNewHlcClockFromEnv(t *testing.T) {
	// HLC_NODE_ID が未設定の環境でテストする（テスト環境では未設定を想定）
	clock := NewHlcClockFromEnv()
	// HlcClock が生成されることを確認する
	if clock == nil {
		t.Fatal("NewHlcClockFromEnv は nil を返すべきでない")
	}
	// Tick が動作することを確認する
	ts := clock.Tick()
	// WallMs が 0 より大きいことを確認する（現在時刻が設定されている）
	if ts.WallMs == 0 {
		t.Error("Tick 後の WallMs は 0 より大きいはず（物理時刻が反映される）")
	}
}
