import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_http/src/client/http_client_config.dart';
import 'package:k1s0_http/src/interceptors/logging_interceptor.dart';
import 'package:k1s0_http/src/types/request_options.dart';

void main() {
  group('HttpClientConfig', () {
    test('creates with default values', () {
      const config = HttpClientConfig(baseUrl: 'https://api.example.com');

      expect(config.baseUrl, 'https://api.example.com');
      expect(config.timeout, const Duration(seconds: 30));
      expect(config.connectTimeout, const Duration(seconds: 10));
      expect(config.retryPolicy, RetryPolicy.none);
      expect(config.defaultHeaders, isEmpty);
      expect(config.logLevel, HttpLogLevel.basic);
      expect(config.validateStatus, isNull);
    });

    test('creates with custom values', () {
      bool customValidator(int? status) => status != null && status < 500;

      final config = HttpClientConfig(
        baseUrl: 'https://api.example.com',
        timeout: const Duration(seconds: 60),
        connectTimeout: const Duration(seconds: 20),
        retryPolicy: const RetryPolicy(maxAttempts: 5),
        defaultHeaders: {'X-Custom': 'value'},
        logLevel: HttpLogLevel.full,
        validateStatus: customValidator,
      );

      expect(config.timeout, const Duration(seconds: 60));
      expect(config.connectTimeout, const Duration(seconds: 20));
      expect(config.retryPolicy.maxAttempts, 5);
      expect(config.defaultHeaders['X-Custom'], 'value');
      expect(config.logLevel, HttpLogLevel.full);
      expect(config.validateStatus, isNotNull);
    });

    test('copyWith creates new instance with updated values', () {
      const original = HttpClientConfig(baseUrl: 'https://api.example.com');

      final copied = original.copyWith(
        baseUrl: 'https://new-api.example.com',
        timeout: const Duration(seconds: 45),
      );

      expect(original.baseUrl, 'https://api.example.com');
      expect(original.timeout, const Duration(seconds: 30));
      expect(copied.baseUrl, 'https://new-api.example.com');
      expect(copied.timeout, const Duration(seconds: 45));
      // Unchanged values
      expect(copied.connectTimeout, original.connectTimeout);
      expect(copied.logLevel, original.logLevel);
    });
  });

  group('K1s0RequestOptions', () {
    test('creates with default values', () {
      const options = K1s0RequestOptions();

      expect(options.headers, isNull);
      expect(options.queryParameters, isNull);
      expect(options.timeout, isNull);
      expect(options.skipAuth, false);
      expect(options.retry, false);
      expect(options.retryCount, isNull);
      expect(options.traceId, isNull);
      expect(options.extra, isNull);
    });

    test('creates with custom values', () {
      const options = K1s0RequestOptions(
        headers: {'Authorization': 'Bearer token'},
        queryParameters: {'page': 1},
        timeout: 5000,
        skipAuth: true,
        retry: true,
        retryCount: 3,
        traceId: 'trace-123',
        extra: {'customKey': 'customValue'},
      );

      expect(options.headers, {'Authorization': 'Bearer token'});
      expect(options.queryParameters, {'page': 1});
      expect(options.timeout, 5000);
      expect(options.skipAuth, true);
      expect(options.retry, true);
      expect(options.retryCount, 3);
      expect(options.traceId, 'trace-123');
      expect(options.extra, {'customKey': 'customValue'});
    });

    test('copyWith creates new instance with updated values', () {
      const original = K1s0RequestOptions(
        timeout: 5000,
        retry: false,
      );

      final copied = original.copyWith(
        retry: true,
        traceId: 'new-trace',
      );

      expect(original.retry, false);
      expect(original.traceId, isNull);
      expect(copied.retry, true);
      expect(copied.traceId, 'new-trace');
      // Unchanged values
      expect(copied.timeout, original.timeout);
    });
  });

  group('RetryPolicy', () {
    test('none policy has maxAttempts of 0', () {
      expect(RetryPolicy.none.maxAttempts, 0);
    });

    test('creates with default values', () {
      const policy = RetryPolicy();

      expect(policy.maxAttempts, 3);
      expect(policy.initialDelay, const Duration(milliseconds: 1000));
      expect(policy.maxDelay, const Duration(seconds: 30));
      expect(policy.backoffMultiplier, 2.0);
      expect(policy.retryStatusCodes, [502, 503, 504]);
      expect(policy.retryOnTimeout, true);
      expect(policy.retryOnConnectionError, true);
    });

    test('creates with custom values', () {
      const policy = RetryPolicy(
        maxAttempts: 5,
        initialDelay: Duration(milliseconds: 500),
        maxDelay: Duration(seconds: 60),
        backoffMultiplier: 1.5,
        retryStatusCodes: [429, 500, 502, 503],
        retryOnTimeout: false,
        retryOnConnectionError: false,
      );

      expect(policy.maxAttempts, 5);
      expect(policy.initialDelay, const Duration(milliseconds: 500));
      expect(policy.maxDelay, const Duration(seconds: 60));
      expect(policy.backoffMultiplier, 1.5);
      expect(policy.retryStatusCodes, [429, 500, 502, 503]);
      expect(policy.retryOnTimeout, false);
      expect(policy.retryOnConnectionError, false);
    });

    test('delayForAttempt returns zero for first attempt', () {
      const policy = RetryPolicy();

      expect(policy.delayForAttempt(0), Duration.zero);
    });

    test('delayForAttempt calculates exponential backoff', () {
      const policy = RetryPolicy(
        initialDelay: Duration(milliseconds: 1000),
        backoffMultiplier: 2.0,
      );

      expect(policy.delayForAttempt(1), const Duration(milliseconds: 1000));
      expect(policy.delayForAttempt(2), const Duration(milliseconds: 2000));
      expect(policy.delayForAttempt(3), const Duration(milliseconds: 4000));
      expect(policy.delayForAttempt(4), const Duration(milliseconds: 8000));
    });

    test('delayForAttempt respects maxDelay', () {
      const policy = RetryPolicy(
        initialDelay: Duration(seconds: 10),
        maxDelay: Duration(seconds: 15),
        backoffMultiplier: 2.0,
      );

      expect(policy.delayForAttempt(1), const Duration(seconds: 10));
      expect(policy.delayForAttempt(2), const Duration(seconds: 15));
      expect(policy.delayForAttempt(3), const Duration(seconds: 15));
    });

    test('delayForAttempt with multiplier 1.0 returns constant delay', () {
      const policy = RetryPolicy(
        initialDelay: Duration(milliseconds: 1000),
        backoffMultiplier: 1.0,
      );

      expect(policy.delayForAttempt(1), const Duration(milliseconds: 1000));
      expect(policy.delayForAttempt(2), const Duration(milliseconds: 1000));
      expect(policy.delayForAttempt(3), const Duration(milliseconds: 1000));
    });
  });
}
