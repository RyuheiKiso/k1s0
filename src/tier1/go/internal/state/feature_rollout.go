// 本ファイルは Feature Flag の段階ロールアウト (percentage-based assignment) helper。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md (FR-T1-FEATURE-002)
//   docs/02_構想設計/adr/ADR-FM-001-flagd.md (flagd / OpenFeature 採用)
//
// 役割:
//   FR-T1-FEATURE-002「flag 定義に rollout: { percentage: 10 } を指定すると、ユーザー
//   ID のハッシュベースで 10% のユーザに有効化される / 同一 user_id は常に同じ結果を
//   返す」を、tier1 facade 内 client-side ヘルパとして実装する。
//
// 設計判断:
//   flagd の Component fractional 機構 (CEL 式) と独立に、tier1 facade で sticky な
//   user_id 単位の rollout 評価を行えるようにする。flagd を使わない / flagd 障害時
//   (NFR-A-CONT-006 fallback) でも tier1 facade が default rollout を返せる経路。
//
// 性能特性:
//   - 1 回の評価は SHA-256 ハッシュ計算 1 回 (~1µs オーダ)、percentage 比較 1 回
//   - context-free で goroutine-safe (state を持たない純関数)
//   - 同 (user_id, flag_key) 組合せで常に同結果 (sticky 保証)

package state

import (
	// SHA-256 ハッシュ計算。
	"crypto/sha256"
	// hash 結果の bytes → uint64 変換。
	"encoding/binary"
)

// featureRolloutBucket は percentage 比較用の固定 bucket 数。100 で割合 % と一致する。
// 大きく取ると粒度が細かくなるが、本 helper は要件の「10% / 50% / 100% 段階拡大」が
// 主用途のため、100 で十分な粒度。
const featureRolloutBucket = 100

// RolloutAssign は (userID, flagKey, percentage) で sticky な true/false を返す。
//
// 受け入れ基準 (FR-T1-FEATURE-002):
//   - 同一 user_id は常に同じ結果を返す → SHA-256 hash の決定性で保証
//   - パーセントを 10 → 50 → 100 に変更で段階拡大 → percentage 引数のみ変更で粒度移行
//   - percentage <= 0 で全 false、>= 100 で全 true
//
// 入力は trim されない (caller の前提通り扱う)。空文字 user_id でも sticky に動作する
// (anonymous user 用途)。空文字 flagKey は呼出側の不備として扱い、本関数では
// 通常の処理経路 (hash 計算) を通す (defensive 補正は呼出側が行う)。
func RolloutAssign(userID, flagKey string, percentage int) bool {
	// percentage 0 以下は全員 false (rollout 未開始)。
	if percentage <= 0 {
		return false
	}
	// percentage 100 以上は全員 true (全展開済)。
	if percentage >= featureRolloutBucket {
		return true
	}
	// hash 入力を組み立てる: flag_key と user_id を ":" で連結する。
	// flag_key を先頭にすることで、複数 flag が同 user_id でも独立した bucket に割り当たる
	// (= 異なる flag で異なるユーザ群が選ばれる、相関性が出ない)。
	input := flagKey + ":" + userID
	// SHA-256 hash を計算する。
	digest := sha256.Sum256([]byte(input))
	// 最初の 8 byte を big-endian uint64 として読む (decode 高速、暗号学的偏りなし)。
	bucket := binary.BigEndian.Uint64(digest[:8])
	// modulo bucket 数で 0..99 に正規化、percentage と比較する。
	return int(bucket%featureRolloutBucket) < percentage
}

// RolloutPercentageOf は (userID, flagKey) の bucket 番号 (0..99) を返す helper。
// テスト / debug 用途で、user が「何%の閾値で有効化されるか」を確認できる。
// 業務コードでは RolloutAssign を使う (本関数は内部的に bucket を露出する)。
func RolloutPercentageOf(userID, flagKey string) int {
	// hash 入力を RolloutAssign と同じ規則で組み立てる。
	input := flagKey + ":" + userID
	// SHA-256 hash を計算する。
	digest := sha256.Sum256([]byte(input))
	// 最初の 8 byte を uint64 に変換する。
	bucket := binary.BigEndian.Uint64(digest[:8])
	// 0..99 の範囲に正規化して返す。
	return int(bucket % featureRolloutBucket)
}
