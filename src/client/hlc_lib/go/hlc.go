// hlc.go — k1s0-hlc: Hybrid Logical Clock (HLC) の Go 実装
// Kulkarni et al. (2014) "Logical Physical Clocks" のアルゴリズムを Go で実装する。
// wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// TTL / deadline の比較は HLC elapsed で管理し、time.Now() を直接参照しない。
// 本パッケージが time.Now() を扱う唯一の許可された場所であり、呼び出し元は本パッケージを経由する。
// Rust 実装（src/client/hlc_lib/rust/src/lib.rs）と同等の API を提供する。
package hlc

// fmt: FormatCompact / ParseCompact で文字列フォーマットに使用する
import (
	// fmt: 文字列フォーマット（FormatCompact）と scanf（ParseCompact）に使用する
	"fmt"
	// math: saturating_add 相当の上限クランプに使用する
	"math"
	// os: HLC_NODE_ID 環境変数の取得に使用する
	"os"
	// strconv: 環境変数文字列の u16 パースに使用する
	"strconv"
	// sync: HlcClock の Mutex 保護に使用する
	"sync"
	// time: wall-clock の取得に使用する（HlcClock 内部のみ許可）
	"time"
)

// HlcTimestamp: HLC のタイムスタンプ（WallMs + Logical + NodeId の 3 tuple）
// 全順序（WallMs > Logical > NodeId の辞書順）で比較可能。
// FormatCompact で "{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式に変換できる。
type HlcTimestamp struct {
	// WallMs: UNIX epoch からの経過ミリ秒（wall-clock 部分、記録目的のみ）
	WallMs uint64
	// Logical: 同一 WallMs 内の単調カウンタ（最大 65535 = uint16 の最大値）
	Logical uint16
	// NodeId: ノード識別子（複数インスタンスでの衝突回避、環境変数 HLC_NODE_ID で指定）
	NodeId uint16
}

// Epoch は WallMs=0, Logical=0, NodeId=0 の最小タイムスタンプ定数（未初期化判定に使用）
var Epoch = HlcTimestamp{WallMs: 0, Logical: 0, NodeId: 0}

// FormatCompact は HlcTimestamp を文字列に変換する
// 形式: "{wall_ms_hex_16}-{logical_04x}-{node_04x}"
// tier2 cache_layer の generate_hlc_timestamp が生成していた形式と互換性を保つ
func (t HlcTimestamp) FormatCompact() string {
	// 16 桁 hex + 4 桁 hex + 4 桁 hex の形式でフォーマットして返す
	return fmt.Sprintf("%016x-%04x-%04x", t.WallMs, t.Logical, t.NodeId)
}

// ParseCompact は FormatCompact が出力した文字列を HlcTimestamp にパースする
// パースに失敗した場合は HlcTimestamp{} と false を返す（不正入力を呼び出し元で処理させる）
func ParseCompact(s string) (HlcTimestamp, bool) {
	// WallMs / Logical / NodeId の 3 フィールドを宣言する
	var wallMs uint64
	var logical uint16
	var nodeId uint16
	// Sscanf で "WWWWWWWWWWWWWWWW-LLLL-NNNN" の形式をパースする
	n, err := fmt.Sscanf(s, "%016x-%04x-%04x", &wallMs, &logical, &nodeId)
	// パースに失敗した場合（n != 3 または err != nil）は false を返す
	if err != nil || n != 3 {
		return HlcTimestamp{}, false
	}
	// 入力文字列の長さが期待値（26 文字）と一致することを確認する（余分な文字を拒否）
	if len(s) != 26 {
		return HlcTimestamp{}, false
	}
	// パースに成功した場合は HlcTimestamp を返す
	return HlcTimestamp{WallMs: wallMs, Logical: logical, NodeId: nodeId}, true
}

// AddMs は self に durationMs を加算した deadline 用 HlcTimestamp を返す
// wall-clock の直接使用を禁止するため、deadline 表現はこの関数を経由する
// overflow 時は math.MaxUint64 に飽和する（Rust の saturating_add と同等）
func (t HlcTimestamp) AddMs(durationMs uint64) HlcTimestamp {
	// WallMs に durationMs を加算する（オーバーフロー検出）
	newWall := t.WallMs + durationMs
	// 加算結果が元の WallMs より小さい場合はオーバーフローが発生している
	if newWall < t.WallMs {
		// MaxUint64 に飽和させる（Rust の saturating_add と同等）
		newWall = math.MaxUint64
	}
	// deadline の先頭イベントを表すため Logical を 0 にリセットする
	// NodeId は引き継ぐ（deadline の発行者を追跡する）
	return HlcTimestamp{WallMs: newWall, Logical: 0, NodeId: t.NodeId}
}

