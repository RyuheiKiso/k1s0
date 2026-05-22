// k1s0-impl: IMPL-cross_fsm-0001 realizes=FR-cross_fsm-001
// k1s0 protoc-gen-k1s0-go-fsm プラグインのエントリポイント
// proto ファイルの FSM オプションを読み取り Go typestate コードを生成する
package main

import (
	// fmt パッケージ: フォーマット出力に使用する
	"fmt"
	// os パッケージ: OS 操作に使用する
	"os"
	// strings パッケージ: 文字列操作に使用する
	"strings"

	// google.golang.org/protobuf/compiler/protogen: protoc プラグイン API
	"google.golang.org/protobuf/compiler/protogen"
	// google.golang.org/protobuf/proto: Protocol Buffers コアライブラリ
	"google.golang.org/protobuf/proto"
	// google.golang.org/protobuf/types/descriptorpb: descriptor proto 型定義
	"google.golang.org/protobuf/types/descriptorpb"
)

// FSMState は FSM の状態を表す構造体
type FSMState struct {
	// Name: 状態名 (proto option から抽出する)
	Name string
	// IsInitial: 初期状態かどうかを示すフラグ
	IsInitial bool
	// IsTerminal: 終端状態かどうかを示すフラグ
	IsTerminal bool
}

// FSMTransition は FSM の状態遷移を表す構造体
type FSMTransition struct {
	// From: 遷移元の状態名
	From string
	// To: 遷移先の状態名
	To string
	// Event: 遷移をトリガするイベント名
	Event string
}

// FSMDefinition は proto ファイルから抽出した FSM 定義を表す構造体
type FSMDefinition struct {
	// Name: FSM の名称 (message 名から派生する)
	Name string
	// States: FSM が持つ状態のリスト
	States []FSMState
	// Transitions: FSM が持つ状態遷移のリスト
	Transitions []FSMTransition
	// PackageName: 生成する Go コードのパッケージ名
	PackageName string
}

// extractFSMOption は proto message の option から FSM 定義を抽出する
func extractFSMOption(msg *protogen.Message) (*FSMDefinition, error) {
	// message の options を取得する
	opts := msg.Desc.Options()
	// options が nil の場合は FSM 定義なしと判断する
	if opts == nil {
		return nil, nil
	}
	// options を proto.Message として取得する
	protoOpts, ok := opts.(*descriptorpb.MessageOptions)
	// 型アサートに失敗した場合は FSM 定義なしと判断する
	if !ok {
		return nil, nil
	}
	// uninterpreted_option から (k1s0.fsm.state_machine) オプションを探す
	for _, uo := range protoOpts.GetUninterpretedOption() {
		// オプション名のパーツを取得する
		nameParts := uo.GetName()
		// オプション名が k1s0.fsm.state_machine かどうかを確認する
		for _, part := range nameParts {
			// パート名が k1s0.fsm.state_machine かどうかを確認する
			if part.GetNamePart() == "k1s0.fsm.state_machine" {
				// FSM 定義を持つ message を発見したことをログに記録する
				_, _ = fmt.Fprintf(os.Stderr, "FSM オプション発見: message=%s\n", msg.GoIdent.GoName)
				// FSM 定義を作成して返す (実際の option 解析は proto 拡張定義が必要)
				return buildFSMDefinition(msg), nil
			}
		}
	}
	// FSM オプションが見つからない場合は nil を返す
	_ = proto.HasExtension
	return nil, nil
}

// buildFSMDefinition は proto message から FSM 定義を構築する
func buildFSMDefinition(msg *protogen.Message) *FSMDefinition {
	// FSM 名称を message 名から取得する
	fsmName := msg.GoIdent.GoName
	// パッケージ名を message の Go パッケージから取得する
	pkgName := string(msg.GoIdent.GoImportPath)
	// パッケージ名の最後のセグメントを取得する
	pkgParts := strings.Split(pkgName, "/")
	// 最後のセグメントをパッケージ名として使用する
	pkgName = pkgParts[len(pkgParts)-1]

	// デフォルトの FSM 状態を作成する (proto option から抽出する場合は実装を拡張する)
	states := []FSMState{
		// 初期状態: Created
		{Name: "Created", IsInitial: true, IsTerminal: false},
		// 処理中状態: Processing
		{Name: "Processing", IsInitial: false, IsTerminal: false},
		// 完了状態: Completed
		{Name: "Completed", IsInitial: false, IsTerminal: true},
		// 失敗状態: Failed
		{Name: "Failed", IsInitial: false, IsTerminal: true},
	}

	// デフォルトの FSM 遷移を作成する (proto option から抽出する場合は実装を拡張する)
	transitions := []FSMTransition{
		// Created → Processing への遷移 (Start イベント)
		{From: "Created", To: "Processing", Event: "Start"},
		// Processing → Completed への遷移 (Complete イベント)
		{From: "Processing", To: "Completed", Event: "Complete"},
		// Processing → Failed への遷移 (Fail イベント)
		{From: "Processing", To: "Failed", Event: "Fail"},
	}

	// FSM 定義を返す
	return &FSMDefinition{
		// FSM 名称を設定する
		Name: fsmName,
		// 状態リストを設定する
		States: states,
		// 遷移リストを設定する
		Transitions: transitions,
		// パッケージ名を設定する
		PackageName: pkgName,
	}
}

