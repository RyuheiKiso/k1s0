import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_http/src/error/api_error.dart';
import 'package:k1s0_http/src/error/problem_details.dart';
import 'package:mocktail/mocktail.dart';

class MockDioException extends Mock implements DioException {}

class MockRequestOptions extends Mock implements RequestOptions {}

class MockResponse extends Mock implements Response<dynamic> {}

void main() {
  late MockRequestOptions mockRequestOptions;

  setUp(() {
    mockRequestOptions = MockRequestOptions();
    when(() => mockRequestOptions.extra).thenReturn({});
  });

  group('ApiErrorKind', () {
    test('contains all expected error kinds', () {
      expect(ApiErrorKind.values, contains(ApiErrorKind.validation));
      expect(ApiErrorKind.values, contains(ApiErrorKind.authentication));
      expect(ApiErrorKind.values, contains(ApiErrorKind.authorization));
      expect(ApiErrorKind.values, contains(ApiErrorKind.notFound));
      expect(ApiErrorKind.values, contains(ApiErrorKind.conflict));
      expect(ApiErrorKind.values, contains(ApiErrorKind.rateLimit));
      expect(ApiErrorKind.values, contains(ApiErrorKind.dependency));
      expect(ApiErrorKind.values, contains(ApiErrorKind.temporary));
      expect(ApiErrorKind.values, contains(ApiErrorKind.timeout));
      expect(ApiErrorKind.values, contains(ApiErrorKind.network));
      expect(ApiErrorKind.values, contains(ApiErrorKind.connection));
      expect(ApiErrorKind.values, contains(ApiErrorKind.cancelled));
      expect(ApiErrorKind.values, contains(ApiErrorKind.unknown));
    });
  });

  group('ApiError', () {
    test('creates with required fields', () {
      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
      );

      expect(error.kind, ApiErrorKind.validation);
      expect(error.message, 'Validation failed');
      expect(error.statusCode, isNull);
      expect(error.errorCode, isNull);
      expect(error.traceId, isNull);
      expect(error.problemDetails, isNull);
      expect(error.cause, isNull);
    });

    test('creates with all fields', () {
      const problemDetails = ProblemDetails(
        title: 'Validation Error',
        status: 400,
        errorCode: 'VALIDATION_ERROR',
      );
      final cause = Exception('Original error');
      final stackTrace = StackTrace.current;

      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
        statusCode: 400,
        errorCode: 'VALIDATION_ERROR',
        traceId: 'trace-123',
        problemDetails: problemDetails,
        cause: cause,
        stackTrace: stackTrace,
      );

      expect(error.statusCode, 400);
      expect(error.errorCode, 'VALIDATION_ERROR');
      expect(error.traceId, 'trace-123');
      expect(error.problemDetails, problemDetails);
      expect(error.cause, cause);
      expect(error.stackTrace, stackTrace);
    });

    test('requiresAuthentication returns true for authentication kind', () {
      final error = ApiError(
        kind: ApiErrorKind.authentication,
        message: 'Unauthorized',
      );

      expect(error.requiresAuthentication, true);
    });

    test('requiresAuthentication returns false for other kinds', () {
      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
      );

      expect(error.requiresAuthentication, false);
    });

    test('isRetryable returns true for retryable kinds', () {
      final retryableKinds = [
        ApiErrorKind.temporary,
        ApiErrorKind.dependency,
        ApiErrorKind.timeout,
        ApiErrorKind.network,
        ApiErrorKind.connection,
      ];

      for (final kind in retryableKinds) {
        final error = ApiError(kind: kind, message: 'Error');
        expect(error.isRetryable, true, reason: 'Expected $kind to be retryable');
      }
    });

    test('isRetryable returns false for non-retryable kinds', () {
      final nonRetryableKinds = [
        ApiErrorKind.validation,
        ApiErrorKind.authentication,
        ApiErrorKind.authorization,
        ApiErrorKind.notFound,
        ApiErrorKind.conflict,
        ApiErrorKind.rateLimit,
        ApiErrorKind.cancelled,
        ApiErrorKind.unknown,
      ];

      for (final kind in nonRetryableKinds) {
        final error = ApiError(kind: kind, message: 'Error');
        expect(error.isRetryable, false,
            reason: 'Expected $kind to not be retryable');
      }
    });

    test('fieldErrors returns errors from problemDetails', () {
      const problemDetails = ProblemDetails(
        title: 'Validation Error',
        status: 400,
        errorCode: 'VALIDATION_ERROR',
        errors: [
          FieldError(field: 'email', message: 'Invalid'),
        ],
      );

      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
        problemDetails: problemDetails,
      );

      expect(error.fieldErrors.length, 1);
      expect(error.fieldErrors[0].field, 'email');
    });

    test('fieldErrors returns empty list when problemDetails is null', () {
      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
      );

      expect(error.fieldErrors, isEmpty);
    });

    test('hasFieldErrors returns true when there are field errors', () {
      const problemDetails = ProblemDetails(
        title: 'Validation Error',
        status: 400,
        errorCode: 'VALIDATION_ERROR',
        errors: [
          FieldError(field: 'email', message: 'Invalid'),
        ],
      );

      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
        problemDetails: problemDetails,
      );

      expect(error.hasFieldErrors, true);
    });

    test('toString includes all relevant information', () {
      final error = ApiError(
        kind: ApiErrorKind.validation,
        message: 'Validation failed',
        statusCode: 400,
        errorCode: 'VALIDATION_ERROR',
        traceId: 'trace-123',
      );

      final str = error.toString();

      expect(str, contains('Validation failed'));
      expect(str, contains('VALIDATION_ERROR'));
      expect(str, contains('400'));
      expect(str, contains('trace-123'));
    });
  });

  group('ApiError.fromDioError', () {
    test('creates timeout error for connection timeout', () {
      final dioError = DioException(
        type: DioExceptionType.connectionTimeout,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.timeout);
      expect(error.message, 'Request timed out');
    });

    test('creates timeout error for send timeout', () {
      final dioError = DioException(
        type: DioExceptionType.sendTimeout,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.timeout);
    });

    test('creates timeout error for receive timeout', () {
      final dioError = DioException(
        type: DioExceptionType.receiveTimeout,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.timeout);
    });

    test('creates connection error for connection error', () {
      final dioError = DioException(
        type: DioExceptionType.connectionError,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.connection);
      expect(error.message, 'Connection failed');
    });

    test('creates cancelled error for cancel', () {
      final dioError = DioException(
        type: DioExceptionType.cancel,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.cancelled);
      expect(error.message, 'Request cancelled');
    });

    test('creates network error for bad certificate', () {
      final dioError = DioException(
        type: DioExceptionType.badCertificate,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError);

      expect(error.kind, ApiErrorKind.network);
      expect(error.message, 'Certificate verification failed');
    });

    test('preserves trace ID from request options', () {
      when(() => mockRequestOptions.extra)
          .thenReturn({'traceId': 'request-trace-123'});

      final dioError = DioException(
        type: DioExceptionType.connectionTimeout,
        requestOptions: mockRequestOptions,
      );

      final error = ApiError.fromDioError(dioError, 'trace-456');

      expect(error.traceId, 'trace-456');
    });
  });

  group('ApiError.fromResponse', () {
    late MockResponse mockResponse;

    setUp(() {
      mockResponse = MockResponse();
    });

    test('maps 400 to validation error', () {
      when(() => mockResponse.statusCode).thenReturn(400);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.validation);
    });

    test('maps 401 to authentication error', () {
      when(() => mockResponse.statusCode).thenReturn(401);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.authentication);
    });

    test('maps 403 to authorization error', () {
      when(() => mockResponse.statusCode).thenReturn(403);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.authorization);
    });

    test('maps 404 to not found error', () {
      when(() => mockResponse.statusCode).thenReturn(404);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.notFound);
    });

    test('maps 409 to conflict error', () {
      when(() => mockResponse.statusCode).thenReturn(409);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.conflict);
    });

    test('maps 429 to rate limit error', () {
      when(() => mockResponse.statusCode).thenReturn(429);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.rateLimit);
    });

    test('maps 502 to dependency error', () {
      when(() => mockResponse.statusCode).thenReturn(502);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.dependency);
    });

    test('maps 503 to dependency error', () {
      when(() => mockResponse.statusCode).thenReturn(503);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.dependency);
    });

    test('maps 504 to timeout error', () {
      when(() => mockResponse.statusCode).thenReturn(504);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.timeout);
    });

    test('maps other 5xx to temporary error', () {
      when(() => mockResponse.statusCode).thenReturn(500);
      when(() => mockResponse.data).thenReturn(null);

      final error = ApiError.fromResponse(mockResponse);

      expect(error.kind, ApiErrorKind.temporary);
    });

    test('parses ProblemDetails from response data', () {
      when(() => mockResponse.statusCode).thenReturn(400);
      when(() => mockResponse.data).thenReturn({
        'title': 'Validation Error',
        'status': 400,
        'error_code': 'VALIDATION_ERROR',
        'detail': 'The email field is invalid',
        'trace_id': 'trace-123',
      });

      final error = ApiError.fromResponse(mockResponse);

      expect(error.message, 'The email field is invalid');
      expect(error.errorCode, 'VALIDATION_ERROR');
      expect(error.traceId, 'trace-123');
      expect(error.problemDetails, isNotNull);
    });

    test('handles string response data', () {
      when(() => mockResponse.statusCode).thenReturn(500);
      when(() => mockResponse.data).thenReturn('Internal Server Error');

      final error = ApiError.fromResponse(mockResponse);

      expect(error.message, 'Internal Server Error');
    });

    test('uses default message when data is empty', () {
      when(() => mockResponse.statusCode).thenReturn(404);
      when(() => mockResponse.data).thenReturn('');

      final error = ApiError.fromResponse(mockResponse);

      expect(error.message, 'Resource not found');
    });
  });

  group('ApiErrorExtension', () {
    test('localizedMessage returns Japanese messages', () {
      expect(
        ApiError(kind: ApiErrorKind.validation, message: '')
            .localizedMessage
            .isNotEmpty,
        true,
      );
      expect(
        ApiError(kind: ApiErrorKind.authentication, message: '')
            .localizedMessage,
        contains('認証'),
      );
      expect(
        ApiError(kind: ApiErrorKind.timeout, message: '').localizedMessage,
        contains('タイムアウト'),
      );
      expect(
        ApiError(kind: ApiErrorKind.network, message: '').localizedMessage,
        contains('ネットワーク'),
      );
    });
  });
}
