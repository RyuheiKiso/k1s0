// 本ファイルは tier1 facade HTTP handler の fuzz target 雛形（Go 1.18+ 標準 fuzzing）。
// 採用初期 で State.Save / PubSub.Publish 等の handler を fuzz する。
package targets

import "testing"

// FuzzFacadeRequest は tier1 facade の HTTP request body を fuzz する雛形。
// `go test -fuzz=FuzzFacadeRequest -fuzztime=5m` で起動する。
func FuzzFacadeRequest(f *testing.F) {
	// 種コーパス（採用初期 で正常リクエストの seed を追加）
	f.Add([]byte(`{"key":"k","value":"v"}`))

	f.Fuzz(func(t *testing.T, data []byte) {
		// TODO(release-initial): tier1 facade の handler を直接呼んで panic / OOM を検出する
		_ = data
	})
}
