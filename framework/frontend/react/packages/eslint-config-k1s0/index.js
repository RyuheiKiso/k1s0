/**
 * @k1s0/eslint-config - Recommended Configuration
 *
 * This is the recommended configuration that includes all rules:
 * - Base JavaScript rules
 * - TypeScript rules
 * - React and JSX rules
 * - Accessibility rules
 * - Prettier integration
 * - k1s0 project-specific rules (environment variable prohibition, etc.)
 *
 * Use this configuration for React + TypeScript projects.
 *
 * Compatible with ESLint 8 (legacy .eslintrc format).
 * For ESLint 9+ flat config, use the exported configurations directly.
 */

import baseConfig from './base.js';
import reactConfig from './react.js';
import typescriptConfig from './typescript.js';
import k1s0RulesConfig, { k1s0Rules, k1s0Overrides } from './k1s0-rules.js';

/**
 * Create a merged configuration that combines the React config with k1s0 rules.
 *
 * @param {import('eslint').Linter.Config} baseConfig - The base configuration to extend
 * @returns {import('eslint').Linter.Config} The merged configuration
 */
function createK1s0Config(baseConfig) {
  return {
    ...baseConfig,
    rules: {
      ...baseConfig.rules,
      ...k1s0Rules,
    },
    overrides: [
      ...(baseConfig.overrides || []),
      ...k1s0Overrides,
    ],
  };
}

// Full k1s0 configuration (React + TypeScript + k1s0 rules)
const k1s0FullConfig = createK1s0Config(reactConfig);

// Default export is the full React + TypeScript + k1s0 config
export default k1s0FullConfig;

// Named exports for specific configurations
export { baseConfig as base };
export { typescriptConfig as typescript };
export { reactConfig as react };

// k1s0 rules exports
export { k1s0RulesConfig as k1s0 };
export { k1s0Rules, k1s0Overrides };

// Helper function export
export { createK1s0Config };
