// Client
export {
  type ApiClientConfig,
  type RequestOptions,
  type ApiResponse,
  type RetryPolicy,
  DEFAULT_RETRY_POLICY,
  DEFAULT_TIMEOUT,
  ApiClient,
  createApiClient,
  ApiClientProvider,
  useApiClient,
  PublicApiClientProvider,
  type ApiRequestState,
  type UseApiRequestResult,
  type UseMutationResult,
  useApiRequest,
  useMutation,
} from './client/index.js';

// Error
export {
  type ProblemDetails,
  ProblemDetailsSchema,
  type ApiErrorKind,
  mapStatusToErrorKind,
  isRetryableError,
  getDefaultErrorMessage,
  ApiError,
} from './error/index.js';

// Auth
export {
  type TokenPair,
  type TokenResult,
  type AuthState,
  type TokenStorage,
  type TokenRefresher,
  type AuthConfig,
  TokenManager,
  SessionTokenStorage,
  LocalTokenStorage,
  AuthProvider,
  useAuth,
  useAuthState,
  useIsAuthenticated,
} from './auth/index.js';

// Telemetry
export {
  type RequestTelemetry,
  type TelemetryEventType,
  type TelemetryEvent,
  type TelemetryListener,
  ApiTelemetry,
  defaultTelemetry,
} from './telemetry/index.js';

// Throttle
export {
  type ThrottleConfig,
  DEFAULT_THROTTLE_CONFIG,
  RequestThrottle,
} from './throttle.js';

// UI
export {
  ErrorDisplay,
  InlineError,
  ApiErrorBoundary,
  withErrorBoundary,
  AsyncContent,
  DataLoader,
  SkeletonLoader,
} from './ui/index.js';
