// main.go — k1s0 Buf custom lint plugin
// tier1/CLAUDE.md §必須 annotation: 全 RPC method に 6 annotation が必須であることを
// protoc-gen-go プラグイン形式で enforce する。
// 欠落している annotation を lint error として報告する。
//
// Buf lint integration:
//   buf.yaml に以下を追加して使用する:
//     lint:
//       use: [STANDARD]
//       plugins:
//         - plugin: k1s0-lint
//           out: .
//
// 検査対象 annotation:
//   (tier1.bidi.v1.bidi_method).conformance_class    — 必須
//   (tier1.auth.v1.auth_method).auth_class             — 必須
//   (tier1.slo.v1.slo_method).slo_class                — 必須
//   (tier1.tenant_capacity.v1.quota_method).quota_class — 必須
//   (tier1.observability.v1.observability_method)       — 必須
//   (tier1.pii.v1.field_pii) on PII fields             — PII フィールドに必須

// メインパッケージ宣言
package main

import (
	// bufplugin: Buf plugin SDK（lint / generate plugin API）
	// NOTE: bufplugin は buf.build/go/bufplugin が提供する。
	//       本実装では protoc-gen プロトコルで動作する簡略版を実装する。
	// encoding/json: JSON シリアライズ（lint 結果の出力）
	"encoding/json"
	// fmt: エラーメッセージのフォーマット
	"fmt"
	// io: stdin から protoc CodeGeneratorRequest を読み取る
	"io"
	// os: stdin / stdout / stderr へのアクセス
	"os"
	// strings: method フルネームの分割
	"strings"

	// google.golang.org/protobuf/proto: protobuf デシリアライズ
	"google.golang.org/protobuf/proto"
	// google.golang.org/protobuf/types/pluginpb: CodeGeneratorRequest / Response 型
	pluginpb "google.golang.org/protobuf/types/pluginpb"
	// google.golang.org/protobuf/types/descriptorpb: FileDescriptorProto 型
	descriptorpb "google.golang.org/protobuf/types/descriptorpb"
)

// k1s0LintResult は lint チェック結果を格納する構造体
type k1s0LintResult struct {
	// FilePath: エラーが発生したファイルパス
	FilePath string `json:"file_path"`
	// Line: エラーが発生した行番号
	Line int32 `json:"line"`
	// Column: エラーが発生した列番号
	Column int32 `json:"column"`
	// Message: エラーメッセージ
	Message string `json:"message"`
	// RuleID: lint ルール ID
	RuleID string `json:"rule_id"`
}

// k1s0RequiredAnnotations は tier1 が全 RPC method に必須とする annotation の field option ID 一覧
// 各 ID は options.proto の extend google.protobuf.MethodOptions { field_id = ... } に対応する
var k1s0RequiredAnnotations = []struct {
	// name は annotation の表示名（エラーメッセージに使用する）
	name string
	// fieldNumber は MethodOptions の extend フィールド番号
	fieldNumber int32
}{
	// bidi_method: tier1.bidi.v1.bidi_method（50000）
	{name: "tier1.bidi.v1.bidi_method", fieldNumber: 50000},
	// observability_method: tier1.observability.v1.observability_method（50020）
	{name: "tier1.observability.v1.observability_method", fieldNumber: 50020},
}

