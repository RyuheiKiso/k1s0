import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

class MockDio extends Mock implements Dio {}

void main() {
  late MockDio mockDio;

  setUp(() {
    mockDio = MockDio();
  });

  group('NavigationResponse', () {
    test('fromJson でルートとガードが正しくパースされる', () {
      final json = {
        'routes': [
          {
            'id': 'root',
            'path': '/',
            'redirect_to': '/dashboard',
          },
          {
            'id': 'login',
            'path': '/login',
            'component_id': 'LoginPage',
            'guards': <String>[],
          },
          {
            'id': 'dashboard',
            'path': '/dashboard',
            'component_id': 'DashboardPage',
            'guards': ['auth_required'],
          },
        ],
        'guards': [
          {
            'id': 'auth_required',
            'type': 'auth_required',
            'redirect_to': '/login',
          },
        ],
      };

      final nav = NavigationResponse.fromJson(json);

      expect(nav.routes, hasLength(3));
      expect(nav.routes[0].id, equals('root'));
      expect(nav.routes[0].redirectTo, equals('/dashboard'));
      expect(nav.routes[1].componentId, equals('LoginPage'));
      expect(nav.routes[2].guards, contains('auth_required'));
      expect(nav.guards, hasLength(1));
      expect(nav.guards.first.type, equals(GuardType.authRequired));
      expect(nav.guards.first.redirectTo, equals('/login'));
    });

    test('fromYaml でYAML文字列から正しくパースされる', () {
      const yamlStr = '''
version: 1

guards:
  - id: auth_required
    type: auth_required
    redirect_to: /login

routes:
  - id: root
    path: /
    redirect_to: /dashboard

  - id: login
    path: /login
    component_id: LoginPage
''';

      final nav = NavigationResponse.fromYaml(yamlStr);

      expect(nav.routes, hasLength(2));
      expect(nav.routes[0].id, equals('root'));
      expect(nav.routes[0].redirectTo, equals('/dashboard'));
      expect(nav.routes[1].componentId, equals('LoginPage'));
      expect(nav.guards, hasLength(1));
      expect(nav.guards.first.id, equals('auth_required'));
    });

    test('子ルートがネストして正しくパースされる', () {
      final json = {
        'routes': [
          {
            'id': 'settings',
            'path': '/settings',
            'component_id': 'SettingsPage',
            'children': [
              {
                'id': 'profile',
                'path': 'profile',
                'component_id': 'ProfilePage',
              },
            ],
          },
        ],
        'guards': <Map<String, dynamic>>[],
      };

      final nav = NavigationResponse.fromJson(json);

      expect(nav.routes.first.children, hasLength(1));
      expect(nav.routes.first.children.first.id, equals('profile'));
      expect(nav.routes.first.children.first.path, equals('profile'));
    });
  });

  group('NavigationInterpreter', () {
    test('remote mode で API からナビゲーションを取得して GoRouter が構築される',
        () async {
      final navJson = {
        'routes': [
          {
            'id': 'root',
            'path': '/',
            'redirect_to': '/home',
          },
          {
            'id': 'home',
            'path': '/home',
            'component_id': 'HomePage',
          },
        ],
        'guards': <Map<String, dynamic>>[],
      };

      when(() => mockDio.get<Map<String, dynamic>>('/api/v1/navigation'))
          .thenAnswer(
        (_) async => Response(
          data: navJson,
          statusCode: 200,
          requestOptions: RequestOptions(),
        ),
      );

      final interpreter = NavigationInterpreter(
        mode: NavigationMode.remote,
        componentRegistry: {},
        dio: mockDio,
      );

      final router = await interpreter.build();
      expect(router, isNotNull);
      verify(() => mockDio.get<Map<String, dynamic>>('/api/v1/navigation'))
          .called(1);
    });

    test('API 失敗時に例外がスローされる', () async {
      when(() => mockDio.get<Map<String, dynamic>>('/api/v1/navigation'))
          .thenThrow(
        DioException(
          requestOptions: RequestOptions(),
          type: DioExceptionType.badResponse,
          response: Response(
            statusCode: 500,
            requestOptions: RequestOptions(),
          ),
        ),
      );

      final interpreter = NavigationInterpreter(
        mode: NavigationMode.remote,
        componentRegistry: {},
        dio: mockDio,
      );

      expect(
        () => interpreter.build(),
        throwsA(isA<DioException>()),
      );
    });
  });

  group('NavigationGuard', () {
    test('fromJson で roles が正しくパースされる', () {
      final json = {
        'id': 'role_required',
        'type': 'role_required',
        'redirect_to': '/unauthorized',
        'roles': ['admin', 'editor'],
      };

      final guard = NavigationGuard.fromJson(json);

      expect(guard.type, equals(GuardType.roleRequired));
      expect(guard.roles, equals(['admin', 'editor']));
    });

    test('roles が省略された場合は空リストになる', () {
      final json = {
        'id': 'auth_required',
        'type': 'auth_required',
        'redirect_to': '/login',
      };

      final guard = NavigationGuard.fromJson(json);
      expect(guard.roles, isEmpty);
    });
  });

  group('NavigationParam', () {
    test('fromJson で各 ParamType が正しくパースされる', () {
      expect(
        NavigationParam.fromJson({'name': 'id', 'type': 'int'}).type,
        equals(ParamType.int),
      );
      expect(
        NavigationParam.fromJson({'name': 'id', 'type': 'uuid'}).type,
        equals(ParamType.uuid),
      );
      expect(
        NavigationParam.fromJson({'name': 'slug', 'type': 'string'}).type,
        equals(ParamType.string),
      );
      expect(
        NavigationParam.fromJson({'name': 'other', 'type': 'unknown'}).type,
        equals(ParamType.string),
      );
    });
  });
}
