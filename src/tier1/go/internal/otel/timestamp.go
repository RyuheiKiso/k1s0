// 本ファイルは OTel 関連の timestamp 変換ユーティリティ。
//
// k1s0 proto は UTC unix nanoseconds または google.protobuf.Timestamp で
// 時刻を表現する。OTel SDK は time.Time 系を期待するため変換を提供する。

package otel

import (
	// time.Time 変換に使う。
	"time"
)

// otelLogTimestampFromUnixNanos は unix nanoseconds から time.Time を作る。
// OTel Log の Record.SetTimestamp は time.Time を要求する（Logger 側で変換）。
func otelLogTimestampFromUnixNanos(ns int64) time.Time {
	return time.Unix(0, ns).UTC()
}
