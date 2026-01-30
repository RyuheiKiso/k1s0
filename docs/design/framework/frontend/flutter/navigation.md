# k1s0_navigation (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

go_router ベースの設定駆動型ナビゲーションを提供する。ルート設定、ルートガード、認証連携、Shell Routes を統合。

## 主要な型

### RouteConfig / RouteEntry

```dart
/// ルート設定
class RouteConfig {
  final List<RouteEntry> routes;
  final String initialLocation;
  final bool debugLogDiagnostics;
  final bool routerNeglect;

  RouteConfig copyWith({...});
}

/// ルートエントリ
class RouteEntry {
  final String path;
  final String name;
  final Widget Function(BuildContext, GoRouterState) builder;
  final List<RouteEntry> children;
  final List<String> guards;

  /// GoRoute に変換
  GoRoute toGoRoute({required Map<String, RouteGuardCallback> guardCallbacks});
}

/// ルート設定ビルダー
class RouteConfigBuilder {
  RouteConfigBuilder addRoute(RouteEntry route);
  RouteConfigBuilder addRoutes(List<RouteEntry> routes);
  RouteConfigBuilder initialLocation(String location);
  RouteConfigBuilder enableDebugLogging();
  RouteConfig build();
}
```

### Route Guards

```dart
/// ルートガード基底クラス
abstract class RouteGuard {
  final String name;

  /// ガードチェック（null を返すと通過、String を返すとリダイレクト）
  String? check(BuildContext context, GoRouterState state);
}

/// 関数ベースのルートガード
class FunctionalRouteGuard extends RouteGuard {
  FunctionalRouteGuard({
    required String name,
    required String? Function(BuildContext, GoRouterState) checkFn,
  });
}

/// 認証ガード
class AuthGuard extends RouteGuard {
  final bool Function(BuildContext) isAuthenticated;
  final String loginPath;
  final String? returnToParameter;
  final List<String> excludedPaths;

  AuthGuard({
    String name = 'auth',
    required this.isAuthenticated,
    this.loginPath = '/login',
    this.returnToParameter = 'returnTo',
    this.excludedPaths = const [],
  });
}

/// ロールベースガード
class RoleGuard extends RouteGuard {
  final List<String> Function(BuildContext) getRoles;
  final List<String> requiredRoles;
  final bool requireAll;
  final String fallbackPath;

  RoleGuard({
    String name = 'role',
    required this.getRoles,
    required this.requiredRoles,
    this.requireAll = false,
    this.fallbackPath = '/forbidden',
  });
}

/// 複合ガード（複数ガードを順番にチェック）
class CompositeRouteGuard extends RouteGuard {
  final List<RouteGuard> guards;

  CompositeRouteGuard({
    required String name,
    required this.guards,
  });
}

/// ガードレジストリ
class RouteGuardRegistry {
  void register(RouteGuard guard);
  void unregister(String name);
  RouteGuard? get(String name);
  bool contains(String name);
  void clear();
}
```

### K1s0Router

```dart
/// k1s0 Router ラッパー
class K1s0Router {
  final RouteConfig config;
  final RouteGuardRegistry guardRegistry;
  final List<NavigatorObserver> observers;
  final Widget Function(BuildContext, GoRouterState)? errorBuilder;
  final Listenable? refreshListenable;

  K1s0Router({
    required this.config,
    RouteGuardRegistry? guardRegistry,
    this.observers = const [],
    this.errorBuilder,
    this.refreshListenable,
  });

  /// GoRouter インスタンスを作成
  GoRouter createRouter();
}
```

### Shell Routes

```dart
/// Shell Route 設定
class K1s0ShellRoute {
  final String? name;
  final Widget Function(BuildContext, GoRouterState, Widget) builder;
  final List<RouteEntry> routes;
  final List<String> guards;
  final GlobalKey<NavigatorState>? navigatorKey;

  ShellRoute toShellRoute({required Map<String, RouteGuardCallback> guardCallbacks});
}

/// Stateful Shell Route（タブナビゲーション用）
class K1s0StatefulShellRoute {
  final List<K1s0ShellBranch> branches;
  final Widget Function(BuildContext, GoRouterState, StatefulNavigationShell) builder;
  final String? restorationScopeId;

  StatefulShellRoute toStatefulShellRoute({...});
}

class K1s0ShellBranch {
  final GlobalKey<NavigatorState>? navigatorKey;
  final String? restorationScopeId;
  final String? initialLocation;
  final List<RouteEntry> routes;
}
```

### Riverpod Providers

```dart
/// ルート設定プロバイダー
final routeConfigProvider = Provider<RouteConfig>((ref) => throw UnimplementedError());

/// ガードレジストリプロバイダー
final routeGuardRegistryProvider = Provider<RouteGuardRegistry>((ref) {
  return RouteGuardRegistry();
});

/// GoRouter プロバイダー
final routerProvider = Provider<GoRouter>((ref) {
  final config = ref.watch(routeConfigProvider);
  final registry = ref.watch(routeGuardRegistryProvider);

  final k1s0Router = K1s0Router(
    config: config,
    guardRegistry: registry,
  );

  return k1s0Router.createRouter();
});

/// 認証状態通知
class SimpleAuthStateNotifier extends ChangeNotifier {
  bool _isAuthenticated;

  bool get isAuthenticated => _isAuthenticated;

  void login() { ... }
  void logout() { ... }
}

final authStateProvider = ChangeNotifierProvider<SimpleAuthStateNotifier>((ref) {
  return SimpleAuthStateNotifier();
});
```

## 使用例

```dart
// ルート設定
final config = RouteConfigBuilder()
    .addRoute(RouteEntry(
      path: '/',
      name: 'home',
      builder: (context, state) => const HomePage(),
    ))
    .addRoute(RouteEntry(
      path: '/login',
      name: 'login',
      builder: (context, state) => const LoginPage(),
    ))
    .addRoute(RouteEntry(
      path: '/dashboard',
      name: 'dashboard',
      builder: (context, state) => const DashboardPage(),
      guards: ['auth'],  // 認証ガードを適用
    ))
    .initialLocation('/')
    .enableDebugLogging()
    .build();

// ガード登録
final registry = RouteGuardRegistry();
registry.register(AuthGuard(
  isAuthenticated: (context) => ref.read(authStateProvider).isAuthenticated,
  loginPath: '/login',
  excludedPaths: ['/login', '/register'],
));

// Router 作成
final router = K1s0Router(
  config: config,
  guardRegistry: registry,
  refreshListenable: ref.read(authStateProvider),
).createRouter();

// MaterialApp で使用
MaterialApp.router(
  routerConfig: router,
)

// Shell Route（共通レイアウト）
final shellRoute = K1s0ShellRoute(
  builder: (context, state, child) => Scaffold(
    appBar: AppBar(title: Text('My App')),
    body: child,
  ),
  routes: [
    RouteEntry(path: '/home', name: 'home', builder: ...),
    RouteEntry(path: '/settings', name: 'settings', builder: ...),
  ],
);

// Stateful Shell Route（タブナビゲーション）
final statefulShell = K1s0StatefulShellRoute(
  branches: [
    K1s0ShellBranch(
      routes: [RouteEntry(path: '/home', name: 'home', builder: ...)],
    ),
    K1s0ShellBranch(
      routes: [RouteEntry(path: '/profile', name: 'profile', builder: ...)],
    ),
  ],
  builder: (context, state, shell) => Scaffold(
    body: shell,
    bottomNavigationBar: BottomNavigationBar(
      currentIndex: shell.currentIndex,
      onTap: shell.goBranch,
      items: [...],
    ),
  ),
);
```