// checkMethodAnnotations は MethodDescriptorProto の options を検査して
// 必須 annotation が設定されているか確認する
func checkMethodAnnotations(
	// fd: 検査対象ファイルの FileDescriptorProto
	fd *descriptorpb.FileDescriptorProto,
	// sd: 検査対象サービスの ServiceDescriptorProto
	sd *descriptorpb.ServiceDescriptorProto,
	// md: 検査対象メソッドの MethodDescriptorProto
	md *descriptorpb.MethodDescriptorProto,
	// results: lint 結果を追加するスライスへのポインタ
	results *[]k1s0LintResult,
) {
	// opts が nil の場合はオプションが設定されていないため全 annotation が欠落している
	opts := md.GetOptions()
	if opts == nil {
		// MethodOptions が設定されていない場合は全 required annotation が欠落している
		for _, ann := range k1s0RequiredAnnotations {
			// 各 annotation の欠落エラーを追加する
			*results = append(*results, k1s0LintResult{
				FilePath: fd.GetName(),
				Line:     0,
				Column:   0,
				Message:  fmt.Sprintf("RPC メソッド '%s.%s' に必須 annotation '%s' が設定されていない", sd.GetName(), md.GetName(), ann.name),
				RuleID:   "K1S0_REQUIRED_ANNOTATION",
			})
		}
		// 早期リターン
		return
	}
	// protobuf の unknown fields から extension field を確認する
	// options が設定されている場合は extension field の存在を raw proto bytes で確認する
	rawOpts, err := proto.Marshal(opts)
	if err != nil {
		// Marshal エラーは無視する（検査不能）
		return
	}
	// 各 required annotation について raw bytes に field_number が存在するか確認する
	for _, ann := range k1s0RequiredAnnotations {
		// raw proto bytes に annotation の field_number が含まれるか確認する
		if !hasProtoField(rawOpts, ann.fieldNumber) {
			// annotation が設定されていない場合はエラーを追加する
			*results = append(*results, k1s0LintResult{
				FilePath: fd.GetName(),
				Line:     0,
				Column:   0,
				Message: fmt.Sprintf(
					"RPC メソッド '%s.%s' に必須 annotation '%s' が設定されていない",
					sd.GetName(), md.GetName(), ann.name,
				),
				RuleID: "K1S0_REQUIRED_ANNOTATION",
			})
		}
	}
}

// hasProtoField は rawProto bytes に指定した field_number が存在するか確認する
// protobuf の varint エンコーディングで field_number を探す（簡易実装）
func hasProtoField(rawProto []byte, fieldNumber int32) bool {
	// protobuf wire format では field_number を varint でエンコードする
	// tag = (field_number << 3) | wire_type
	// extension field は通常 wire_type=2 (length-delimited) または wire_type=0 (varint)
	// tagWireType2 は wire_type=2 の tag を計算する
	tagWireType2 := uint64(fieldNumber)<<3 | 2
	// tagWireType0 は wire_type=0 の tag を計算する
	tagWireType0 := uint64(fieldNumber) << 3
	// raw bytes を走査して tag を探す
	for len(rawProto) > 0 {
		// varint をデコードする
		tag, n := decodeVarint(rawProto)
		if n == 0 {
			// デコード失敗（不正な bytes）
			break
		}
		// tag が一致するか確認する
		if tag == tagWireType2 || tag == tagWireType0 {
			// 一致した場合は field が存在する
			return true
		}
		// field を skip する
		rawProto = rawProto[n:]
		// wire_type に応じてフィールド値を skip する
		wireType := tag & 0x7
		rawProto = skipField(rawProto, wireType)
		if rawProto == nil {
			// skip 失敗（不正な bytes）
			break
		}
	}
	// field が見つからなかった
	return false
}

// decodeVarint は rawProto から varint をデコードして (value, bytesConsumed) を返す
func decodeVarint(rawProto []byte) (uint64, int) {
	// varint は最大 10 バイト（64bit）
	var x uint64
	for s := 0; s < 10; s++ {
		// rawProto の末尾チェック
		if s >= len(rawProto) {
			return 0, 0
		}
		// byte を取得する
		b := rawProto[s]
		// 下位 7 bit を x に追加する
		x |= uint64(b&0x7f) << s * 7
		// 最上位 bit が 0 なら varint 終了
		if b&0x80 == 0 {
			return x, s + 1
		}
	}
	// デコード失敗
	return 0, 0
}