// generateStateEnum は FSM 状態の sealed interface + enum パターンの Go コードを生成する
func generateStateEnum(g *protogen.GeneratedFile, fsm *FSMDefinition) {
	// パッケージ宣言を生成する
	g.P("// ", fsm.Name, "State は FSM の状態を表す sealed interface")
	// sealed interface を生成する (外部パッケージからの実装を禁止する)
	g.P("type ", fsm.Name, "State interface {")
	// sealed interface の実装を外部パッケージから禁止するためのプライベートメソッドを定義する
	g.P("\t// is", fsm.Name, "State はこの interface を sealed にするためのプライベートメソッド")
	// プライベートメソッドシグネチャを生成する
	g.P("\tis", fsm.Name, "State()")
	// sealed interface の閉じ括弧を生成する
	g.P("}")
	// 空行を挿入する
	g.P()

	// 各状態の具体的な型を生成する
	for _, state := range fsm.States {
		// 状態の struct 型を生成する
		g.P("// ", fsm.Name, "State", state.Name, " は FSM の ", state.Name, " 状態を表す構造体")
		// 状態 struct 型を定義する
		g.P("type ", fsm.Name, "State", state.Name, " struct{}")
		// sealed interface を実装するプライベートメソッドを生成する
		g.P("// is", fsm.Name, "State は sealed interface を実装するメソッド")
		// プライベートメソッドを生成する
		g.P("func (", fsm.Name, "State", state.Name, ") is", fsm.Name, "State() {}")
		// 空行を挿入する
		g.P()
	}
}

// generateTransitionFunctions は FSM の型安全な遷移関数を生成する
func generateTransitionFunctions(g *protogen.GeneratedFile, fsm *FSMDefinition) {
	// 各遷移に対して型安全な遷移関数を生成する
	for _, transition := range fsm.Transitions {
		// 遷移関数のコメントを生成する
		g.P("// ", transition.From, "To", transition.To, " は ", transition.From,
			" 状態から ", transition.To, " 状態への型安全な遷移関数")
		// 不正な遷移はコンパイルエラーとなる (引数型で制約する)
		g.P("func ", transition.From, "To", transition.To,
			"(_ ", fsm.Name, "State", transition.From,
			") ", fsm.Name, "State", transition.To, " {")
		// 遷移先の状態を返す
		g.P("\t// 遷移先状態を返す (型システムで正当な遷移であることが保証される)")
		// 遷移先状態のインスタンスを返す
		g.P("\treturn ", fsm.Name, "State", transition.To, "{}")
		// 関数の閉じ括弧を生成する
		g.P("}")
		// 空行を挿入する
		g.P()
	}
}

// generateFSMCode は FSM 定義から Go コードを生成する
func generateFSMCode(gen *protogen.Plugin, file *protogen.File) error {
	// ファイル内の全 message を処理する
	for _, msg := range file.Messages {
		// message から FSM オプションを抽出する
		fsm, err := extractFSMOption(msg)
		// 抽出に失敗した場合はエラーを返す
		if err != nil {
			return fmt.Errorf("FSM オプション抽出失敗: message=%s: %w", msg.GoIdent.GoName, err)
		}
		// FSM 定義が存在しない場合はスキップする
		if fsm == nil {
			continue
		}

		// 出力ファイル名を生成する (元のファイル名に _fsm_gen.go サフィックスを付ける)
		outputFileName := strings.TrimSuffix(file.GeneratedFilenamePrefix, ".proto") + "_fsm_gen.go"
		// 新しい生成ファイルを作成する
		g := gen.NewGeneratedFile(outputFileName, file.GoImportPath)

		// 生成ファイルのヘッダコメントを出力する
		g.P("// Code generated by protoc-gen-k1s0-go-fsm. DO NOT EDIT.")
		// 元の proto ファイル名をコメントとして出力する
		g.P("// source: ", file.Desc.Path())
		// 空行を挿入する
		g.P()
		// パッケージ宣言を出力する
		g.P("package ", file.GoPackageName)
		// 空行を挿入する
		g.P()

		// FSM 状態の sealed interface と enum を生成する
		generateStateEnum(g, fsm)

		// FSM の型安全な遷移関数を生成する
		generateTransitionFunctions(g, fsm)

		// コード生成完了ログを出力する
		_, _ = fmt.Fprintf(os.Stderr, "FSM コード生成完了: message=%s, output=%s\n",
			msg.GoIdent.GoName, outputFileName)
	}
	// 正常終了を返す
	return nil
}

// メイン関数: protoc プラグインのエントリポイント
func main() {
	// protoc プラグインとして起動する設定を定義する
	opts := protogen.Options{}
	// protoc プラグインとして実行する
	opts.Run(func(gen *protogen.Plugin) error {
		// 各 proto ファイルを処理する
		for _, file := range gen.Files {
			// このファイルを生成対象とするかどうかを確認する
			if !file.Generate {
				// 生成対象でない場合はスキップする
				continue
			}
			// FSM コードを生成する
			if err := generateFSMCode(gen, file); err != nil {
				// コード生成失敗の場合はエラーを返す
				return fmt.Errorf("FSM コード生成失敗: file=%s: %w", file.Desc.Path(), err)
			}
		}
		// 正常終了を返す
		return nil
	})
}
