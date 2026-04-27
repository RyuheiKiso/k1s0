/// <reference types="vite/client" />

// Vite の import.meta.env への型補完。
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
