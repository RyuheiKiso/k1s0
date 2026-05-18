// k1s0 tier2 i18n formatter Go インターフェース定義
// ICU 風の number / date / currency / unit formatter の Go インターフェース
// Rust 実装（rust/src/i18n_formatter.rs）と 4 言語等価強度を持つ Go 版
// wall clock TTL 禁止規約準拠: 日付は HLC タイムスタンプから変換した上で fmt へ渡す

// パッケージ名: tier2 Go ルートパッケージ
package tier2

import (
	// fmt パッケージ: スタブ実装のフォーマットに使用する
	"fmt"
)

// NumberFormatOptions: 数値フォーマットオプション
// ICU NumberFormat オプションに相当するオプション集合
// Rust の NumberFormatOptions 構造体に対応する
type NumberFormatOptions struct {
	// MinimumFractionDigits: 小数点以下の最小桁数（-1 で未指定）
	MinimumFractionDigits int
	// MaximumFractionDigits: 小数点以下の最大桁数（-1 で未指定）
	MaximumFractionDigits int
	// UseGrouping: 千単位区切り文字を使用するか
	UseGrouping bool
}

// DateFormatStyle: 日付フォーマットの種別（ICU の DateTimeStyle に対応する）
// Rust の DateFormat enum に対応する
type DateFormatStyle int

const (
	// DateFormatShort: 短形式（例: 2024/01/15）
	DateFormatShort DateFormatStyle = iota
	// DateFormatMedium: 中形式（例: 2024年1月15日）
	DateFormatMedium
	// DateFormatLong: 長形式（例: 2024年1月15日 月曜日）
	DateFormatLong
	// DateFormatCustom: カスタム書式文字列（ICU パターン形式）
	DateFormatCustom
)

// LocaleFormatter: ICU 風 number / date / currency / unit formatter の Go インターフェース
// Rust の LocaleFormatter トレイトに対応する
// 4 言語等価原則により Rust / C# / Go / TypeScript で同じシグネチャを持つ
// wall clock を直接 TTL に使うことを禁止し、HLC timestamp を経由する設計を維持する
type LocaleFormatter interface {
	// FormatNumber: 数値をロケール形式にフォーマットする
	// value: フォーマット対象の数値（float64）
	// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
	// options: フォーマットオプション（小数点桁数 / 区切り文字など）
	// フォーマットされた文字列を返す
	FormatNumber(value float64, locale string, options *NumberFormatOptions) (string, error)

	// FormatDate: 日付をロケール形式にフォーマットする
	// timestampMillis: ミリ秒エポック形式の日付（HLC タイムスタンプから変換する）
	// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
	// style: 日付フォーマット種別（DateFormatShort / Medium / Long / Custom）
	// customPattern: style が DateFormatCustom の場合に使用する ICU パターン文字列
	// フォーマットされた文字列を返す
	FormatDate(timestampMillis int64, locale string, style DateFormatStyle, customPattern string) (string, error)

	// FormatCurrency: 通貨金額をロケール形式にフォーマットする
	// amount: フォーマット対象の金額（float64）
	// currency: ISO 4217 通貨コード（例: "JPY" / "USD" / "EUR"）
	// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
	// フォーマットされた文字列を返す
	FormatCurrency(amount float64, currency string, locale string) (string, error)

	// FormatUnit: 単位付き数値をロケール形式にフォーマットする
	// value: フォーマット対象の数値（float64）
	// unit: CLDR unit identifier（例: "kilogram" / "meter" / "liter"）
	// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
	// フォーマットされた文字列を返す
	FormatUnit(value float64, unit string, locale string) (string, error)
}

// LocaleFormatterStub: テスト / stub 実装
// 実際のロケール変換は行わず、引数をそのまま文字列化して返す
// production / development 区別禁止規約に従い、本実装は必ず ICU backend を使用すること
type LocaleFormatterStub struct{}

// FormatNumber スタブ実装（ロケール変換なしで float64 を文字列化する）
func (s *LocaleFormatterStub) FormatNumber(value float64, _ string, _ *NumberFormatOptions) (string, error) {
	// スタブとして数値を文字列化して返す（ICU 実装では CLDR データを使用する）
	return fmt.Sprintf("%g", value), nil
}

// FormatDate スタブ実装（エポック ms をそのまま文字列化して返す）
func (s *LocaleFormatterStub) FormatDate(timestampMillis int64, _ string, _ DateFormatStyle, _ string) (string, error) {
	// スタブとしてエポックミリ秒を文字列化して返す（ICU 実装では CLDR カレンダーを使用する）
	return fmt.Sprintf("%d", timestampMillis), nil
}

// FormatCurrency スタブ実装（通貨コードと金額を連結して返す）
func (s *LocaleFormatterStub) FormatCurrency(amount float64, currency string, _ string) (string, error) {
	// スタブとして "{currency} {amount}" 形式で返す
	return fmt.Sprintf("%s %g", currency, amount), nil
}

// FormatUnit スタブ実装（単位と数値を連結して返す）
func (s *LocaleFormatterStub) FormatUnit(value float64, unit string, _ string) (string, error) {
	// スタブとして "{value} {unit}" 形式で返す
	return fmt.Sprintf("%g %s", value, unit), nil
}
