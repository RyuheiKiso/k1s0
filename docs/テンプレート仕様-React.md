# テンプレート仕様 — React

本ドキュメントは、[テンプレート仕様-クライアント](テンプレート仕様-クライアント.md) から分割された React テンプレートの詳細仕様である。

## 概要

k1s0 CLI の `ひな形生成 → client → react` で使用するテンプレートファイル群を定義する。テンプレートエンジン **Tera** の構文でパラメータ化されており、CLI の対話フローで収集した情報をもとに実用的なプロジェクトスケルトンを生成する。

| フレームワーク | 言語       | 用途               | テンプレートパス                  |
| -------------- | ---------- | ------------------ | --------------------------------- |
| React          | TypeScript | SPA（ブラウザ）    | `CLI/templates/client/react/`     |

### 配置制約

- **system 層には client を配置しない** — system は基盤提供が目的であり、ユーザー向け画面を持たない（[ディレクトリ構成図](ディレクトリ構成図.md) 参照）
- client は **business** および **service** Tier のみに配置する

### 認証方式

クライアントは BFF（Backend for Frontend）経由の **HttpOnly Cookie** 方式で認証を行う（[認証認可設計](認証認可設計.md) D-013 参照）。テンプレートの API クライアント設定は `withCredentials: true` を前提とする。

## 参照マップ

| テンプレートファイル                        | 参照ドキュメント                                  | 該当セクション                               |
| ------------------------------------------- | ------------------------------------------------- | -------------------------------------------- |
| `api-client.ts`                             | [認証認可設計](認証認可設計.md)                    | D-013 BFF + HttpOnly Cookie                  |
| `api-client.ts`（CSRF トークン）            | [認証認可設計](認証認可設計.md)                    | CSRF 対策                                    |
| `Dockerfile`                                | [Dockerイメージ戦略](Dockerイメージ戦略.md)        | ベースイメージ一覧・マルチステージビルド      |
| `nginx.conf`                                | [Dockerイメージ戦略](Dockerイメージ戦略.md)        | React クライアント                           |
| `vitest.config.ts` / `msw-setup.ts`         | [コーディング規約](コーディング規約.md)            | テストツール一覧（TypeScript）               |
| `eslint.config.mjs` / `.prettierrc`         | [コーディング規約](コーディング規約.md)            | TypeScript ツール・設定                      |
| `package.json`（変数展開）                  | [テンプレートエンジン仕様](テンプレートエンジン仕様.md) | テンプレート変数一覧・フィルタ               |
| `tests/testutil/setup.ts`                    | [コーディング規約](コーディング規約.md)            | Vitest セットアップ                          |
| `tests/App.test.tsx`                         | [コーディング規約](コーディング規約.md)            | コンポーネントテスト（Testing Library）      |
| `README.md`                                  | ---                                                | プロジェクト概要・セットアップ手順           |

---

## Tier 別配置パス

### business Tier

```
regions/business/{domain}/client/react/{service_name}/
```

例:

| domain       | service_name       | 配置パス                                                   |
| ------------ | ------------------ | ---------------------------------------------------------- |
| `accounting` | `ledger-app`       | `regions/business/accounting/client/react/ledger-app/`     |

### service Tier

```
regions/service/{service_name}/client/react/
```

例:

| service_name | 配置パス                                      |
| ------------ | --------------------------------------------- |
| `order`      | `regions/service/order/client/react/`          |

---

## React テンプレート

テンプレートファイルは `CLI/templates/client/react/` に配置する。以下に各ファイルの完全なスケルトンコードを示す。

### package.json

`CLI/templates/client/react/package.json.tera`

```json
{
  "name": "{{ service_name }}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write 'src/**/*.{ts,tsx,json,css}'",
    "format:check": "prettier --check 'src/**/*.{ts,tsx,json,css}'"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@tanstack/react-query": "^5.62.0",
    "@tanstack/react-router": "^1.92.0",
    "zustand": "^5.0.0",
    "react-hook-form": "^7.54.0",
    "@hookform/resolvers": "^3.9.0",
    "zod": "^3.24.0",
    "axios": "^1.7.0",
    "@radix-ui/react-dialog": "^1.1.0",
    "@radix-ui/react-dropdown-menu": "^2.1.0",
    "@radix-ui/react-label": "^2.1.0",
    "@radix-ui/react-slot": "^1.1.0",
    "@radix-ui/react-toast": "^1.2.0"
  },
  "devDependencies": {
    "typescript": "^5.7.0",
    "vite": "^6.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0",
    "vitest": "^2.1.0",
    "@testing-library/react": "^16.1.0",
    "@testing-library/jest-dom": "^6.6.0",
    "msw": "^2.7.0",
    "eslint": "^9.16.0",
    "@eslint/js": "^9.16.0",
    "typescript-eslint": "^8.18.0",
    "eslint-plugin-react": "^7.37.0",
    "eslint-plugin-react-hooks": "^5.1.0",
    "eslint-plugin-import": "^2.31.0",
    "prettier": "^3.4.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0"
  }
}
```

