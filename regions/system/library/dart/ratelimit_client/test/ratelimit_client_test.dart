import 'package:test/test.dart';
import 'package:k1s0_ratelimit_client/ratelimit_client.dart';

void main() {
  late InMemoryRateLimitClient client;

  setUp(() {
    client = InMemoryRateLimitClient();
  });

  group('check', () {
    test('returns allowed when under limit', () async {
      final status = await client.check('test-key', 1);
      expect(status.allowed, isTrue);
      expect(status.remaining, equals(99));
      expect(status.retryAfterSecs, isNull);
    });

    test('returns denied when over limit', () async {
      client.setPolicy('limited', const RateLimitPolicy(
        key: 'limited',
        limit: 2,
        windowSecs: 60,
        algorithm: 'fixed_window',
      ));

      await client.consume('limited', 2);
      final status = await client.check('limited', 1);
      expect(status.allowed, isFalse);
      expect(status.remaining, equals(0));
      expect(status.retryAfterSecs, equals(60));
    });
  });

  group('consume', () {
    test('consumes and returns remaining', () async {
      final result = await client.consume('test-key', 1);
      expect(result.remaining, equals(99));
      expect(client.getUsedCount('test-key'), equals(1));
    });

    test('throws when exceeding limit', () async {
      client.setPolicy('small', const RateLimitPolicy(
        key: 'small',
        limit: 1,
        windowSecs: 60,
        algorithm: 'token_bucket',
      ));

      await client.consume('small', 1);
      expect(
        () => client.consume('small', 1),
        throwsA(isA<RateLimitError>()),
      );
    });
  });

  group('getLimit', () {
    test('returns default policy', () async {
      final policy = await client.getLimit('unknown');
      expect(policy.limit, equals(100));
      expect(policy.windowSecs, equals(3600));
      expect(policy.algorithm, equals('token_bucket'));
    });

    test('returns custom policy', () async {
      client.setPolicy('tenant:T1', const RateLimitPolicy(
        key: 'tenant:T1',
        limit: 50,
        windowSecs: 1800,
        algorithm: 'sliding_window',
      ));

      final policy = await client.getLimit('tenant:T1');
      expect(policy.key, equals('tenant:T1'));
      expect(policy.limit, equals(50));
      expect(policy.algorithm, equals('sliding_window'));
    });
  });

  group('RateLimitError', () {
    test('contains code and message', () {
      const error = RateLimitError('exceeded', code: 'LIMIT_EXCEEDED', retryAfterSecs: 30);
      expect(error.code, equals('LIMIT_EXCEEDED'));
      expect(error.retryAfterSecs, equals(30));
      expect(error.toString(), contains('LIMIT_EXCEEDED'));
    });
  });

  group('RateLimitStatus', () {
    test('stores all fields', () {
      final status = RateLimitStatus(
        allowed: true,
        remaining: 50,
        resetAt: DateTime.now(),
      );
      expect(status.allowed, isTrue);
      expect(status.remaining, equals(50));
      expect(status.retryAfterSecs, isNull);
    });
  });
}
