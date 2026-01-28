/**
 * @k1s0/eslint-config - k1s0 Project-Specific Rules
 *
 * This module provides k1s0 project-specific ESLint rules:
 * - Prohibition of environment variable usage (process.env)
 * - Restriction on certain insecure patterns
 *
 * These rules enforce k1s0 development conventions.
 */

/**
 * k1s0 specific rules that extend the base configuration.
 *
 * @type {import('eslint').Linter.RulesRecord}
 */
export const k1s0Rules = {
  // ==========================================================================
  // Environment Variable Prohibition (K020-equivalent)
  // ==========================================================================
  //
  // k1s0 projects must not use environment variables directly.
  // Instead, use config/*.yaml files with @k1s0/config library.
  //
  // Prohibited patterns:
  // - process.env.VARIABLE_NAME
  // - process.env['VARIABLE_NAME']
  //
  // Correct approach:
  // import { useConfig } from '@k1s0/config';
  // const config = useConfig();
  // const apiUrl = config.api.baseUrl;
  'no-restricted-syntax': [
    'error',
    {
      selector: "MemberExpression[object.object.name='process'][object.property.name='env']",
      message:
        'Direct access to process.env is prohibited in k1s0 projects. Use @k1s0/config instead. See docs/conventions/config-and-secrets.md',
    },
    {
      selector: "MemberExpression[object.name='process'][property.name='env']",
      message:
        'Access to process.env is prohibited in k1s0 projects. Use @k1s0/config instead. See docs/conventions/config-and-secrets.md',
    },
  ],

  // ==========================================================================
  // Hardcoded Secrets Prevention
  // ==========================================================================
  //
  // Prevent common patterns that might indicate hardcoded secrets.
  // This is a best-effort detection and doesn't replace proper secret scanning.
  'no-restricted-properties': [
    'error',
    {
      object: 'process',
      property: 'env',
      message:
        'Direct access to process.env is prohibited. Use @k1s0/config for configuration management.',
    },
  ],
};

/**
 * k1s0 ESLint overrides for specific file patterns.
 *
 * @type {import('eslint').Linter.ConfigOverride[]}
 */
export const k1s0Overrides = [
  // Allow process.env in config loaders and build scripts
  {
    files: [
      '**/config-loader.ts',
      '**/config-loader.js',
      '**/vite.config.ts',
      '**/vite.config.js',
      '**/webpack.config.ts',
      '**/webpack.config.js',
      '**/next.config.ts',
      '**/next.config.js',
      '**/jest.config.ts',
      '**/jest.config.js',
      '**/vitest.config.ts',
      '**/vitest.config.js',
      '**/*.config.ts',
      '**/*.config.js',
      '**/scripts/**',
      '**/test/**',
      '**/tests/**',
      '**/__tests__/**',
      '**/*.test.ts',
      '**/*.test.tsx',
      '**/*.spec.ts',
      '**/*.spec.tsx',
    ],
    rules: {
      'no-restricted-syntax': 'off',
      'no-restricted-properties': 'off',
    },
  },
];

/**
 * Full k1s0 rules configuration to merge with base config.
 *
 * Usage:
 * ```js
 * import { k1s0Rules, k1s0Overrides } from '@k1s0/eslint-config/k1s0-rules';
 *
 * export default {
 *   ...baseConfig,
 *   rules: {
 *     ...baseConfig.rules,
 *     ...k1s0Rules,
 *   },
 *   overrides: [
 *     ...baseConfig.overrides,
 *     ...k1s0Overrides,
 *   ],
 * };
 * ```
 */
export default {
  rules: k1s0Rules,
  overrides: k1s0Overrides,
};