// ElapsedMsSince は reference から self までの経過ミリ秒を返す
// self が reference より前の場合は 0 を返す（負の elapsed は表現しない）
// deadline との差分比較（expired 判定）に使用する
func (t HlcTimestamp) ElapsedMsSince(reference HlcTimestamp) uint64 {
	// WallMs の差分を返す（self が reference 以前なら 0 に飽和する）
	if t.WallMs <= reference.WallMs {
		// self が reference 以前の場合は 0 を返す（飽和減算）
		return 0
	}
	// self が reference より後の場合は差分を返す
	return t.WallMs - reference.WallMs
}

// IsExpiredAt は deadline と比較して self が期限切れかどうかを返す
// current: 現在の HLC タイムスタンプ（HlcClock.Now() で取得）
// true = current が self（deadline）を超えた = 期限切れ
func (t HlcTimestamp) IsExpiredAt(current HlcTimestamp) bool {
	// current が self 以上（後または同時）なら期限切れ
	return t.Compare(current) <= 0
}

// Compare は self と other を全順序で比較する
// 戻り値: -1（self < other）/ 0（self == other）/ 1（self > other）
// 比較順序: WallMs → Logical → NodeId（辞書順）
func (a HlcTimestamp) Compare(b HlcTimestamp) int {
	// WallMs を比較する
	if a.WallMs < b.WallMs {
		return -1
	}
	if a.WallMs > b.WallMs {
		return 1
	}
	// WallMs が同一なら Logical を比較する
	if a.Logical < b.Logical {
		return -1
	}
	if a.Logical > b.Logical {
		return 1
	}
	// Logical も同一なら NodeId を比較する（完全全順序を保証する）
	if a.NodeId < b.NodeId {
		return -1
	}
	if a.NodeId > b.NodeId {
		return 1
	}
	// 全フィールドが同一なら 0（等値）を返す
	return 0
}

// HlcClock: スレッドセーフな HLC クロック（sync.Mutex で保護）
// tick / recv / now の 3 操作でイベント間の因果関係を追跡する
// 複数 goroutine から安全に使用できる
type HlcClock struct {
	// mu: stateWall / stateLogical を保護する Mutex
	mu sync.Mutex
	// stateWall: 最後に観測した WallMs（単調増加を保証するための状態）
	stateWall uint64
	// stateLogical: 最後に使用した Logical カウンタ
	stateLogical uint16
	// nodeId: このノード固有の識別子（HLC_NODE_ID 環境変数 or 0）
	nodeId uint16
}

// NewHlcClock は nodeId を受け取って HlcClock を生成する
// 複数ノード環境では異なる nodeId を設定してタイムスタンプの衝突を避ける
func NewHlcClock(nodeId uint16) *HlcClock {
	// 初期ステート: stateWall=0, stateLogical=0（first Tick で物理クロックに更新される）
	return &HlcClock{
		// stateWall: 初期値 0（first Tick で更新される）
		stateWall: 0,
		// stateLogical: 初期値 0
		stateLogical: 0,
		// nodeId を設定する
		nodeId: nodeId,
	}
}

// NewHlcClockFromEnv は環境変数 HLC_NODE_ID から nodeId を読み込んで HlcClock を生成する
// HLC_NODE_ID が未設定 / パース失敗時は nodeId=0 を使用する
func NewHlcClockFromEnv() *HlcClock {
	// HLC_NODE_ID 環境変数を文字列として取得する
	nodeIdStr := os.Getenv("HLC_NODE_ID")
	// 環境変数が未設定 / 空の場合は nodeId=0 を使用する
	if nodeIdStr == "" {
		return NewHlcClock(0)
	}
	// 文字列を 64-bit 整数にパースする（u16 範囲を確認するため ParseUint で bit=16 指定）
	parsed, err := strconv.ParseUint(nodeIdStr, 10, 16)
	// パース失敗の場合は nodeId=0 を使用する
	if err != nil {
		return NewHlcClock(0)
	}
	// パース成功した nodeId を使って HlcClock を初期化する
	return NewHlcClock(uint16(parsed))
}

