// 本ファイルは feature.go から分離された JSON 変換 helper。
//
// 設計正典:
//   docs/02_構想設計/adr/ADR-FM-001-flagd-openfeature.md
//
// 役割:
//   flagd の Object flag 評価結果（map[string]any 等の生 Go 値）を
//   JSON bytes に marshal するヘルパ。標準 encoding/json 直接呼出を集約することで、
//   将来 sonic / segmentio/encoding に差し替えるときの変更箇所を 1 か所にする。

package dapr

import (
	// 標準 JSON エンコーダ。
	"encoding/json"
)

// jsonMarshal は flagd の Object 値を JSON bytes に marshal する薄いラッパ。
// 失敗時は err を返し、呼出側で fail-soft（ERROR reason 返却）を行う。
func jsonMarshal(v any) ([]byte, error) {
	return json.Marshal(v)
}
