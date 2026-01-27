# @k1s0/tsconfig

Shared TypeScript configuration for k1s0 projects.

## Installation

```bash
pnpm add -D @k1s0/tsconfig typescript
```

## Available Configurations

| Configuration | Description | Use Case |
|--------------|-------------|----------|
| `@k1s0/tsconfig/base.json` | Base strict configuration | All projects (foundation) |
| `@k1s0/tsconfig/react.json` | React application config | React apps with bundlers |
| `@k1s0/tsconfig/library.json` | Library/package config | Shared packages |
| `@k1s0/tsconfig/node.json` | Node.js application config | Backend/CLI tools |

## Usage

### React Application

```json
{
  "extends": "@k1s0/tsconfig/react.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### Shared Library/Package

```json
{
  "extends": "@k1s0/tsconfig/library.json",
  "compilerOptions": {
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts", "**/*.test.tsx"]
}
```

### Node.js Application

```json
{
  "extends": "@k1s0/tsconfig/node.json",
  "compilerOptions": {
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### Base Configuration Only

```json
{
  "extends": "@k1s0/tsconfig/base.json",
  "compilerOptions": {
    "lib": ["ES2022", "DOM"],
    "jsx": "react-jsx"
  }
}
```

## Configuration Details

### Base Configuration

The base configuration includes strict TypeScript settings:

```json
{
  "target": "ES2022",
  "module": "ESNext",
  "moduleResolution": "bundler",
  "strict": true,
  "noUncheckedIndexedAccess": true,
  "noImplicitOverride": true,
  "noPropertyAccessFromIndexSignature": true,
  "verbatimModuleSyntax": true,
  "isolatedModules": true,
  "esModuleInterop": true,
  "skipLibCheck": true
}
```

### Key Features

#### Strict Type Checking
- `strict: true` - Enables all strict type-checking options
- `noUncheckedIndexedAccess` - Adds `undefined` to index signatures
- `noImplicitOverride` - Requires `override` keyword for overridden methods
- `noPropertyAccessFromIndexSignature` - Requires bracket notation for index signatures

#### Modern Module System
- `moduleResolution: "bundler"` - Optimized for modern bundlers (Vite, esbuild)
- `verbatimModuleSyntax` - Enforces explicit type-only imports
- `isolatedModules` - Ensures compatibility with transpilers

#### Library Configuration
- Emits declaration files (`.d.ts`)
- Includes declaration maps for debugging
- Source maps for development

#### React Configuration
- `jsx: "react-jsx"` - Uses React 17+ automatic JSX transform
- DOM and DOM.Iterable libs included
- No emit (bundler handles output)

#### Node.js Configuration
- `moduleResolution: "NodeNext"` - Native Node.js ES modules
- `module: "NodeNext"` - Proper ESM support for Node.js
- Includes `@types/node`

## Path Aliases

Configure path aliases in your project's tsconfig:

```json
{
  "extends": "@k1s0/tsconfig/react.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"],
      "@components/*": ["./src/components/*"],
      "@hooks/*": ["./src/hooks/*"],
      "@utils/*": ["./src/utils/*"]
    }
  }
}
```

Note: Path aliases also need to be configured in your bundler (Vite, webpack, etc.).

### Vite Configuration Example

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

## Extending Configurations

You can override any settings in your project's tsconfig:

```json
{
  "extends": "@k1s0/tsconfig/react.json",
  "compilerOptions": {
    "target": "ES2020",
    "noUncheckedIndexedAccess": false
  }
}
```

## Related Packages

- [@k1s0/eslint-config](../eslint-config-k1s0) - ESLint configuration
- [@k1s0/ui](../k1s0-ui) - UI components
- [@k1s0/api-client](../k1s0-api-client) - API client

## License

MIT
