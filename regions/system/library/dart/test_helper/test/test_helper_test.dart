import 'package:test/test.dart';
import 'package:k1s0_test_helper/test_helper.dart';

void main() {
  group('JwtTestHelper', () {
    final helper = JwtTestHelper(secret: 'test-secret');

    test('creates admin token', () {
      final token = helper.createAdminToken();
      final parts = token.split('.');
      expect(parts.length, equals(3));
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.sub, equals('admin'));
      expect(claims.roles, equals(['admin']));
    });

    test('creates user token', () {
      final token = helper.createUserToken('user-123', ['user']);
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.sub, equals('user-123'));
      expect(claims.roles, equals(['user']));
    });

    test('creates token with tenant', () {
      final token = helper.createToken(TestClaims(
        sub: 'svc',
        roles: ['service'],
        tenantId: 't-1',
      ));
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.tenantId, equals('t-1'));
    });

    test('returns null for invalid token', () {
      expect(helper.decodeClaims('invalid'), isNull);
    });
  });

  group('MockServerBuilder', () {
    test('builds notification server mock', () {
      final server = MockServerBuilder.notificationServer()
          .withHealthOk()
          .withSuccessResponse('/send', '{"id":"1","status":"sent"}')
          .build();

      final health = server.handle('GET', '/health');
      expect(health, isNotNull);
      expect(health!.status, equals(200));
      expect(health.body, contains('ok'));

      final send = server.handle('POST', '/send');
      expect(send, isNotNull);
      expect(send!.status, equals(200));

      expect(server.requestCount, equals(2));
    });

    test('returns null for unknown route', () {
      final server =
          MockServerBuilder.ratelimitServer().withHealthOk().build();
      expect(server.handle('GET', '/nonexistent'), isNull);
    });

    test('supports error responses', () {
      final server = MockServerBuilder.tenantServer()
          .withErrorResponse('/create', 500)
          .build();
      final res = server.handle('POST', '/create');
      expect(res, isNotNull);
      expect(res!.status, equals(500));
      expect(res.body, contains('error'));
    });
  });

  group('FixtureBuilder', () {
    test('generates valid UUID', () {
      final id = FixtureBuilder.uuid();
      expect(id.length, equals(36));
      expect(id, contains('-'));
    });

    test('generates email', () {
      final email = FixtureBuilder.email();
      expect(email, contains('@example.com'));
    });

    test('generates name with prefix', () {
      final name = FixtureBuilder.name();
      expect(name, startsWith('user-'));
    });

    test('generates int in range', () {
      for (var i = 0; i < 100; i++) {
        final val = FixtureBuilder.intValue(min: 10, max: 20);
        expect(val, greaterThanOrEqualTo(10));
        expect(val, lessThan(20));
      }
    });

    test('returns min when min equals max', () {
      expect(FixtureBuilder.intValue(min: 5, max: 5), equals(5));
    });

    test('generates tenant id', () {
      expect(FixtureBuilder.tenantId(), startsWith('tenant-'));
    });

    test('generates unique values', () {
      final a = FixtureBuilder.uuid();
      final b = FixtureBuilder.uuid();
      expect(a, isNot(equals(b)));
    });
  });

  group('AssertionHelper', () {
    test('passes on JSON partial match', () {
      AssertionHelper.assertJsonContains(
        {'id': '1', 'status': 'ok', 'extra': 'ignored'},
        {'id': '1', 'status': 'ok'},
      );
    });

    test('passes on nested JSON partial match', () {
      AssertionHelper.assertJsonContains(
        {
          'user': {'id': '1', 'name': 'test'},
          'status': 'ok'
        },
        {
          'user': {'id': '1'}
        },
      );
    });

    test('fails on JSON mismatch', () {
      expect(
        () => AssertionHelper.assertJsonContains({'id': '1'}, {'id': '2'}),
        throwsA(isA<AssertionError>()),
      );
    });

    test('finds emitted event', () {
      final events = [
        {'type': 'created', 'id': '1'},
        {'type': 'updated', 'id': '2'},
      ];
      AssertionHelper.assertEventEmitted(events, 'created');
      AssertionHelper.assertEventEmitted(events, 'updated');
    });

    test('throws for missing event', () {
      expect(
        () => AssertionHelper.assertEventEmitted(
          [{'type': 'created'}],
          'deleted',
        ),
        throwsA(isA<AssertionError>()),
      );
    });
  });
}
