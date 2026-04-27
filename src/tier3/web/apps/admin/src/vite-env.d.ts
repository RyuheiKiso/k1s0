/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BFF_URL: string;
  readonly VITE_TENANT_ID: string;
  readonly VITE_ENVIRONMENT: 'dev' | 'staging' | 'prod';
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
