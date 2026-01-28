/// k1s0 Navigation Library
///
/// Provides config-driven routing for k1s0 Flutter applications using go_router.
///
/// ## Features
///
/// - Config-driven route definition
/// - Type-safe route parameters
/// - Deep linking support
/// - Route guards (authentication, authorization)
/// - Nested navigation
/// - Shell routes for layout
///
/// ## Usage
///
/// ```dart
/// final routeConfig = RouteConfig(
///   routes: [
///     RouteEntry(
///       path: '/',
///       name: 'home',
///       builder: (context, state) => const HomePage(),
///     ),
///     RouteEntry(
///       path: '/users/:id',
///       name: 'user-detail',
///       builder: (context, state) => UserDetailPage(
///         userId: state.pathParameters['id']!,
///       ),
///     ),
///   ],
/// );
///
/// final router = K1s0Router.fromConfig(routeConfig);
/// ```
library k1s0_navigation;

export 'src/config/route_config.dart';
export 'src/config/route_entry.dart';
export 'src/guards/auth_guard.dart';
export 'src/guards/route_guard.dart';
export 'src/provider/navigation_provider.dart';
export 'src/router/k1s0_router.dart';
export 'src/shell/shell_route.dart';
