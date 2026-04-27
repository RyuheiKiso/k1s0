// 本ファイルは Cargo.toml を走査し、path 依存の依存方向違反を検出する。
package checker

import (
	"io/fs"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
)

// Violation は単一 path 依存の違反を表す。
type Violation struct {
	// 違反元 Cargo.toml（リポジトリ root からの相対 path）
	CargoToml string `json:"cargoToml"`
	// 違反元 crate の Tier
	SourceTier Tier `json:"sourceTier"`
	// 解決された path 依存先（リポジトリ root からの相対 path）
	ResolvedPath string `json:"resolvedPath"`
	// 解決先 path の Tier
	TargetTier Tier `json:"targetTier"`
	// 依存名（key）
	DepName string `json:"depName"`
}

// Check は root 配下の全 Cargo.toml を走査して違反のスライスを返す。
func Check(root string) ([]Violation, error) {
	var violations []Violation

	skipDirs := map[string]bool{
		".git":         true,
		"target":       true,
		"node_modules": true,
		"vendor":       true,
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
		// Cargo.toml のみ対象
		if d.Name() != "Cargo.toml" {
			return nil
		}
		fileViolations, err := checkCargoToml(root, path)
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

// cargoManifest は Cargo.toml の最小スキーマ（path 依存検出のみ）。
type cargoManifest struct {
	Package          map[string]interface{}            `toml:"package"`
	Dependencies     map[string]toml.Primitive         `toml:"dependencies"`
	DevDependencies  map[string]toml.Primitive         `toml:"dev-dependencies"`
	BuildDeps        map[string]toml.Primitive         `toml:"build-dependencies"`
	WorkspaceDeps    map[string]toml.Primitive         `toml:"workspace.dependencies"`
	Workspace        *cargoWorkspace                   `toml:"workspace"`
	Target           map[string]map[string]interface{} `toml:"target"`
}

type cargoWorkspace struct {
	Members      []string                  `toml:"members"`
	Exclude      []string                  `toml:"exclude"`
	Dependencies map[string]toml.Primitive `toml:"dependencies"`
}

// checkCargoToml は単一 Cargo.toml を解析し、path 依存違反のスライスを返す。
func checkCargoToml(root, absPath string) ([]Violation, error) {
	var manifest cargoManifest
	meta, err := toml.DecodeFile(absPath, &manifest)
	if err != nil {
		// パースできない Cargo.toml は warning として skip
		return nil, nil
	}

	rel, err := filepath.Rel(root, absPath)
	if err != nil {
		return nil, err
	}
	rel = filepath.ToSlash(rel)
	srcTier := SourceTierByPath(rel)
	if srcTier == TierUnknown {
		return nil, nil
	}

	cargoDir := filepath.Dir(absPath)
	var out []Violation

	// 検査対象の dep table を集める。
	tables := []struct {
		section string
		deps    map[string]toml.Primitive
	}{
		{"dependencies", manifest.Dependencies},
		{"dev-dependencies", manifest.DevDependencies},
		{"build-dependencies", manifest.BuildDeps},
	}
	if manifest.Workspace != nil {
		tables = append(tables, struct {
			section string
			deps    map[string]toml.Primitive
		}{"workspace.dependencies", manifest.Workspace.Dependencies})
	}

	for _, tbl := range tables {
		for name, prim := range tbl.deps {
			// dep の値が map（テーブル）形式の場合、`path` キーを抽出する。
			var depTable map[string]interface{}
			if err := meta.PrimitiveDecode(prim, &depTable); err != nil {
				// 文字列 dep（例: `clap = "4"`）は path 依存ではないので skip
				continue
			}
			pathStr, ok := depTable["path"].(string)
			if !ok || pathStr == "" {
				continue
			}
			// 解決先 path を絶対 → リポジトリ root 相対へ変換
			resolved, err := filepath.Abs(filepath.Join(cargoDir, pathStr))
			if err != nil {
				continue
			}
			resolvedRel, err := filepath.Rel(root, resolved)
			if err != nil {
				continue
			}
			resolvedRel = filepath.ToSlash(resolvedRel)
			tgtTier := SourceTierByPath(resolvedRel)
			if !IsAllowed(srcTier, tgtTier) {
				out = append(out, Violation{
					CargoToml:    rel,
					SourceTier:   srcTier,
					ResolvedPath: resolvedRel,
					TargetTier:   tgtTier,
					DepName:      name,
				})
			}
			_ = strings.HasPrefix // 利用される変数として retain
		}
	}
	return out, nil
}
