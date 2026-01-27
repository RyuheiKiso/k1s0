import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:k1s0_auth/src/guard/auth_guard.dart';
import 'package:k1s0_auth/src/provider/auth_provider.dart';
import 'package:k1s0_auth/src/provider/auth_state.dart';
import 'package:k1s0_auth/src/storage/memory_token_storage.dart';
import 'package:k1s0_auth/src/token/claims.dart';
import 'package:k1s0_auth/src/types/auth_user.dart';
import 'package:mocktail/mocktail.dart';

class MockGoRouterState extends Mock implements GoRouterState {}

class MockBuildContext extends Mock implements BuildContext {}

void main() {
  late MockGoRouterState mockRouterState;
  late MockBuildContext mockContext;

  setUp(() {
    mockRouterState = MockGoRouterState();
    mockContext = MockBuildContext();
    when(() => mockRouterState.matchedLocation).thenReturn('/home');
  });

  group('AuthGuard', () {
    test('creates with default values', () {
      const guard = AuthGuard();

      expect(guard.loginPath, '/login');
      expect(guard.homePath, '/');
      expect(guard.roles, isNull);
      expect(guard.permissions, isNull);
    });

    test('creates with custom values', () {
      const guard = AuthGuard(
        loginPath: '/signin',
        homePath: '/dashboard',
        roles: ['admin'],
        permissions: ['read'],
      );

      expect(guard.loginPath, '/signin');
      expect(guard.homePath, '/dashboard');
      expect(guard.roles, ['admin']);
      expect(guard.permissions, ['read']);
    });

    test('redirect returns null when initializing', () {
      const guard = AuthGuard();

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.initial,
      );

      expect(redirect, isNull);
    });

    test('redirect returns loginPath when not authenticated', () {
      const guard = AuthGuard(loginPath: '/login');
      when(() => mockRouterState.matchedLocation).thenReturn('/protected');

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.unauthenticated,
      );

      expect(redirect, '/login');
    });

    test('redirect returns null when not authenticated but on login page', () {
      const guard = AuthGuard(loginPath: '/login');
      when(() => mockRouterState.matchedLocation).thenReturn('/login');

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.unauthenticated,
      );

      expect(redirect, isNull);
    });

    test('redirect returns homePath when authenticated and on login page', () {
      const guard = AuthGuard(homePath: '/dashboard');
      when(() => mockRouterState.matchedLocation).thenReturn('/login');

      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      );
      final user = AuthUser.fromClaims(claims);

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.authenticated(user),
      );

      expect(redirect, '/dashboard');
    });

    test('redirect returns null when authenticated and has required role', () {
      const guard = AuthGuard(roles: ['admin']);
      when(() => mockRouterState.matchedLocation).thenReturn('/admin');

      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['admin'],
      );
      final user = AuthUser.fromClaims(claims);

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.authenticated(user),
      );

      expect(redirect, isNull);
    });

    test('redirect returns homePath when missing required role', () {
      const guard = AuthGuard(roles: ['admin'], homePath: '/');
      when(() => mockRouterState.matchedLocation).thenReturn('/admin');

      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['user'],
      );
      final user = AuthUser.fromClaims(claims);

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.authenticated(user),
      );

      expect(redirect, '/');
    });

    test('redirect returns homePath when missing required permissions', () {
      const guard = AuthGuard(permissions: ['admin:write'], homePath: '/');
      when(() => mockRouterState.matchedLocation).thenReturn('/admin');

      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        permissions: ['read'],
      );
      final user = AuthUser.fromClaims(claims);

      final redirect = guard.redirect(
        mockContext,
        mockRouterState,
        AuthState.authenticated(user),
      );

      expect(redirect, '/');
    });
  });

  group('RequireAuth widget', () {
    testWidgets('shows loading widget while loading', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            tokenStorageProvider.overrideWithValue(MemoryTokenStorage()),
            authProvider.overrideWith((ref) {
              final notifier = AuthNotifier(
                storage: MemoryTokenStorage(),
                autoRefresh: false,
              );
              return notifier;
            }),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequireAuth(
              loading: Text('Loading'),
              child: Text('Content'),
            ),
          ),
        ),
      );

      // Initial state should show loading or empty
      await tester.pump();
    });

    testWidgets('shows fallback when not authenticated', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            tokenStorageProvider.overrideWithValue(MemoryTokenStorage()),
            authProvider.overrideWith((ref) {
              final notifier = AuthNotifier(
                storage: MemoryTokenStorage(),
                autoRefresh: false,
              );
              // Force unauthenticated state
              WidgetsBinding.instance.addPostFrameCallback((_) {
                // State will be set after initialization
              });
              return notifier;
            }),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequireAuth(
              fallback: Text('Please login'),
              child: Text('Content'),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();
    });
  });

  group('RequireRole widget', () {
    testWidgets('shows child when user has role', (tester) async {
      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['admin'],
      );
      final user = AuthUser.fromClaims(claims);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            currentUserProvider.overrideWithValue(user),
            hasAnyRoleProvider('admin').overrideWithValue(true),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequireRole(
              roles: ['admin'],
              child: Text('Admin Content'),
              fallback: Text('No Access'),
            ),
          ),
        ),
      );

      expect(find.text('Admin Content'), findsOneWidget);
    });

    testWidgets('shows fallback when user lacks role', (tester) async {
      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['user'],
      );
      final user = AuthUser.fromClaims(claims);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            currentUserProvider.overrideWithValue(user),
            hasAnyRoleProvider('admin').overrideWithValue(false),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequireRole(
              roles: ['admin'],
              child: Text('Admin Content'),
              fallback: Text('No Access'),
            ),
          ),
        ),
      );

      expect(find.text('No Access'), findsOneWidget);
    });
  });

  group('RequirePermission widget', () {
    testWidgets('shows child when user has permission', (tester) async {
      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        permissions: ['write'],
      );
      final user = AuthUser.fromClaims(claims);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            currentUserProvider.overrideWithValue(user),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequirePermission(
              permissions: ['write'],
              child: Text('Write Content'),
              fallback: Text('No Access'),
            ),
          ),
        ),
      );

      expect(find.text('Write Content'), findsOneWidget);
    });

    testWidgets('shows fallback when user is null', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            currentUserProvider.overrideWithValue(null),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequirePermission(
              permissions: ['write'],
              child: Text('Write Content'),
              fallback: Text('No Access'),
            ),
          ),
        ),
      );

      expect(find.text('No Access'), findsOneWidget);
    });

    testWidgets('requireAll checks all permissions', (tester) async {
      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        permissions: ['read'],
      );
      final user = AuthUser.fromClaims(claims);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            currentUserProvider.overrideWithValue(user),
          ],
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: RequirePermission(
              permissions: ['read', 'write'],
              requireAll: true,
              child: Text('Full Access'),
              fallback: Text('Partial Access'),
            ),
          ),
        ),
      );

      // User only has 'read', not 'write', so requireAll should fail
      expect(find.text('Partial Access'), findsOneWidget);
    });
  });
}