### tsconfig.json

`CLI/templates/client/react/tsconfig.json.tera`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "outDir": "./dist",
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### vite.config.ts

`CLI/templates/client/react/vite.config.ts.tera`

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
  },
});
```

### eslint.config.mjs

`CLI/templates/client/react/eslint.config.mjs.tera`

```javascript
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import importPlugin from 'eslint-plugin-import';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: { react, 'react-hooks': reactHooks, import: importPlugin },
    rules: {
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
      'import/order': [
        'error',
        {
          groups: ['builtin', 'external', 'internal', 'parent', 'sibling'],
          'newlines-between': 'always',
          alphabetize: { order: 'asc' },
        },
      ],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-floating-promises': 'error',
    },
  },
  {
    files: ['tests/**/*.{ts,tsx}'],
    languageOptions: {
      globals: {
        describe: 'readonly',
        it: 'readonly',
        expect: 'readonly',
        vi: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
      },
    },
  },
);
```

### .prettierrc

`CLI/templates/client/react/.prettierrc.tera`

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "tabWidth": 2
}
```

### vitest.config.ts

`CLI/templates/client/react/vitest.config.ts.tera`

```typescript
import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/testutil/setup.ts'],
    include: ['tests/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/**/*.d.ts', 'src/main.tsx'],
    },
  },
});
```

### src/app/App.tsx

`CLI/templates/client/react/src/app/App.tsx.tera`

```typescript
import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider, createRouter } from '@tanstack/react-router';

import { queryClient } from '@/lib/query-client';
import { routeTree } from '@/app/route-tree.gen';

const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
```

### src/lib/api-client.ts

`CLI/templates/client/react/src/lib/api-client.ts.tera`

BFF + HttpOnly Cookie 認証に対応した axios インスタンス。トークンは BFF がサーバーサイドで管理するため、クライアントから直接扱わない（[認証認可設計](認証認可設計.md) D-013 参照）。

```typescript
import axios from 'axios';

/**
 * API クライアント
 *
 * - baseURL: BFF のプロキシエンドポイント
 * - withCredentials: Cookie を自動送信（BFF + HttpOnly Cookie 方式）
 * - CSRF トークン: BFF が発行する X-CSRF-Token をリクエストヘッダーに付与
 */
const apiClient = axios.create({
  baseURL: '/api',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

// CSRF トークンをリクエストヘッダーに付与
apiClient.interceptors.request.use((config) => {
  const csrfToken = document.querySelector<HTMLMetaElement>(
    'meta[name="csrf-token"]',
  )?.content;
  if (csrfToken) {
    config.headers['X-CSRF-Token'] = csrfToken;
  }
  return config;
});

// レスポンスインターセプター: エラーハンドリング
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (axios.isAxiosError(error)) {
      switch (error.response?.status) {
        case 401:
          // 認証エラー: ログインページへリダイレクト
          window.location.href = '/auth/login';
          break;
        case 403:
          // 権限エラー: アクセス拒否
          console.error('アクセスが拒否されました');
          break;
        case 500:
          // サーバーエラー
          console.error('サーバーエラーが発生しました');
          break;
      }
    }
    return Promise.reject(error);
  },
);

export { apiClient };
```

### src/lib/query-client.ts

`CLI/templates/client/react/src/lib/query-client.ts.tera`

```typescript
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,       // 5 分間はキャッシュを新鮮とみなす
      gcTime: 1000 * 60 * 30,          // 30 分間キャッシュを保持
      retry: 1,                         // リトライ 1 回
      refetchOnWindowFocus: false,      // ウィンドウフォーカス時の再取得を無効化
    },
    mutations: {
      retry: 0,                         // ミューテーションはリトライしない
    },
  },
});
```

### tests/testutil/msw-setup.ts

`CLI/templates/client/react/tests/testutil/msw-setup.ts.tera`

