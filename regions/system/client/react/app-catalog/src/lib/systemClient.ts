// system-client パッケージから再エクスポート（相対パス依存を排除）
export { AuthProvider } from 'system-client/auth/AuthProvider';
export { createApiClient } from 'system-client/http/apiClient';
export { ProtectedRoute } from 'system-client/routing/ProtectedRoute';
export { LoadingSpinner } from 'system-client/components/LoadingSpinner';
// アクセス拒否コンポーネント: 権限不足時のフォールバック表示用（M-27 監査対応）
export { AccessDenied } from 'system-client/components/AccessDenied';
// CRIT-2 監査対応: ErrorBoundary を再エクスポートする。App.tsx がこのエントリポイント経由でインポートするため必須。
export { ErrorBoundary } from 'system-client/components/ErrorBoundary';
export { useAuth } from 'system-client/auth/useAuth';
export type { User } from 'system-client/auth/AuthContext';
