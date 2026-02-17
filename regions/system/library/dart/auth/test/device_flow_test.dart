import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:k1s0_auth/auth.dart';
import 'package:test/test.dart';

const _deviceEndpoint = 'https://auth.example.com/device';
const _tokenEndpoint = 'https://auth.example.com/token';

void main() {
  late List<String> postUrls;
  late List<Map<String, String>> postBodies;

  setUp(() {
    postUrls = [];
    postBodies = [];
  });

  DeviceAuthClient createClient({
    Future<http.Response> Function(Uri, {Map<String, String>? headers, Object? body})?
        httpPost,
  }) {
    return DeviceAuthClient(DeviceAuthClientOptions(
      deviceEndpoint: _deviceEndpoint,
      tokenEndpoint: _tokenEndpoint,
      httpPost: httpPost,
    ));
  }

  group('DeviceAuthClient', () {
    group('requestDeviceCode', () {
      test('should return DeviceCodeResponse on success', () async {
        final client = createClient(
          httpPost: (url, {headers, body}) async {
            postUrls.add(url.toString());
            final bodyMap = body as Map<String, String>;
            postBodies.add(bodyMap);
            return http.Response(
              jsonEncode({
                'device_code': 'device-code-123',
                'user_code': 'ABCD-EFGH',
                'verification_uri': 'https://auth.example.com/device',
                'verification_uri_complete':
                    'https://auth.example.com/device?user_code=ABCD-EFGH',
                'expires_in': 600,
                'interval': 5,
              }),
              200,
            );
          },
        );

        final resp = await client.requestDeviceCode('test-client', 'openid profile');
        expect(resp.deviceCode, equals('device-code-123'));
        expect(resp.userCode, equals('ABCD-EFGH'));
        expect(resp.verificationUri, equals('https://auth.example.com/device'));
        expect(
          resp.verificationUriComplete,
          equals('https://auth.example.com/device?user_code=ABCD-EFGH'),
        );
        expect(resp.expiresIn, equals(600));
        expect(resp.interval, equals(5));
        expect(postBodies.first['client_id'], equals('test-client'));
        expect(postBodies.first['scope'], equals('openid profile'));
      });
    });

    group('pollToken', () {
      test('should poll with authorization_pending then return token',
          () async {
        var callCount = 0;

        final client = createClient(
          httpPost: (url, {headers, body}) async {
            callCount++;
            final bodyMap = body as Map<String, String>;
            expect(
              bodyMap['grant_type'],
              equals('urn:ietf:params:oauth:grant-type:device_code'),
            );
            expect(bodyMap['device_code'], equals('device-code-123'));
            expect(bodyMap['client_id'], equals('test-client'));

            if (callCount <= 2) {
              return http.Response(
                jsonEncode({'error': 'authorization_pending'}),
                400,
              );
            }
            return http.Response(
              jsonEncode({
                'access_token': 'access-token-xyz',
                'refresh_token': 'refresh-token-xyz',
                'token_type': 'Bearer',
                'expires_in': 900,
              }),
              200,
            );
          },
        );

        final result = await client.pollToken(
          'test-client',
          'device-code-123',
          0,
        );
        expect(result.accessToken, equals('access-token-xyz'));
        expect(result.refreshToken, equals('refresh-token-xyz'));
        expect(result.tokenType, equals('Bearer'));
        expect(result.expiresIn, equals(900));
        expect(callCount, greaterThanOrEqualTo(3));
      }, timeout: Timeout(Duration(seconds: 30)));

      test('should increase interval on slow_down', () async {
        var callCount = 0;
        final callTimestamps = <DateTime>[];

        final client = createClient(
          httpPost: (url, {headers, body}) async {
            callCount++;
            callTimestamps.add(DateTime.now());
            if (callCount == 1) {
              return http.Response(
                jsonEncode({'error': 'slow_down'}),
                400,
              );
            }
            return http.Response(
              jsonEncode({
                'access_token': 'access-token',
                'token_type': 'Bearer',
                'expires_in': 900,
              }),
              200,
            );
          },
        );

        final start = DateTime.now();
        final result = await client.pollToken(
          'test-client',
          'device-code-123',
          0,
        );
        final elapsed = DateTime.now().difference(start);

        expect(result.accessToken, equals('access-token'));
        // slow_down を受け取った後、interval が増加（5+5=10 秒以上）
        expect(elapsed.inSeconds, greaterThanOrEqualTo(10));
      }, timeout: Timeout(Duration(seconds: 30)));

      test('should throw DeviceFlowError on expired_token', () async {
        final client = createClient(
          httpPost: (url, {headers, body}) async {
            return http.Response(
              jsonEncode({'error': 'expired_token'}),
              400,
            );
          },
        );

        expect(
          () => client.pollToken('test-client', 'device-code-123', 5),
          throwsA(isA<DeviceFlowError>().having(
            (e) => e.errorCode,
            'errorCode',
            'expired_token',
          )),
        );
      });

      test('should throw DeviceFlowError on access_denied', () async {
        final client = createClient(
          httpPost: (url, {headers, body}) async {
            return http.Response(
              jsonEncode({'error': 'access_denied'}),
              400,
            );
          },
        );

        expect(
          () => client.pollToken('test-client', 'device-code-123', 5),
          throwsA(isA<DeviceFlowError>().having(
            (e) => e.errorCode,
            'errorCode',
            'access_denied',
          )),
        );
      });

      test('should cancel polling when cancelled future completes', () async {
        final client = createClient(
          httpPost: (url, {headers, body}) async {
            return http.Response(
              jsonEncode({'error': 'authorization_pending'}),
              400,
            );
          },
        );

        final completer = Completer<void>();

        // 2 秒後にキャンセル
        Timer(const Duration(seconds: 2), () => completer.complete());

        expect(
          () => client.pollToken(
            'test-client',
            'device-code-123',
            1,
            cancelled: completer.future,
          ),
          throwsA(isA<DeviceFlowError>().having(
            (e) => e.errorCode,
            'errorCode',
            'cancelled',
          )),
        );
      }, timeout: Timeout(Duration(seconds: 15)));
    });

    group('deviceFlow', () {
      test('should execute the full device flow', () async {
        var tokenCallCount = 0;

        final client = createClient(
          httpPost: (url, {headers, body}) async {
            final bodyMap = body as Map<String, String>;

            // device code request (no grant_type)
            if (!bodyMap.containsKey('grant_type')) {
              return http.Response(
                jsonEncode({
                  'device_code': 'device-code-flow',
                  'user_code': 'WXYZ-1234',
                  'verification_uri': 'https://auth.example.com/device',
                  'verification_uri_complete':
                      'https://auth.example.com/device?user_code=WXYZ-1234',
                  'expires_in': 600,
                  'interval': 0,
                }),
                200,
              );
            }

            // token request
            tokenCallCount++;
            if (tokenCallCount <= 1) {
              return http.Response(
                jsonEncode({'error': 'authorization_pending'}),
                400,
              );
            }

            return http.Response(
              jsonEncode({
                'access_token': 'flow-access-token',
                'refresh_token': 'flow-refresh-token',
                'token_type': 'Bearer',
                'expires_in': 900,
              }),
              200,
            );
          },
        );

        String? receivedUserCode;
        String? receivedVerificationUri;

        final result = await client.deviceFlow(
          'test-client',
          'openid',
          (resp) {
            receivedUserCode = resp.userCode;
            receivedVerificationUri = resp.verificationUri;
          },
        );

        expect(result.accessToken, equals('flow-access-token'));
        expect(result.refreshToken, equals('flow-refresh-token'));
        expect(receivedUserCode, equals('WXYZ-1234'));
        expect(
          receivedVerificationUri,
          equals('https://auth.example.com/device'),
        );
      }, timeout: Timeout(Duration(seconds: 30)));
    });

    group('DeviceFlowError', () {
      test('should have correct errorCode and message', () {
        final error = DeviceFlowError('expired_token', 'Device code expired');
        expect(error.errorCode, equals('expired_token'));
        expect(error.description, equals('Device code expired'));
        expect(error.toString(), contains('expired_token'));
        expect(error.toString(), contains('Device code expired'));
      });

      test('should work without description', () {
        final error = DeviceFlowError('access_denied');
        expect(error.errorCode, equals('access_denied'));
        expect(error.description, isNull);
        expect(error.toString(), contains('access_denied'));
      });
    });
  });
}
