// 本ファイルは portal web アプリ向け Vite の `import.meta.env` 型補完。
// 環境別に注入される VITE_* 変数を strict に型付けし、コード側の参照ミスを防ぐ。
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BFF_URL: string;
  readonly VITE_TENANT_ID: string;
  readonly VITE_ENVIRONMENT: 'dev' | 'staging' | 'prod';
  readonly VITE_OTEL_COLLECTOR_URL: string;
  readonly VITE_KEYCLOAK_ISSUER: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
