// 本ファイルは admin web アプリ向け Vite の `import.meta.env` 型補完。
// Vite client 型を拡張して環境別変数（VITE_BFF_URL 等）を strict に型付けする。
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BFF_URL: string;
  readonly VITE_TENANT_ID: string;
  readonly VITE_ENVIRONMENT: 'dev' | 'staging' | 'prod';
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
