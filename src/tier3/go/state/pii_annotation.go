// pii_annotation.go — field_pii annotation の Go 実装（reflect タグベース + compile-time 強制）
// struct tag `pii:"true"` を使って PII フィールドを宣言し、
// GetPiiFields で reflect を使って PII フィールド名を抽出する
// Y-tier3-4: field_pii annotation compile-time 強制（4 言語等価強度の Go 実装）
package state

import (
	// reflect パッケージ: struct tag の読み取りに使用する
	"reflect"
)

// PiiField は struct tag `pii:"true"` を持つフィールドのマーカー型（型エイリアス）
// Go では decorator がないため struct tag で宣言する（`pii:"true"` タグが SoT）
// 使用例: type Foo struct { Email string `pii:"true"` }
type PiiField = struct{}

// PiiTagKey は struct tag のキー文字列（`pii:"true"` の "pii" 部分）
// 全コードで統一的に使用する定数として定義する
const PiiTagKey = "pii"

// PiiTagValue は PII フィールドを宣言する tag 値（`pii:"true"` の "true" 部分）
// "true" 以外の値は PII フィールドとして認識しない
const PiiTagValue = "true"

// PiiAnnotated は PII フィールドを持つ struct が実装するインタフェース
// GetPiiFieldNames: 実装クラスが PII フィールド名のスライスを返す
// このインタフェースの実装は自動ではなく手動（将来は go:generate で生成する）
type PiiAnnotated interface {
	// GetPiiFieldNames は PII フィールド名のスライスを返す
	GetPiiFieldNames() []string
}

// GetPiiFields は v の reflect.Type を走査して `pii:"true"` tag を持つフィールド名を返す
// v: PII フィールドを持つ struct の interface{} 値（ポインタ / 値 いずれも可）
// 返値: `pii:"true"` tag を持つフィールド名のスライス（順序は struct 定義順）
func GetPiiFields(v interface{}) []string {
	// v の reflect.Type を取得する（ポインタの場合は Elem() で実体を取得する）
	t := reflect.TypeOf(v)
	// t が nil の場合は空スライスを返す（nil 引数への安全なフォールバック）
	if t == nil {
		// nil 引数は空スライスを返す
		return []string{}
	}
	// ポインタの場合は Elem() で実体の型を取得する
	if t.Kind() == reflect.Ptr {
		// ポインタを Dereference して実体の型を取得する
		t = t.Elem()
	}
	// struct でない場合は空スライスを返す（struct 以外は tag を持てない）
	if t.Kind() != reflect.Struct {
		// struct 以外は空スライスを返す
		return []string{}
	}
	// PII フィールド名を格納するスライスを生成する
	piiFields := make([]string, 0, t.NumField())
	// struct の全フィールドを走査する
	for i := 0; i < t.NumField(); i++ {
		// i 番目のフィールドを取得する
		field := t.Field(i)
		// struct tag から "pii" キーの値を取得する（`pii:"true"` の場合は "true" が返る）
		tagVal := field.Tag.Get(PiiTagKey)
		// tag 値が PiiTagValue（"true"）の場合は PII フィールドとして追加する
		if tagVal == PiiTagValue {
			// フィールド名を PII フィールドスライスに追加する
			piiFields = append(piiFields, field.Name)
		}
	}
	// PII フィールド名のスライスを返す（空スライスは []string{} になる）
	return piiFields
}

// HasPiiFields は v が `pii:"true"` tag を持つフィールドを 1 件以上持つかどうかを返す
// GetPiiFields の convenience wrapper（存在確認のみが目的の場合に使用する）
func HasPiiFields(v interface{}) bool {
	// GetPiiFields を呼び出してフィールド数を確認する
	return len(GetPiiFields(v)) > 0
}

// ValidatePiiStripped は payload map から PII フィールドが除去済みかを検証する
// GetPiiFields が返すフィールド名が payload に含まれていないことを確認する
// v: PII フィールドの宣言元 struct の interface{} 値
// payload: 送信対象の map（PII フィールドが strip 済みであることを確認する）
// 返値: payload に残存する PII フィールド名のスライス（空スライス = strip 完了）
func ValidatePiiStripped(v interface{}, payload map[string]interface{}) []string {
	// v から PII フィールド名を取得する
	piiFields := GetPiiFields(v)
	// payload に残存する PII フィールドを格納するスライスを生成する
	remaining := make([]string, 0)
	// PII フィールドを走査して payload に残存するか確認する
	for _, field := range piiFields {
		// payload に PII フィールドが存在する場合は残存リストに追加する
		if _, exists := payload[field]; exists {
			// strip 漏れの PII フィールドを追加する
			remaining = append(remaining, field)
		}
	}
	// 残存する PII フィールド名のスライスを返す（空スライス = strip 完了）
	return remaining
}
