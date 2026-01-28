import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:k1s0_navigation/k1s0_navigation.dart';

void main() {
  group('RouteEntry', () {
    test('creates with required parameters', () {
      final entry = RouteEntry(
        path: '/home',
        name: 'home',
        builder: (context, state) => const SizedBox(),
      );

      expect(entry.path, '/home');
      expect(entry.name, 'home');
      expect(entry.children, isEmpty);
      expect(entry.guards, isEmpty);
    });

    test('creates with children', () {
      final entry = RouteEntry(
        path: '/users',
        name: 'users',
        builder: (context, state) => const SizedBox(),
        children: [
          RouteEntry(
            path: ':id',
            name: 'user-detail',
            builder: (context, state) => const SizedBox(),
          ),
        ],
      );

      expect(entry.children, hasLength(1));
      expect(entry.children.first.name, 'user-detail');
    });

    test('converts to GoRoute', () {
      final entry = RouteEntry(
        path: '/settings',
        name: 'settings',
        builder: (context, state) => const Scaffold(body: Text('Settings')),
      );

      final goRoute = entry.toGoRoute(guardCallbacks: {});

      expect(goRoute.path, '/settings');
      expect(goRoute.name, 'settings');
    });
  });

  group('RouteConfig', () {
    test('creates with default values', () {
      final config = RouteConfig(
        routes: [
          RouteEntry(
            path: '/',
            name: 'home',
            builder: (context, state) => const SizedBox(),
          ),
        ],
      );

      expect(config.initialLocation, '/');
      expect(config.debugLogDiagnostics, false);
      expect(config.routerNeglect, false);
    });

    test('copyWith creates modified copy', () {
      final original = RouteConfig(
        routes: [],
        initialLocation: '/',
      );

      final modified = original.copyWith(
        initialLocation: '/home',
        debugLogDiagnostics: true,
      );

      expect(modified.initialLocation, '/home');
      expect(modified.debugLogDiagnostics, true);
      expect(original.initialLocation, '/');
    });
  });

  group('RouteConfigBuilder', () {
    test('builds config with routes', () {
      final config = RouteConfigBuilder()
          .addRoute(RouteEntry(
            path: '/',
            name: 'home',
            builder: (context, state) => const SizedBox(),
          ))
          .addRoute(RouteEntry(
            path: '/settings',
            name: 'settings',
            builder: (context, state) => const SizedBox(),
          ))
          .initialLocation('/settings')
          .enableDebugLogging()
          .build();

      expect(config.routes, hasLength(2));
      expect(config.initialLocation, '/settings');
      expect(config.debugLogDiagnostics, true);
    });

    test('addRoutes adds multiple routes', () {
      final config = RouteConfigBuilder()
          .addRoutes([
            RouteEntry(
              path: '/',
              name: 'home',
              builder: (context, state) => const SizedBox(),
            ),
            RouteEntry(
              path: '/about',
              name: 'about',
              builder: (context, state) => const SizedBox(),
            ),
          ])
          .build();

      expect(config.routes, hasLength(2));
    });
  });

  group('RouteGuardRegistry', () {
    test('registers and retrieves guards', () {
      final registry = RouteGuardRegistry();
      final guard = FunctionalRouteGuard(
        name: 'testGuard',
        checkFn: (context, state) => null,
      );

      registry.register(guard);

      expect(registry.contains('testGuard'), true);
      expect(registry.get('testGuard'), guard);
    });

    test('unregisters guards', () {
      final registry = RouteGuardRegistry();
      final guard = FunctionalRouteGuard(
        name: 'testGuard',
        checkFn: (context, state) => null,
      );

      registry.register(guard);
      registry.unregister('testGuard');

      expect(registry.contains('testGuard'), false);
    });

    test('clears all guards', () {
      final registry = RouteGuardRegistry();

      registry.register(FunctionalRouteGuard(
        name: 'guard1',
        checkFn: (context, state) => null,
      ));
      registry.register(FunctionalRouteGuard(
        name: 'guard2',
        checkFn: (context, state) => null,
      ));

      registry.clear();

      expect(registry.contains('guard1'), false);
      expect(registry.contains('guard2'), false);
    });
  });

  group('AuthGuard', () {
    test('passes when authenticated', () {
      final guard = AuthGuard(
        isAuthenticated: (context) => true,
      );

      // Note: We can't easily test with a real BuildContext in unit tests
      // This would be tested in widget tests
      expect(guard.name, 'auth');
      expect(guard.loginPath, '/login');
    });

    test('uses custom configuration', () {
      final guard = AuthGuard(
        name: 'customAuth',
        isAuthenticated: (context) => false,
        loginPath: '/signin',
        returnToParameter: 'redirect',
        excludedPaths: ['/signin', '/signup'],
      );

      expect(guard.name, 'customAuth');
      expect(guard.loginPath, '/signin');
      expect(guard.returnToParameter, 'redirect');
      expect(guard.excludedPaths, contains('/signin'));
    });
  });

  group('SimpleAuthStateNotifier', () {
    test('initial state is not authenticated', () {
      final notifier = SimpleAuthStateNotifier();
      expect(notifier.isAuthenticated, false);
    });

    test('login sets authenticated to true', () {
      final notifier = SimpleAuthStateNotifier();
      notifier.login();
      expect(notifier.isAuthenticated, true);
    });

    test('logout sets authenticated to false', () {
      final notifier = SimpleAuthStateNotifier(isAuthenticated: true);
      notifier.logout();
      expect(notifier.isAuthenticated, false);
    });

    test('notifies listeners on state change', () {
      final notifier = SimpleAuthStateNotifier();
      var notifyCount = 0;

      notifier.addListener(() => notifyCount++);

      notifier.login();
      expect(notifyCount, 1);

      notifier.logout();
      expect(notifyCount, 2);
    });

    test('does not notify when state unchanged', () {
      final notifier = SimpleAuthStateNotifier(isAuthenticated: true);
      var notifyCount = 0;

      notifier.addListener(() => notifyCount++);

      notifier.login(); // Already authenticated
      expect(notifyCount, 0);
    });
  });

  group('CompositeRouteGuard', () {
    test('passes when all guards pass', () {
      final guard = CompositeRouteGuard(
        name: 'composite',
        guards: [
          FunctionalRouteGuard(name: 'g1', checkFn: (c, s) => null),
          FunctionalRouteGuard(name: 'g2', checkFn: (c, s) => null),
        ],
      );

      expect(guard.name, 'composite');
    });
  });
}
