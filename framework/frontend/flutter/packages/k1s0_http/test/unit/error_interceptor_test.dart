import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_http/src/error/api_error.dart';
import 'package:k1s0_http/src/interceptors/error_interceptor.dart';
import 'package:mocktail/mocktail.dart';

class MockErrorInterceptorHandler extends Mock
    implements ErrorInterceptorHandler {}

class MockRequestOptions extends Mock implements RequestOptions {}

class MockResponse extends Mock implements Response<dynamic> {}

class FakeApiError extends Fake implements ApiError {}

void main() {
  late ErrorInterceptor interceptor;
  late MockErrorInterceptorHandler mockHandler;
  late MockRequestOptions mockRequestOptions;

  setUpAll(() {
    registerFallbackValue(FakeApiError());
    registerFallbackValue(
      DioException(requestOptions: RequestOptions(path: '')),
    );
  });

  setUp(() {
    mockHandler = MockErrorInterceptorHandler();
    mockRequestOptions = MockRequestOptions();
    when(() => mockRequestOptions.extra).thenReturn({});
  });

  group('ErrorInterceptor', () {
    test('creates without callbacks', () {
      interceptor = ErrorInterceptor();

      expect(interceptor.errorCallback, isNull);
      expect(interceptor.authErrorCallback, isNull);
    });

    test('creates with callbacks', () {
      void onError(ApiError error) {}
      void onAuthError(ApiError error) {}

      interceptor = ErrorInterceptor(
        errorCallback: onError,
        authErrorCallback: onAuthError,
      );

      expect(interceptor.errorCallback, isNotNull);
      expect(interceptor.authErrorCallback, isNotNull);
    });

    test('calls errorCallback when error occurs', () {
      ApiError? capturedError;
      interceptor = ErrorInterceptor(
        errorCallback: (error) => capturedError = error,
      );

      final dioError = DioException(
        type: DioExceptionType.connectionTimeout,
        requestOptions: mockRequestOptions,
      );

      interceptor.onError(dioError, mockHandler);

      expect(capturedError, isNotNull);
      expect(capturedError!.kind, ApiErrorKind.timeout);
      verify(() => mockHandler.next(dioError)).called(1);
    });

    test('calls authErrorCallback for 401 errors', () {
      ApiError? capturedAuthError;
      interceptor = ErrorInterceptor(
        authErrorCallback: (error) => capturedAuthError = error,
      );

      final mockResponse = MockResponse();
      when(() => mockResponse.statusCode).thenReturn(401);
      when(() => mockResponse.data).thenReturn(null);

      final dioError = DioException(
        type: DioExceptionType.badResponse,
        requestOptions: mockRequestOptions,
        response: mockResponse,
      );

      interceptor.onError(dioError, mockHandler);

      expect(capturedAuthError, isNotNull);
      expect(capturedAuthError!.kind, ApiErrorKind.authentication);
      expect(capturedAuthError!.requiresAuthentication, true);
    });

    test('does not call authErrorCallback for non-auth errors', () {
      ApiError? capturedAuthError;
      interceptor = ErrorInterceptor(
        authErrorCallback: (error) => capturedAuthError = error,
      );

      final mockResponse = MockResponse();
      when(() => mockResponse.statusCode).thenReturn(404);
      when(() => mockResponse.data).thenReturn(null);

      final dioError = DioException(
        type: DioExceptionType.badResponse,
        requestOptions: mockRequestOptions,
        response: mockResponse,
      );

      interceptor.onError(dioError, mockHandler);

      expect(capturedAuthError, isNull);
    });

    test('passes error to next handler', () {
      interceptor = ErrorInterceptor();

      final dioError = DioException(
        type: DioExceptionType.connectionTimeout,
        requestOptions: mockRequestOptions,
      );

      interceptor.onError(dioError, mockHandler);

      verify(() => mockHandler.next(dioError)).called(1);
    });

    test('calls both callbacks for auth errors', () {
      ApiError? capturedError;
      ApiError? capturedAuthError;
      interceptor = ErrorInterceptor(
        errorCallback: (error) => capturedError = error,
        authErrorCallback: (error) => capturedAuthError = error,
      );

      final mockResponse = MockResponse();
      when(() => mockResponse.statusCode).thenReturn(401);
      when(() => mockResponse.data).thenReturn(null);

      final dioError = DioException(
        type: DioExceptionType.badResponse,
        requestOptions: mockRequestOptions,
        response: mockResponse,
      );

      interceptor.onError(dioError, mockHandler);

      expect(capturedError, isNotNull);
      expect(capturedAuthError, isNotNull);
      expect(capturedError!.kind, ApiErrorKind.authentication);
      expect(capturedAuthError!.kind, ApiErrorKind.authentication);
    });

    test('extracts trace ID from request options', () {
      ApiError? capturedError;
      interceptor = ErrorInterceptor(
        errorCallback: (error) => capturedError = error,
      );

      when(() => mockRequestOptions.extra)
          .thenReturn({'traceId': 'trace-from-request'});

      final dioError = DioException(
        type: DioExceptionType.connectionTimeout,
        requestOptions: mockRequestOptions,
      );

      interceptor.onError(dioError, mockHandler);

      expect(capturedError, isNotNull);
      expect(capturedError!.traceId, 'trace-from-request');
    });
  });
}
