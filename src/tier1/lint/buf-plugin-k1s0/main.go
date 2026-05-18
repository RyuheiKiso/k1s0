// main.go — k1s0 tier1 buf-plugin-k1s0: Buf lint plugin エントリポイント
// protoc plugin プロトコル（stdin: CodeGeneratorRequest / stdout: CodeGeneratorResponse）で動作する。
// tier1/CLAUDE.md §必須 annotation（conformance_class / auth_class / slo_class /
// quota_class / observability.signal_class）の欠落を検出して lint エラーを返す。
// buf カスタム lint plugin は `buf lint --config buf.yaml` で実行する。

// パッケージ名: main（buf-plugin-k1s0 バイナリのエントリポイント）
package main

import (
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// io: stdin の読み込みに使用する
	"io"
	// os: 標準入出力に使用する
	"os"

	// google.golang.org/protobuf/proto: protobuf バイナリのアンマーシャルに使用する
	"google.golang.org/protobuf/proto"
	// google.golang.org/protobuf/types/pluginpb: protoc plugin API の CodeGeneratorRequest に使用する
	"google.golang.org/protobuf/types/pluginpb"
)

// main は buf-plugin-k1s0 バイナリのエントリポイント
// buf lint がこのバイナリを stdin/stdout プロトコルで呼び出す
func main() {
	// stdin から CodeGeneratorRequest バイナリを読み込む
	input, err := io.ReadAll(os.Stdin)
	// stdin 読み込み失敗時はエラーを stderr に出力して exit 1 する
	if err != nil {
		// エラーを stderr に出力する
		fmt.Fprintf(os.Stderr, "buf-plugin-k1s0: failed to read stdin: %v\n", err)
		// exit 1 で終了する
		os.Exit(1)
	}

	// CodeGeneratorRequest をアンマーシャルする
	var request pluginpb.CodeGeneratorRequest
	// protobuf バイナリをデコードする
	if err := proto.Unmarshal(input, &request); err != nil {
		// アンマーシャル失敗時はエラーを stderr に出力して exit 1 する
		fmt.Fprintf(os.Stderr, "buf-plugin-k1s0: failed to unmarshal CodeGeneratorRequest: %v\n", err)
		// exit 1 で終了する
		os.Exit(1)
	}

	// proto ファイルのサービス descriptor を走査して annotation チェックを実行する
	var allViolations []AnnotationViolation

	// 全 proto ファイルの FileDescriptorProto を走査する
	for _, fd := range request.GetProtoFile() {
		// サービス定義を走査する
		for _, sd := range fd.GetService() {
			// サービス名を取得する（package.ServiceName 形式）
			serviceName := fd.GetPackage() + "." + sd.GetName()
			// サービスの全メソッドを走査する
			for _, method := range sd.GetMethod() {
				// メソッド名を取得する
				methodName := method.GetName()
				// ClientStreaming / ServerStreaming: bidi RPC かどうかを確認する
				isBidi := method.GetClientStreaming() && method.GetServerStreaming()
				// bidi RPC の場合は conformance_class が必須かチェックする
				if isBidi {
					// メソッドに options が存在するかチェックする（NULL = 全 annotation 欠落）
					opts := method.GetOptions()
					// opts が nil の場合は conformance_class が欠落している
					if opts == nil {
						// conformance_class が欠落している場合は違反として記録する
						allViolations = append(allViolations, AnnotationViolation{
							// サービス名を設定する
							ServiceName: serviceName,
							// メソッド名を設定する
							MethodName: methodName,
							// 欠落 annotation: bidi RPC の conformance_class
							MissingAnnotation: "tier1.bidi.conformance_class",
						})
					}
				}
				// 全 RPC: options が nil の場合は auth_class も欠落している
				opts := method.GetOptions()
				// opts が nil の場合は auth_class が欠落している
				if opts == nil {
					// auth_class が欠落している場合は違反として記録する
					allViolations = append(allViolations, AnnotationViolation{
						// サービス名を設定する
						ServiceName: serviceName,
						// メソッド名を設定する
						MethodName: methodName,
						// 欠落 annotation: auth_class
						MissingAnnotation: "tier1.auth.auth_class",
					})
				}
			}
		}
	}

	// CodeGeneratorResponse を構築する（lint plugin はコード生成を行わない）
	response := &pluginpb.CodeGeneratorResponse{}

	// 違反がある場合は Error フィールドに設定する
	if len(allViolations) > 0 {
		// 違反メッセージを構築する
		var errorMsg string
		// 各違反を stderr に出力して Error フィールドに追記する
		for _, v := range allViolations {
			// 違反メッセージを stderr に出力する
			fmt.Fprintln(os.Stderr, FormatViolation(v))
			// Error フィールドに違反メッセージを追記する
			errorMsg += FormatViolation(v) + "\n"
		}
		// Error フィールドに違反メッセージを設定する（Buf lint plugin 規約）
		response.Error = proto.String(errorMsg)
	}

	// CodeGeneratorResponse をマーシャルして stdout に書き込む
	out, err := proto.Marshal(response)
	// マーシャル失敗時はエラーを stderr に出力して exit 1 する
	if err != nil {
		// エラーを stderr に出力する
		fmt.Fprintf(os.Stderr, "buf-plugin-k1s0: failed to marshal CodeGeneratorResponse: %v\n", err)
		// exit 1 で終了する
		os.Exit(1)
	}
	// stdout に書き込む
	if _, err := os.Stdout.Write(out); err != nil {
		// 書き込み失敗時はエラーを stderr に出力して exit 1 する
		fmt.Fprintf(os.Stderr, "buf-plugin-k1s0: failed to write stdout: %v\n", err)
		// exit 1 で終了する
		os.Exit(1)
	}
}