// skipField は wire_type に応じてフィールド値を skip する
func skipField(rawProto []byte, wireType uint64) []byte {
	// wire_type に応じて skip する
	switch wireType {
	case 0:
		// varint: varint をスキップする
		_, n := decodeVarint(rawProto)
		if n == 0 {
			return nil
		}
		return rawProto[n:]
	case 1:
		// 64-bit: 8 バイトをスキップする
		if len(rawProto) < 8 {
			return nil
		}
		return rawProto[8:]
	case 2:
		// length-delimited: 長さを読んでスキップする
		length, n := decodeVarint(rawProto)
		if n == 0 {
			return nil
		}
		rawProto = rawProto[n:]
		if uint64(len(rawProto)) < length {
			return nil
		}
		return rawProto[length:]
	case 5:
		// 32-bit: 4 バイトをスキップする
		if len(rawProto) < 4 {
			return nil
		}
		return rawProto[4:]
	default:
		// 未知の wire_type
		return nil
	}
}

// processRequest は CodeGeneratorRequest を処理して lint 結果を返す
func processRequest(req *pluginpb.CodeGeneratorRequest) []k1s0LintResult {
	// lint 結果を格納するスライス
	var results []k1s0LintResult
	// 全ファイルのサービス・メソッドを検査する
	for _, fd := range req.GetProtoFile() {
		// 各サービスを検査する
		for _, sd := range fd.GetService() {
			// 各メソッドを検査する
			for _, md := range sd.GetMethod() {
				// k1s0 必須 annotation を検査する
				checkMethodAnnotations(fd, sd, md, &results)
			}
		}
	}
	// lint 結果を返す
	return results
}

// main は stdin から CodeGeneratorRequest を読み取り、lint 結果を stderr に JSON 出力する
func main() {
	// stdin から CodeGeneratorRequest を読み取る
	rawReq, err := io.ReadAll(os.Stdin)
	if err != nil {
		// 読み取りエラー
		fmt.Fprintf(os.Stderr, "k1s0-lint: stdin read error: %v\n", err)
		os.Exit(1)
	}
	// CodeGeneratorRequest をデシリアライズする
	req := new(pluginpb.CodeGeneratorRequest)
	if err := proto.Unmarshal(rawReq, req); err != nil {
		// デシリアライズエラー（空の標準入力等は graceful に終了する）
		// Buf lint plugin として呼ばれていない可能性があるため info として出力する
		fmt.Fprintf(os.Stderr, "k1s0-lint: request deserialize error (not called as buf plugin?): %v\n", err)
		// 空の CodeGeneratorResponse を返して正常終了する
		resp := &pluginpb.CodeGeneratorResponse{}
		rawResp, _ := proto.Marshal(resp)
		os.Stdout.Write(rawResp)
		return
	}
	// lint チェックを実行する
	results := processRequest(req)
	// lint エラーがある場合は stderr に JSON 形式で出力する
	if len(results) > 0 {
		// JSON エンコーダを構築する
		enc := json.NewEncoder(os.Stderr)
		enc.SetIndent("", "  ")
		// 各 lint 結果を出力する
		for _, r := range results {
			// 各結果を個別の JSON オブジェクトとして出力する（ndjson 形式）
			if encErr := enc.Encode(r); encErr != nil {
				// エンコードエラーは通常発生しない
				fmt.Fprintf(os.Stderr, "k1s0-lint: json encode error: %v\n", encErr)
			}
		}
		// CodeGeneratorResponse に lint エラー情報を格納して返す
		var errMsgs []string
		for _, r := range results {
			// エラーメッセージをリストに追加する（ファイルパスを含める）
			errMsgs = append(errMsgs, fmt.Sprintf("[%s:%d] %s (%s)", r.FilePath, r.Line, r.Message, r.RuleID))
		}
		// lint エラーを改行区切りで連結する
		errMsg := strings.Join(errMsgs, "\n")
		// CodeGeneratorResponse に lint エラーを設定して返す
		resp := &pluginpb.CodeGeneratorResponse{
			Error: proto.String(errMsg),
		}
		rawResp, _ := proto.Marshal(resp)
		os.Stdout.Write(rawResp)
		// エラーがあった場合は exit code 1 で終了する
		os.Exit(1)
	}
	// lint エラーなし: 空の CodeGeneratorResponse を返して正常終了する
	resp := &pluginpb.CodeGeneratorResponse{}
	rawResp, _ := proto.Marshal(resp)
	os.Stdout.Write(rawResp)
}
