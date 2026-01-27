# @k1s0/eslint-config

Shared ESLint configuration for k1s0 React/TypeScript projects.

## Installation

```bash
pnpm add -D @k1s0/eslint-config eslint typescript
```

## Available Configurations

| Configuration | Description | Use Case |
|--------------|-------------|----------|
| `@k1s0/eslint-config` | Full recommended config | React + TypeScript apps |
| `@k1s0/eslint-config/base` | Base JavaScript rules | Plain JS projects |
| `@k1s0/eslint-config/typescript` | TypeScript rules | TypeScript (non-React) |
| `@k1s0/eslint-config/react` | React + TypeScript rules | React applications |

## Usage

### Recommended (React + TypeScript)

Create an `eslint.config.js` (ESLint Flat Config) in your project:

```js
import k1s0Config from '@k1s0/eslint-config';

export default [
  {
    ...k1s0Config,
    languageOptions: {
      parserOptions: {
        project: './tsconfig.json',
      },
    },
  },
  {
    ignores: ['dist/', 'node_modules/', '*.config.js'],
  },
];
```

### TypeScript Only (No React)

```js
import { typescript } from '@k1s0/eslint-config';

export default [
  {
    ...typescript,
    languageOptions: {
      parserOptions: {
        project: './tsconfig.json',
      },
    },
  },
];
```

### Base JavaScript Only

```js
import { base } from '@k1s0/eslint-config';

export default [base];
```

## Legacy Configuration (.eslintrc)

For projects using the legacy `.eslintrc` format:

```json
{
  "extends": ["@k1s0/eslint-config"],
  "parserOptions": {
    "project": "./tsconfig.json"
  }
}
```

Or for TypeScript-only:

```json
{
  "extends": ["@k1s0/eslint-config/typescript"]
}
```

## Included Rules

### Base Rules
- ESLint recommended rules
- Import ordering and organization
- Prefer `const`, no `var`
- Template literals preference
- Console/debugger warnings

### TypeScript Rules
- `@typescript-eslint/recommended`
- `@typescript-eslint/recommended-type-checked`
- Consistent type imports/exports
- Proper promise handling
- Naming conventions

### React Rules
- `react/recommended`
- `react/jsx-runtime` (React 17+ automatic JSX)
- `react-hooks/recommended`
- Self-closing components
- Fragment syntax preference

### Accessibility Rules
- `jsx-a11y/recommended`
- Alt text requirements
- ARIA attribute validation
- Keyboard interaction support
- Focus management

### Prettier Integration
- `eslint-config-prettier` to disable conflicting rules
- Works seamlessly with Prettier

## Customization

Override rules in your ESLint config:

```js
import k1s0Config from '@k1s0/eslint-config';

export default [
  {
    ...k1s0Config,
    rules: {
      ...k1s0Config.rules,
      // Override specific rules
      'no-console': 'off',
      '@typescript-eslint/no-explicit-any': 'error',
    },
  },
];
```

## Dependencies

This package includes the following ESLint plugins as dependencies:

- `@typescript-eslint/eslint-plugin`
- `@typescript-eslint/parser`
- `eslint-config-prettier`
- `eslint-import-resolver-typescript`
- `eslint-plugin-import`
- `eslint-plugin-jsx-a11y`
- `eslint-plugin-react`
- `eslint-plugin-react-hooks`

## Peer Dependencies

- `eslint` ^8.57.0 or ^9.0.0
- `typescript` ^5.0.0 (optional, required for TypeScript rules)

## Related Packages

- [@k1s0/tsconfig](../tsconfig-k1s0) - TypeScript configuration
- [@k1s0/ui](../k1s0-ui) - UI components
- [@k1s0/api-client](../k1s0-api-client) - API client

## License

MIT
