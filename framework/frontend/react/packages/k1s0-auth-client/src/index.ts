// Types
export type {
  Claims,
  OIDCConfig,
  TokenPair,
  TokenResult,
  AuthStatus,
  AuthState,
  AuthUser,
  AuthError,
  AuthErrorCode,
  TokenStorage,
  TokenRefresher,
  AuthClientConfig,
  SessionInfo,
  DeviceInfo,
  AuthGuardConfig,
} from './types.js';

export { ClaimsSchema } from './types.js';

// Token Management
export {
  SessionTokenStorage,
  LocalTokenStorage,
  MemoryTokenStorage,
  decodeToken,
  isTokenValid,
  needsRefresh,
  getTimeUntilExpiry,
  claimsToUser,
  TokenManager,
  type DecodeResult,
  type TokenManagerOptions,
} from './token/index.js';

// Auth Provider
export {
  AuthProvider,
  useAuth,
  useAuthState,
  useIsAuthenticated,
  useUser,
  usePermissions,
  type AuthContextValue,
} from './provider/index.js';

// Auth Guards
export {
  AuthGuard,
  RequireAuth,
  RequireRole,
  RequirePermission,
  withAuth,
  withRequireAuth,
  withRequireRole,
  withRequirePermission,
} from './guard/index.js';

// Session Management
export {
  SessionManager,
  useSession,
  type SessionManagerOptions,
  type UseSessionResult,
} from './session/index.js';