// wallMsNow は現在の UNIX epoch からの経過ミリ秒を返す（内部専用）
// HLC の wall-clock 部分の取得にのみ使用する（TTL/deadline 計算での直接使用禁止）
func wallMsNow() uint64 {
	// time.Now().UnixMilli() を呼び出す（本パッケージ内でのみ許可、呼び出し元では禁止）
	ms := time.Now().UnixMilli()
	// 負値（time.Now() が UNIX epoch より前）は 0 として返す（理論上は発生しないが安全策）
	if ms < 0 {
		return 0
	}
	// int64 を uint64 に変換して返す
	return uint64(ms)
}

// Tick は send/local event のタイムスタンプを生成する（HLC の "send event"）
// アルゴリズム:
//
//	l' = max(stateWall, pt)
//	if l' == stateWall: c' = stateLogical + 1
//	else: c' = 0
func (c *HlcClock) Tick() HlcTimestamp {
	// Mutex を取得する（goroutine セーフ）
	c.mu.Lock()
	// defer で Mutex を解放する
	defer c.mu.Unlock()
	// pt: 現在の物理クロック（ms）を取得する
	pt := wallMsNow()
	// l': max(stateWall, pt) を計算する（単調増加を保証する）
	newWall := c.stateWall
	if pt > newWall {
		newWall = pt
	}
	// c': wall_ms が変化したかどうかで logical を更新する
	var newLogical uint16
	if newWall == c.stateWall {
		// wall_ms が変わらなければ logical を +1 する
		if c.stateLogical == math.MaxUint16 {
			// logical が u16 の最大値に達した場合は panic で早期終了する（Rust と同等）
			panic("HLC logical counter overflow (max uint16 = 65535)")
		}
		newLogical = c.stateLogical + 1
	} else {
		// wall_ms が進んだ場合は logical を 0 にリセットする
		newLogical = 0
	}
	// ステートを更新する（次回の Tick/Recv の比較基準になる）
	c.stateWall = newWall
	c.stateLogical = newLogical
	// 生成した HlcTimestamp を返す
	return HlcTimestamp{WallMs: newWall, Logical: newLogical, NodeId: c.nodeId}
}

// Recv は受信メッセージのタイムスタンプを踏まえてローカルクロックを更新する（HLC の "receive event"）
// アルゴリズム（3-way max）:
//
//	l' = max(stateWall, msgTs.WallMs, pt)
//	3 者の最大一致に応じて logical を更新する
func (c *HlcClock) Recv(msgTs HlcTimestamp) HlcTimestamp {
	// Mutex を取得する
	c.mu.Lock()
	// defer で Mutex を解放する
	defer c.mu.Unlock()
	// pt: 現在の物理クロックを取得する
	pt := wallMsNow()
	// l': max(stateWall, msgTs.WallMs, pt) を計算する（3-way max）
	newWall := c.stateWall
	if msgTs.WallMs > newWall {
		newWall = msgTs.WallMs
	}
	if pt > newWall {
		newWall = pt
	}
	// c': 3-way の最大一致パターンに応じて logical を更新する
	var newLogical uint16
	if newWall == c.stateWall && newWall == msgTs.WallMs {
		// 3 者の wall_ms が同一: max(local_logical, msg_logical) + 1
		maxLogical := c.stateLogical
		if msgTs.Logical > maxLogical {
			maxLogical = msgTs.Logical
		}
		// logical がオーバーフローする場合は panic する
		if maxLogical == math.MaxUint16 {
			panic("HLC logical counter overflow")
		}
		newLogical = maxLogical + 1
	} else if newWall == c.stateWall {
		// ローカルの wall_ms が最大: local_logical + 1
		if c.stateLogical == math.MaxUint16 {
			panic("HLC logical counter overflow")
		}
		newLogical = c.stateLogical + 1
	} else if newWall == msgTs.WallMs {
		// 受信メッセージの wall_ms が最大: msg_logical + 1
		if msgTs.Logical == math.MaxUint16 {
			panic("HLC logical counter overflow")
		}
		newLogical = msgTs.Logical + 1
	} else {
		// pt が最大（物理クロックが両者を上回った）: logical を 0 にリセットする
		newLogical = 0
	}
	// ステートを更新する
	c.stateWall = newWall
	c.stateLogical = newLogical
	// 更新後の HlcTimestamp を返す
	return HlcTimestamp{WallMs: newWall, Logical: newLogical, NodeId: c.nodeId}
}

// Now は現在の HLC タイムスタンプを生成する（Tick の alias）
// キャッシュエントリの CachedAtHlc フィールドへの書き込みに使用する
func (c *HlcClock) Now() HlcTimestamp {
	// Tick と等価：send event として扱う
	return c.Tick()
}
