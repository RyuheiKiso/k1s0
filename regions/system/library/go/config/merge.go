package config

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// mergeFromFile は envPath の YAML を読み込み、cfg にマージする。
func mergeFromFile(cfg *Config, envPath string) error {
	envData, err := os.ReadFile(envPath)
	if err != nil {
		return fmt.Errorf("failed to read env config: %w", err)
	}

	// base を YAML Value として再パースする
	baseData, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal base config: %w", err)
	}

	var baseValue map[string]interface{}
	if err := yaml.Unmarshal(baseData, &baseValue); err != nil {
		return fmt.Errorf("failed to parse base config: %w", err)
	}

	var envValue map[string]interface{}
	if err := yaml.Unmarshal(envData, &envValue); err != nil {
		return fmt.Errorf("failed to parse env config: %w", err)
	}

	merged := deepMerge(baseValue, envValue)

	mergedData, err := yaml.Marshal(merged)
	if err != nil {
		return fmt.Errorf("failed to marshal merged config: %w", err)
	}

	if err := yaml.Unmarshal(mergedData, cfg); err != nil {
		return fmt.Errorf("failed to parse merged config: %w", err)
	}

	return nil
}

// deepMerge は base マップに overlay マップを再帰的にマージする。
func deepMerge(base, overlay map[string]interface{}) map[string]interface{} {
	result := make(map[string]interface{})
	for k, v := range base {
		result[k] = v
	}
	for k, v := range overlay {
		if baseVal, ok := result[k]; ok {
			baseMap, baseIsMap := baseVal.(map[string]interface{})
			overlayMap, overlayIsMap := v.(map[string]interface{})
			if baseIsMap && overlayIsMap {
				result[k] = deepMerge(baseMap, overlayMap)
				continue
			}
		}
		result[k] = v
	}
	return result
}
