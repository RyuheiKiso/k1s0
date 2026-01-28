# Frontend 依存関係

## Frontend 依存関係

```
React:
@k1s0/shell
  └── @k1s0/ui

@k1s0/navigation
  └── @k1s0/config

@k1s0/api-client
  └── (standalone)

@k1s0/config
  └── (standalone)

@k1s0/ui
  └── (standalone, Material-UI依存)

@k1s0/auth-client
  └── (standalone, jose依存)

@k1s0/observability
  └── @opentelemetry/api (optional)

Flutter:
k1s0_navigation
  ├── go_router
  └── flutter_riverpod

k1s0_config
  └── (standalone, yaml依存)

k1s0_http
  └── dio, k1s0_observability(optional)

k1s0_auth
  ├── flutter_secure_storage
  ├── jwt_decoder
  └── go_router(optional)

k1s0_observability
  └── (standalone)

k1s0_ui
  └── flutter_riverpod

k1s0_state
  ├── flutter_riverpod
  ├── shared_preferences
  └── hive_flutter
```

---

# eslint-config-k1s0

## 目的

k1s0 プロジェクト向けの ESLint 共通設定を提供する。TypeScript、React、アクセシビリティ、Prettier 連携、k1s0 固有ルールを統合。

## 設定ファイル

| ファイル | 説明 |
|---------|------|
| `index.js` | 推奨設定（React + TypeScript + k1s0 ルール） |
| `base.js` | JavaScript 基本ルール |
| `typescript.js` | TypeScript 固有ルール |
| `react.js` | React/JSX/a11y ルール |
| `k1s0-rules.js` | k1s0 プロジェクト固有ルール |

## k1s0 固有ルール

### 環境変数使用禁止

```javascript
// 禁止されるパターン:
process.env.API_URL          // NG
process.env['DATABASE_HOST'] // NG

// 正しいアプローチ:
import { useConfig } from '@k1s0/config';
const config = useConfig();
const apiUrl = config.api.baseUrl;
```

### 除外対象ファイル

以下のファイルでは環境変数使用が許可されます:

- `vite.config.ts`, `webpack.config.ts` 等のビルド設定
- `jest.config.ts`, `vitest.config.ts` 等のテスト設定
- `scripts/**` ディレクトリ
- テストファイル (`*.test.ts`, `*.spec.ts`)

## 使用例

```javascript
// .eslintrc.js
module.exports = {
  extends: ['@k1s0/eslint-config'],
};

// または特定の設定のみ使用
module.exports = {
  extends: ['@k1s0/eslint-config/react'],
};

// k1s0 ルールを個別に追加
import { k1s0Rules, k1s0Overrides } from '@k1s0/eslint-config/k1s0-rules';
```

---

# tsconfig-k1s0

## 目的

k1s0 プロジェクト向けの TypeScript 共通設定を提供する。厳格な型チェック、モジュール解決、各種プリセットを統合。

## 設定ファイル

| ファイル | 説明 |
|---------|------|
| `base.json` | 基本設定（厳格な型チェック） |
| `react.json` | React アプリケーション用 |
| `library.json` | ライブラリパッケージ用 |
| `node.json` | Node.js アプリケーション用 |
| `strict.json` | 最も厳格な設定（新規プロジェクト推奨） |

## 基本設定のコンパイラオプション

```json
{
  "compilerOptions": {
    // 厳格な型チェック
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "noPropertyAccessFromIndexSignature": true,
    "noFallthroughCasesInSwitch": true,
    "noImplicitReturns": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "allowUnreachableCode": false,
    "allowUnusedLabels": false,

    // モジュール
    "module": "ESNext",
    "moduleResolution": "bundler",
    "isolatedModules": true,
    "verbatimModuleSyntax": true,

    // 互換性
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

## Strict 設定の追加オプション

```json
{
  "extends": "./base.json",
  "compilerOptions": {
    "exactOptionalPropertyTypes": true,
    "noUncheckedSideEffectImports": true,
    "useUnknownInCatchVariables": true
  }
}
```

## 使用例

```json
// tsconfig.json (React アプリケーション)
{
  "extends": "@k1s0/tsconfig/react.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"]
}

// tsconfig.json (ライブラリ)
{
  "extends": "@k1s0/tsconfig/library.json",
  "compilerOptions": {
    "outDir": "dist"
  },
  "include": ["src"]
}

// tsconfig.json (最も厳格な設定)
{
  "extends": "@k1s0/tsconfig/strict.json"
}
```
