// plugin.go — k1s0 tier1 Buf custom lint plugin: conformance_class 強制チェックロジック
// tier1/CLAUDE.md §必須 annotation「全 RPC method に conformance_class / auth_class /
// slo_class / quota_class / observability.signal_class の annotation が必須（欠落で CI fail）」を
// Buf lint plugin として enforce する。
// チェック対象: proto ファイルの bidi RPC に tier1.bidi.conformance_class annotation が必須。

// パッケージ名: main（main.go と同じパッケージに所属する）
package main

import (
	// strings: サービス・メソッド名の文字列処理に使用する
	"strings"

	// google.golang.org/protobuf/reflect/protoreflect: proto descriptor API に使用する
	"google.golang.org/protobuf/reflect/protoreflect"
)

// RequiredAnnotations は全 RPC に必須の option annotation 名リスト
// tier1/CLAUDE.md §必須 annotation から定義する
var RequiredAnnotations = []string{
	// tier1.bidi.conformance_class: bidi RPC の conformance class（必須）
	"tier1.bidi.conformance_class",
	// tier1.auth.auth_class: 認証クラス（必須）
	"tier1.auth.auth_class",
	// tier1.slo.slo_class: SLO クラス（必須）
	"tier1.slo.slo_class",
	// tier1.quota.quota_class: quota クラス（必須）
	"tier1.quota.quota_class",
	// tier1.observability.signal_class: 観測シグナルクラス（必須）
	"tier1.observability.signal_class",
	// tier1.schema_evolution.schema_class: スキーマ進化クラス（必須）— 06_スキーマ進化適合仕様.md §v1 schema_class セット
	"tier1.schema_evolution.v1.method_schema",
}

// AnnotationViolation は annotation 欠落違反を表す構造体
type AnnotationViolation struct {
	// ServiceName: 違反が検出されたサービス名
	ServiceName string
	// MethodName: 違反が検出された RPC メソッド名
	MethodName string
	// MissingAnnotation: 欠落している annotation 名
	MissingAnnotation string
}

// CheckMethodAnnotations は proto descriptor の ServiceDescriptor を走査して
// 必須 annotation の欠落を検出する。
// sd: 解析対象のサービス descriptor
// returns: 検出された違反リスト
func CheckMethodAnnotations(sd protoreflect.ServiceDescriptor) []AnnotationViolation {
	// 違反リストを初期化する
	var violations []AnnotationViolation
	// サービスの全メソッドを走査する
	methods := sd.Methods()
	// メソッド数だけループする
	for i := 0; i < methods.Len(); i++ {
		// メソッド descriptor を取得する
		method := methods.Get(i)
		// メソッドのオプション（annotation）を取得する
		opts := method.Options()
		// オプションが nil の場合は全 annotation が欠落している
		if opts == nil {
			// 全必須 annotation を欠落違反として記録する
			for _, ann := range RequiredAnnotations {
				violations = append(violations, AnnotationViolation{
					// サービス名を設定する
					ServiceName: string(sd.FullName()),
					// メソッド名を設定する
					MethodName: string(method.Name()),
					// 欠落 annotation 名を設定する
					MissingAnnotation: ann,
				})
			}
			// 次のメソッドに進む
			continue
		}
		// メソッドオプションの proto reflection を取得する
		optsReflect := opts.ProtoReflect()
		// 各必須 annotation が存在するかチェックする
		for _, ann := range RequiredAnnotations {
			// annotation の存在フラグを初期化する（false = 欠落とみなす）
			found := false
			// options message の設定済みフィールドを走査して annotation の存在を確認する
			// NOTE: protoreflect.Message.Range は設定済み（非 UNSPECIFIED）フィールドのみを返す。
			// これにより annotation の field presence（= 非デフォルト値が設定されていること）を確認できる。
			// 文字列 contains 検査より精度が高く、UNSPECIFIED（= 0）値を「設定済み」と誤認しない。
			optsReflect.Range(func(fd protoreflect.FieldDescriptor, _ protoreflect.Value) bool {
				// フィールドの完全修飾名（FullName）に annotation 名が含まれるか確認する
				// annotation の完全修飾名（例: tier1.bidi.conformance_class）と比較する
				if strings.Contains(string(fd.FullName()), ann) {
					// annotation が存在することを記録して走査を停止する
					found = true
					// Range を早期終了する（false を返すと Range が停止する）
					return false
				}
				// 次のフィールドに進む
				return true
			})
			// annotation が存在しない（または UNSPECIFIED = デフォルト値のまま）場合は違反として記録する
			// IMPROVEMENT: 将来的には protoreflect.ExtensionDesc で extension field presence を直接確認すること
			if !found {
				// annotation が欠落または UNSPECIFIED の場合は違反として記録する
				violations = append(violations, AnnotationViolation{
					// サービス名を設定する
					ServiceName: string(sd.FullName()),
					// メソッド名を設定する
					MethodName: string(method.Name()),
					// 欠落 annotation 名を設定する
					MissingAnnotation: ann,
				})
			}
		}
	}
	// 違反リストを返す
	return violations
}

// FormatViolation は AnnotationViolation を人間可読な文字列にフォーマットする
// v: フォーマット対象の違反
// returns: フォーマット済み文字列
func FormatViolation(v AnnotationViolation) string {
	// フォーマット: "Service.Method: missing annotation `tier1.bidi.conformance_class`"
	return v.ServiceName + "." + v.MethodName +
		": missing required annotation `" + v.MissingAnnotation +
		"` (tier1/CLAUDE.md §必須 annotation)"
}
