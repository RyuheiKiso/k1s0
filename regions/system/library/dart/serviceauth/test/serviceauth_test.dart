import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import 'package:k1s0_serviceauth/serviceauth.dart';

class MockHttpClient extends Mock implements http.Client {}

http.Response tokenResponse(String token, {int expiresIn = 3600}) =>
    http.Response(
      jsonEncode({
        'access_token': token,
        'token_type': 'Bearer',
        'expires_in': expiresIn,
        'scope': 'openid',
      }),
      200,
    );

void main() {
  late MockHttpClient mockClient;

  setUpAll(() {
    registerFallbackValue(Uri.parse('http://localhost'));
  });

  setUp(() {
    mockClient = MockHttpClient();
  });

  group('parseSpiffeId', () {
    test('parses valid SPIFFE ID', () {
      final spiffe =
          parseSpiffeId('spiffe://k1s0.internal/ns/system/sa/auth-service');
      expect(spiffe.trustDomain, equals('k1s0.internal'));
      expect(spiffe.namespace, equals('system'));
      expect(spiffe.serviceAccount, equals('auth-service'));
    });

    test('throws on invalid scheme', () {
      expect(
        () => parseSpiffeId('http://not-spiffe'),
        throwsA(isA<ServiceAuthError>()),
      );
    });

    test('throws on missing /ns/ path', () {
      expect(
        () => parseSpiffeId('spiffe://domain/invalid/path'),
        throwsA(isA<ServiceAuthError>()),
      );
    });

    test('throws on empty string', () {
      expect(
        () => parseSpiffeId(''),
        throwsA(isA<ServiceAuthError>()),
      );
    });

    test('parses service tier SPIFFE ID', () {
      final spiffe = parseSpiffeId(
          'spiffe://k1s0.internal/ns/service/sa/payment-service');
      expect(spiffe.namespace, equals('service'));
      expect(spiffe.serviceAccount, equals('payment-service'));
      expect(spiffe.trustDomain, equals('k1s0.internal'));
    });

    test('toString returns correct URI', () {
      final spiffe = parseSpiffeId(
          'spiffe://k1s0.internal/ns/business/sa/order-service');
      expect(spiffe.toString(),
          equals('spiffe://k1s0.internal/ns/business/sa/order-service'));
    });
  });

  group('validateSpiffeId', () {
    test('passes with correct namespace', () {
      final spiffe = validateSpiffeId(
          'spiffe://k1s0.internal/ns/system/sa/auth-service', 'system');
      expect(spiffe.namespace, equals('system'));
    });

    test('throws on wrong namespace', () {
      expect(
        () => validateSpiffeId(
            'spiffe://k1s0.internal/ns/system/sa/auth-service', 'business'),
        throwsA(isA<ServiceAuthError>()),
      );
    });
  });

  group('ServiceToken functions', () {
    test('isExpired returns true for expired token', () {
      final token = ServiceToken(
        accessToken: 'tok',
        tokenType: 'Bearer',
        expiresAt: DateTime.now().subtract(const Duration(seconds: 1)),
      );
      expect(isExpired(token), isTrue);
    });

    test('isExpired returns false for valid token', () {
      final token = ServiceToken(
        accessToken: 'tok',
        tokenType: 'Bearer',
        expiresAt: DateTime.now().add(const Duration(hours: 1)),
      );
      expect(isExpired(token), isFalse);
    });

    test('shouldRefresh returns true within 30 seconds', () {
      final token = ServiceToken(
        accessToken: 'tok',
        tokenType: 'Bearer',
        expiresAt: DateTime.now().add(const Duration(seconds: 29)),
      );
      expect(shouldRefresh(token), isTrue);
    });

    test('shouldRefresh returns false with sufficient remaining time', () {
      final token = ServiceToken(
        accessToken: 'tok',
        tokenType: 'Bearer',
        expiresAt: DateTime.now().add(const Duration(hours: 1)),
      );
      expect(shouldRefresh(token), isFalse);
    });

    test('bearerHeader returns correct header string', () {
      final token = ServiceToken(
        accessToken: 'my-token-123',
        tokenType: 'Bearer',
        expiresAt: DateTime.now().add(const Duration(hours: 1)),
      );
      expect(bearerHeader(token), equals('Bearer my-token-123'));
    });
  });

  group('HttpServiceAuthClient.getToken', () {
    test('retrieves access token', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async => tokenResponse('access-token-123'));

      final client = HttpServiceAuthClient(
        const ServiceAuthConfig(
          tokenEndpoint: 'http://localhost/token',
          clientId: 'test-client',
          clientSecret: 'test-secret',
        ),
        httpClient: mockClient,
      );

      final token = await client.getToken();
      expect(token.accessToken, equals('access-token-123'));
      expect(token.tokenType, equals('Bearer'));
      expect(isExpired(token), isFalse);
    });

    test('throws on server error', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async => http.Response('', 401));

      final client = HttpServiceAuthClient(
        const ServiceAuthConfig(
          tokenEndpoint: 'http://localhost/token',
          clientId: 'bad-client',
          clientSecret: 'bad-secret',
        ),
        httpClient: mockClient,
      );

      expect(() => client.getToken(), throwsA(isA<ServiceAuthError>()));
    });
  });

  group('HttpServiceAuthClient.getCachedToken', () {
    test('uses cache on subsequent calls', () async {
      var callCount = 0;
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async {
        callCount++;
        return tokenResponse('cached-token');
      });

      final client = HttpServiceAuthClient(
        const ServiceAuthConfig(
          tokenEndpoint: 'http://localhost/token',
          clientId: 'test-client',
          clientSecret: 'test-secret',
        ),
        httpClient: mockClient,
      );

      // Call 3 times, token endpoint should be called only once
      for (var i = 0; i < 3; i++) {
        final bearer = await client.getCachedToken();
        expect(bearer, equals('Bearer cached-token'));
      }
      expect(callCount, equals(1));
    });

    test('refreshes when shouldRefresh is true', () async {
      var callCount = 0;
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async {
        callCount++;
        // Return token that expires in 10 seconds (within 30s refresh window)
        return tokenResponse('new-token', expiresIn: 10);
      });

      final client = HttpServiceAuthClient(
        const ServiceAuthConfig(
          tokenEndpoint: 'http://localhost/token',
          clientId: 'client',
          clientSecret: 'secret',
        ),
        httpClient: mockClient,
      );

      // First call fetches token
      await client.getCachedToken();
      expect(callCount, equals(1));

      // Second call should refresh because expiresIn=10 is within 30s window
      await client.getCachedToken();
      expect(callCount, equals(2));
    });
  });

  group('HttpServiceAuthClient.validateSpiffeIdCheck', () {
    test('works correctly', () {
      final client = HttpServiceAuthClient(
        const ServiceAuthConfig(
          tokenEndpoint: 'http://localhost/token',
          clientId: 'client',
          clientSecret: 'secret',
        ),
        httpClient: mockClient,
      );

      final spiffe = client.validateSpiffeIdCheck(
          'spiffe://k1s0.internal/ns/system/sa/auth-service', 'system');
      expect(spiffe.namespace, equals('system'));
    });
  });

  group('ServiceAuthError', () {
    test('has correct message', () {
      const err = ServiceAuthError('test error');
      expect(err.message, equals('test error'));
      expect(err.toString(), contains('test error'));
    });

    test('includes cause when present', () {
      const err = ServiceAuthError('test error', cause: 'root cause');
      expect(err.toString(), contains('cause'));
    });
  });
}
