import 'package:dio/dio.dart';
import 'package:meta/meta.dart';

/// HTTP response wrapper
@immutable
class K1s0Response<T> {
  /// Creates an HTTP response
  const K1s0Response({
    required this.data,
    required this.statusCode,
    required this.headers,
    this.traceId,
    this.requestId,
  });

  /// Creates a response from a Dio response
  factory K1s0Response.fromDioResponse(Response<T> response) =>
      K1s0Response<T>(
        data: response.data as T,
        statusCode: response.statusCode ?? 0,
        headers: response.headers.map,
        traceId: response.headers.value('x-trace-id'),
        requestId: response.headers.value('x-request-id'),
      );

  /// Response data
  final T data;

  /// HTTP status code
  final int statusCode;

  /// Response headers
  final Map<String, List<String>> headers;

  /// Trace ID from the server
  final String? traceId;

  /// Request ID from the server
  final String? requestId;

  /// Whether the response was successful (2xx status code)
  bool get isSuccess => statusCode >= 200 && statusCode < 300;

  /// Get a single header value
  String? header(String name) {
    final values = headers[name.toLowerCase()];
    return (values?.isNotEmpty ?? false) ? values!.first : null;
  }

  /// Transform the response data
  K1s0Response<R> map<R>(R Function(T data) transform) =>
      K1s0Response<R>(
        data: transform(data),
        statusCode: statusCode,
        headers: headers,
        traceId: traceId,
        requestId: requestId,
      );
}

/// Empty response for requests that don't return data
class EmptyResponse {
  const EmptyResponse._();

  /// Singleton instance
  static const instance = EmptyResponse._();
}
