/**
 * @k1s0/eslint-config - Recommended Configuration
 *
 * This is the recommended configuration that includes all rules:
 * - Base JavaScript rules
 * - TypeScript rules
 * - React and JSX rules
 * - Accessibility rules
 * - Prettier integration
 *
 * Use this configuration for React + TypeScript projects.
 *
 * Compatible with ESLint 8 (legacy .eslintrc format).
 * For ESLint 9+ flat config, use the exported configurations directly.
 */

import baseConfig from './base.js';
import reactConfig from './react.js';
import typescriptConfig from './typescript.js';

// Default export is the full React + TypeScript config
export default reactConfig;

// Named exports for specific configurations
export { baseConfig as base };
export { typescriptConfig as typescript };
export { reactConfig as react };
