// Auth
export { AuthContext } from './auth/AuthContext';
export type { AuthContextValue, User } from './auth/AuthContext';
export { AuthProvider } from './auth/AuthProvider';
// L-015 監査対応: i18n 対応のため AUTH_ERROR_MESSAGES と UseAuthOptions を公開する
export { useAuth, AUTH_ERROR_MESSAGES } from './auth/useAuth';
export type { UseAuthOptions } from './auth/useAuth';

// HTTP
export { createApiClient, setCsrfToken } from './http/apiClient';

// Routing
// L-004 監査対応: RolesCondition 型（AND/OR 混合条件）を公開する
export { ProtectedRoute } from './routing/ProtectedRoute';
export type { RolesCondition } from './routing/ProtectedRoute';

// Components
export { AppButton } from './components/AppButton';
export { LoadingSpinner } from './components/LoadingSpinner';
export { ErrorBoundary } from './components/ErrorBoundary';
// アクセス拒否コンポーネント: 権限不足時のフォールバック表示用（M-27 監査対応）
export { AccessDenied } from './components/AccessDenied';

// Config
export { ConfigEditorPage } from './config/ConfigEditorPage';
export { ConfigInterpreter } from './config/ConfigInterpreter';
export type {
  ConfigFieldType,
  ConfigFieldSchema,
  ConfigCategorySchema,
  ConfigEditorSchema,
  ConfigFieldValue,
  ConfigEditorConfig,
} from './config/types';

// Navigation
export { NavigationInterpreter } from './navigation/NavigationInterpreter';
export type { NavigationConfig, ResolvedRoute, RouterResult } from './navigation/NavigationInterpreter';
export { NavigationDevTools } from './navigation/devtools/NavigationDevTools';
export type {
  NavigationResponse,
  NavigationRoute,
  NavigationGuard,
  NavigationParam,
  ComponentRegistry,
  GuardType,
  TransitionType,
  ParamType,
} from './navigation/types';