```typescript
import '@testing-library/jest-dom/vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

/**
 * MSW ハンドラー定義
 *
 * テスト用の API モックハンドラーを定義する。
 * 各テストファイルで server.use() を使い、テスト固有のハンドラーを追加・上書きできる。
 */
const handlers = [
  // ヘルスチェック
  http.get('/api/health', () => {
    return HttpResponse.json({ status: 'ok' });
  }),

  // TODO: {{ service_name }} 固有のハンドラーを追加
];

export const server = setupServer(...handlers);

// テストライフサイクル
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### tests/testutil/setup.ts

`CLI/templates/client/react/tests/testutil/setup.ts.tera`

vitest のグローバルセットアップファイル。`vitest.config.ts` の `setupFiles` から参照される。MSW のセットアップを import し、テスト環境の初期化を一元管理する。

```typescript
/**
 * Vitest グローバルセットアップ
 *
 * vitest.config.ts の setupFiles から参照される。
 * テスト環境の初期化処理を一元管理する。
 */

// MSW（Mock Service Worker）セットアップの読み込み
import './msw-setup';
```

### tests/App.test.tsx

`CLI/templates/client/react/tests/App.test.tsx.tera`

App コンポーネントのスモークテスト。テスト環境が正しく動作することを検証する最小限のテスト。

```typescript
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { QueryClientProvider } from '@tanstack/react-query';

import { queryClient } from '@/lib/query-client';

describe('App', () => {
  it('renders without crashing', () => {
    render(
      <QueryClientProvider client={queryClient}>
        <div data-testid="app-root">{{ service_name_pascal }}</div>
      </QueryClientProvider>,
    );

    expect(screen.getByTestId('app-root')).toBeInTheDocument();
  });
});
```

### Dockerfile

`CLI/templates/client/react/Dockerfile.tera`

```dockerfile
# ---- Build ----
FROM node:22-bookworm AS build
WORKDIR /src

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# ---- Runtime ----
FROM nginx:1.27-alpine
COPY --from=build /src/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

# nginx のデフォルトユーザーは root のため、非 root 実行に切り替える。
# helm設計.md の securityContext（runAsUser: 65532）を使用する場合は
# Dockerfile 側で該当 UID のユーザーを作成し、nginx が listen する
# ポートを 1024 以上に変更する必要がある。
# 簡易的な非 root 化として USER nginx を使用する場合は、
# helm 側の runAsUser を nginx ユーザーの UID（101）に合わせること。
USER nginx
EXPOSE 8080
```

### nginx.conf

`CLI/templates/client/react/nginx.conf.tera`

```nginx
server {
    listen 8080;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    # gzip 圧縮
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_min_length 256;
    gzip_types
        text/plain
        text/css
        text/javascript
        application/javascript
        application/json
        application/xml
        image/svg+xml;

    # SPA ルーティング: 存在しないパスは index.html にフォールバック
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静的アセットのキャッシュ制御
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # index.html はキャッシュしない（常に最新版を配信）
    location = /index.html {
        add_header Cache-Control "no-cache, no-store, must-revalidate";
        add_header Pragma "no-cache";
        add_header Expires "0";
    }

    # セキュリティヘッダー
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
}
```

### README.md

`CLI/templates/client/react/README.md.tera`

```markdown
# {{ service_name }}

{{ service_name_pascal }} クライアント（React SPA）。

## セットアップ

```bash
# 依存インストール
npm install

# 開発サーバー起動
npm run dev

# テスト実行
npm test

# ビルド
npm run build
```

## ディレクトリ構成

```
.
├── src/
│   ├── app/              # アプリケーションルート・ルーティング
│   └── lib/              # API クライアント・ユーティリティ
├── tests/
│   └── testutil/         # テストユーティリティ（MSW セットアップ等）
├── package.json
├── vite.config.ts
├── vitest.config.ts
├── Dockerfile
└── README.md
```

## 開発

- **ビルドツール**: Vite
- **テスト**: Vitest + Testing Library + MSW
- **Linter**: ESLint (Flat Config)
- **Formatter**: Prettier
```

---

## 関連ドキュメント

- [テンプレート仕様-Flutter](テンプレート仕様-Flutter.md) --- Flutter テンプレート
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) --- 変数置換・条件分岐・フィルタの仕様
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) --- サーバーテンプレート
- [テンプレート仕様-ライブラリ](テンプレート仕様-ライブラリ.md) --- ライブラリテンプレート
- [テンプレート仕様-データベース](テンプレート仕様-データベース.md) --- データベーステンプレート
- [CLIフロー](CLIフロー.md) --- CLI の対話フローと操作手順
- [ディレクトリ構成図](ディレクトリ構成図.md) --- 生成先ディレクトリ構成
- [Dockerイメージ戦略](Dockerイメージ戦略.md) --- Docker ビルド戦略
- [認証認可設計](認証認可設計.md) --- BFF + Cookie 認証
- [コーディング規約](コーディング規約.md) --- Linter・テストツール
