/// k1s0 HTTP Client Library
///
/// Provides a Dio-based HTTP client with interceptors for authentication,
/// logging, error handling, and OpenTelemetry tracing support.
library k1s0_http;

export 'src/client/http_client.dart';
export 'src/client/http_client_config.dart';
export 'src/error/api_error.dart';
export 'src/error/problem_details.dart';
export 'src/interceptors/auth_interceptor.dart';
export 'src/interceptors/error_interceptor.dart';
export 'src/interceptors/logging_interceptor.dart';
export 'src/interceptors/trace_interceptor.dart';
export 'src/types/request_options.dart';
export 'src/types/response.dart';
export 'src/throttle.dart';
