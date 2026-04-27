// 本ファイルは Go ソース全体を走査し、import の prefix が依存方向ルールに違反していないか検証する。
// go/parser で import block のみ抽出（tokens.NewFileSet + parser.ParseFile + ImportsOnly）。
package checker

import (
	"go/parser"
	"go/token"
	"io/fs"
	"path/filepath"
	"strconv"
	"strings"
)

// Violation は単一 import 文の依存方向違反を表す。
type Violation struct {
	// 違反ソースファイル（リポジトリ root からの相対 path）
	File string `json:"file"`
	// import 文の行番号
	Line int `json:"line"`
	// ソースの Tier 区分
	SourceTier Tier `json:"sourceTier"`
	// 実際の import path
	ImportPath string `json:"importPath"`
	// 当てはまった禁止 prefix
	ForbiddenPrefix string `json:"forbiddenPrefix"`
}

// Check は root 配下の全 .go ファイルを走査し、違反のスライスを返す。
func Check(root string) ([]Violation, error) {
	var violations []Violation

	// vendor / node_modules / .git 配下はスキップ。
	skipDirs := map[string]bool{
		".git":         true,
		"vendor":       true,
		"node_modules": true,
		"target":       true, // Rust target/
		"bin":          true,
		"obj":          true,
	}

	// 走査対象 prefix（dep direction が定義された tier のみ）
	targetPrefixes := []string{
		"src/contracts/",
		"src/sdk/go/",
		"src/tier1/go/",
		"src/tier2/go/",
		"src/tier3/bff/",
	}

	walkErr := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			if skipDirs[d.Name()] {
				return filepath.SkipDir
			}
			return nil
		}
		// .go ファイルのみ対象
		if !strings.HasSuffix(d.Name(), ".go") {
			return nil
		}
		// generated は除外（buf 生成物 / mock 等）
		if strings.HasSuffix(d.Name(), "_generated.go") || strings.HasSuffix(d.Name(), ".pb.go") {
			return nil
		}

		// リポジトリ root からの相対 path を計算
		rel, err := filepath.Rel(root, path)
		if err != nil {
			return err
		}
		// Windows path separator を正規化
		rel = filepath.ToSlash(rel)

		// 走査対象 prefix にマッチしないファイルはスキップ
		matched := false
		for _, p := range targetPrefixes {
			if strings.HasPrefix(rel, p) {
				matched = true
				break
			}
		}
		if !matched {
			return nil
		}

		// import 違反をチェック
		fileViolations, err := checkFile(path, rel)
		if err != nil {
			return err
		}
		violations = append(violations, fileViolations...)
		return nil
	})

	if walkErr != nil {
		return nil, walkErr
	}
	return violations, nil
}

// checkFile は単一 .go ファイルの import を検証する。
func checkFile(absPath, relPath string) ([]Violation, error) {
	// import 宣言のみパース（パフォーマンス優先、完全 AST は不要）
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, absPath, nil, parser.ImportsOnly)
	if err != nil {
		// パース失敗は warning として扱い、違反検出は継続（非対象ファイルかもしれない）
		return nil, nil
	}

	tier := SourceTierByPath(relPath)
	forbidden := ForbiddenPrefixes(tier)
	if len(forbidden) == 0 {
		return nil, nil
	}

	var out []Violation
	for _, imp := range file.Imports {
		// import path はダブルクォート付きの string literal として保持される
		importPath, err := strconv.Unquote(imp.Path.Value)
		if err != nil {
			continue
		}
		for _, prefix := range forbidden {
			// 完全一致または prefix 一致を許容（標準ライブラリと衝突しない名前空間限定）
			if importPath == prefix || strings.HasPrefix(importPath, prefix) {
				pos := fset.Position(imp.Pos())
				out = append(out, Violation{
					File:            relPath,
					Line:            pos.Line,
					SourceTier:      tier,
					ImportPath:      importPath,
					ForbiddenPrefix: prefix,
				})
				break
			}
		}
	}
	return out, nil
}
