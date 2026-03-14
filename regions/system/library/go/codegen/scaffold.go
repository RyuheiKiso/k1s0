package codegen

import (
	"fmt"
	"os"
	"path/filepath"
	"text/template"
)

// ScaffoldConfig はコード生成の設定を保持する。
// ドメイン・ティア・サービス名・言語を指定してスキャフォールドを生成する。
type ScaffoldConfig struct {
	Domain      string
	Tier        string // "system", "business", "service"
	ServiceName string
	Language    string // "go", "rust", "typescript", "dart"
	OutputDir   string
}

// ScaffoldResult はコード生成の結果を保持する。
type ScaffoldResult struct {
	CreatedFiles []string
}

// Generate は ScaffoldConfig に基づいてスキャフォールドコードを生成する。
// 既存ファイルはスキップされる（冪等性）。
func Generate(cfg ScaffoldConfig) (*ScaffoldResult, error) {
	if err := validateConfig(cfg); err != nil {
		return nil, fmt.Errorf("codegen: invalid config: %w", err)
	}

	result := &ScaffoldResult{}

	switch cfg.Language {
	case "go":
		if err := generateGo(cfg, result); err != nil {
			return nil, err
		}
	case "rust":
		if err := generateRust(cfg, result); err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("codegen: unsupported language: %s", cfg.Language)
	}

	return result, nil
}

// validateConfig は ScaffoldConfig の必須フィールドを検証する。
func validateConfig(cfg ScaffoldConfig) error {
	if cfg.Domain == "" {
		return fmt.Errorf("domain is required")
	}
	if cfg.Tier == "" {
		return fmt.Errorf("tier is required")
	}
	if cfg.ServiceName == "" {
		return fmt.Errorf("service_name is required")
	}
	if cfg.Language == "" {
		return fmt.Errorf("language is required")
	}
	if cfg.OutputDir == "" {
		return fmt.Errorf("output_dir is required")
	}
	return nil
}

// generateGo は Go サービスのスキャフォールドファイルを生成する。
func generateGo(cfg ScaffoldConfig, result *ScaffoldResult) error {
	files := map[string]string{
		"main.go":              goMainTemplate,
		"go.mod":               goModTemplate,
		"internal/app/app.go": goAppTemplate,
	}

	for relPath, tmplStr := range files {
		outPath := filepath.Join(cfg.OutputDir, relPath)
		if err := writeTemplate(outPath, tmplStr, cfg, result); err != nil {
			return err
		}
	}
	return nil
}

// generateRust は Rust サービスのスキャフォールドファイルを生成する。
func generateRust(cfg ScaffoldConfig, result *ScaffoldResult) error {
	files := map[string]string{
		"src/main.rs": rustMainTemplate,
		"Cargo.toml":  rustCargoTemplate,
	}

	for relPath, tmplStr := range files {
		outPath := filepath.Join(cfg.OutputDir, relPath)
		if err := writeTemplate(outPath, tmplStr, cfg, result); err != nil {
			return err
		}
	}
	return nil
}

// writeTemplate はテンプレート文字列を評価してファイルに書き込む。
// 既存ファイルはスキップする。
func writeTemplate(outPath, tmplStr string, cfg ScaffoldConfig, result *ScaffoldResult) error {
	if _, err := os.Stat(outPath); err == nil {
		return nil // already exists, skip
	}

	if err := os.MkdirAll(filepath.Dir(outPath), 0755); err != nil {
		return fmt.Errorf("codegen: mkdir %s: %w", filepath.Dir(outPath), err)
	}

	tmpl, err := template.New("").Parse(tmplStr)
	if err != nil {
		return fmt.Errorf("codegen: parse template: %w", err)
	}

	f, err := os.Create(outPath)
	if err != nil {
		return fmt.Errorf("codegen: create file %s: %w", outPath, err)
	}
	defer f.Close()

	if err := tmpl.Execute(f, cfg); err != nil {
		return fmt.Errorf("codegen: execute template: %w", err)
	}

	result.CreatedFiles = append(result.CreatedFiles, outPath)
	return nil
}
