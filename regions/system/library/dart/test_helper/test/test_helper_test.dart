import 'package:test/test.dart';
import 'package:k1s0_test_helper/test_helper.dart';

void main() {
  group('JwtTestHelperのテスト', () {
    final helper = JwtTestHelper(secret: 'test-secret');

    test('管理者トークンが作成されること', () {
      final token = helper.createAdminToken();
      final parts = token.split('.');
      expect(parts.length, equals(3));
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.sub, equals('admin'));
      expect(claims.roles, equals(['admin']));
    });

    test('ユーザートークンが作成されること', () {
      final token = helper.createUserToken('user-123', ['user']);
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.sub, equals('user-123'));
      expect(claims.roles, equals(['user']));
    });

    test('テナント付きトークンが作成されること', () {
      final token = helper.createToken(TestClaims(
        sub: 'svc',
        roles: ['service'],
        tenantId: 't-1',
      ));
      final claims = helper.decodeClaims(token);
      expect(claims, isNotNull);
      expect(claims!.tenantId, equals('t-1'));
    });

    test('無効なトークンでnullが返されること', () {
      expect(helper.decodeClaims('invalid'), isNull);
    });
  });

  group('MockServerBuilder', () {
    test('通知サーバーモックが構築されること', () {
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

    test('未知のルートでnullが返されること', () {
      final server =
          MockServerBuilder.ratelimitServer().withHealthOk().build();
      expect(server.handle('GET', '/nonexistent'), isNull);
    });

    test('エラーレスポンスが設定できること', () {
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
    test('有効なUUIDが生成されること', () {
      final id = FixtureBuilder.uuid();
      expect(id.length, equals(36));
      expect(id, contains('-'));
    });

    test('メールアドレスが生成されること', () {
      final email = FixtureBuilder.email();
      expect(email, contains('@example.com'));
    });

    test('プレフィックス付きの名前が生成されること', () {
      final name = FixtureBuilder.name();
      expect(name, startsWith('user-'));
    });

    test('指定した範囲内の整数が生成されること', () {
      for (var i = 0; i < 100; i++) {
        final val = FixtureBuilder.intValue(min: 10, max: 20);
        expect(val, greaterThanOrEqualTo(10));
        expect(val, lessThan(20));
      }
    });

    test('最小値と最大値が同じ場合に最小値が返されること', () {
      expect(FixtureBuilder.intValue(min: 5, max: 5), equals(5));
    });

    test('テナントIDが生成されること', () {
      expect(FixtureBuilder.tenantId(), startsWith('tenant-'));
    });

    test('一意な値が生成されること', () {
      final a = FixtureBuilder.uuid();
      final b = FixtureBuilder.uuid();
      expect(a, isNot(equals(b)));
    });
  });

  group('AssertionHelper', () {
    test('JSONの部分一致で検証が通ること', () {
      AssertionHelper.assertJsonContains(
        {'id': '1', 'status': 'ok', 'extra': 'ignored'},
        {'id': '1', 'status': 'ok'},
      );
    });

    test('ネストしたJSONの部分一致で検証が通ること', () {
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

    test('JSONが一致しない場合に検証が失敗すること', () {
      expect(
        () => AssertionHelper.assertJsonContains({'id': '1'}, {'id': '2'}),
        throwsA(isA<AssertionError>()),
      );
    });

    test('発行されたイベントが見つかること', () {
      final events = [
        {'type': 'created', 'id': '1'},
        {'type': 'updated', 'id': '2'},
      ];
      AssertionHelper.assertEventEmitted(events, 'created');
      AssertionHelper.assertEventEmitted(events, 'updated');
    });

    test('イベントが存在しない場合に例外がスローされること', () {
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
