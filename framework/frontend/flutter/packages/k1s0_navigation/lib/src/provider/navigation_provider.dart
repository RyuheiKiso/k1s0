import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../config/route_config.dart';
import '../config/route_entry.dart';
import '../guards/route_guard.dart';
import '../router/k1s0_router.dart';

/// Provider for the route configuration.
///
/// Override this provider to provide your route configuration.
final routeConfigProvider = Provider<RouteConfig>((ref) {
  throw UnimplementedError(
    'You must override routeConfigProvider with your route configuration',
  );
});

/// Provider for the guard registry.
final guardRegistryProvider = Provider<RouteGuardRegistry>((ref) {
  return RouteGuardRegistry();
});

/// Provider for the K1s0Router.
final k1s0RouterProvider = Provider<K1s0Router>((ref) {
  final config = ref.watch(routeConfigProvider);
  final guardRegistry = ref.watch(guardRegistryProvider);

  return K1s0Router.fromConfig(config, guardRegistry: guardRegistry);
});

/// Provider for the GoRouter.
final goRouterProvider = Provider<GoRouter>((ref) {
  return ref.watch(k1s0RouterProvider).router;
});

/// Provider for the current location.
final currentLocationProvider = Provider<String>((ref) {
  return ref.watch(k1s0RouterProvider).location;
});

/// Mixin for navigation functionality.
///
/// Add this mixin to your ConsumerWidget or ConsumerStatefulWidget to get
/// convenient navigation methods.
mixin NavigationMixin on ConsumerWidget {
  /// Gets the router.
  K1s0Router router(WidgetRef ref) => ref.read(k1s0RouterProvider);

  /// Navigates to a location.
  void go(WidgetRef ref, String location, {Object? extra}) {
    ref.read(k1s0RouterProvider).go(location, extra: extra);
  }

  /// Navigates to a named route.
  void goNamed(
    WidgetRef ref,
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    ref.read(k1s0RouterProvider).goNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pushes a location onto the stack.
  Future<T?> push<T>(WidgetRef ref, String location, {Object? extra}) {
    return ref.read(k1s0RouterProvider).push(location, extra: extra);
  }

  /// Pops the current location.
  void pop<T>(WidgetRef ref, [T? result]) {
    ref.read(k1s0RouterProvider).pop(result);
  }
}

/// Extension on WidgetRef for navigation.
extension NavigationWidgetRefExtension on WidgetRef {
  /// Gets the K1s0Router.
  K1s0Router get router => read(k1s0RouterProvider);

  /// Navigates to a location.
  void go(String location, {Object? extra}) {
    read(k1s0RouterProvider).go(location, extra: extra);
  }

  /// Navigates to a named route.
  void goNamed(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    read(k1s0RouterProvider).goNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pushes a location onto the stack.
  Future<T?> push<T>(String location, {Object? extra}) {
    return read(k1s0RouterProvider).push(location, extra: extra);
  }

  /// Pops the current location.
  void pop<T>([T? result]) {
    read(k1s0RouterProvider).pop(result);
  }
}

/// Helper class for creating a router with Riverpod.
class K1s0NavigationScope extends ConsumerWidget {
  /// Creates a navigation scope.
  const K1s0NavigationScope({
    super.key,
    required this.child,
  });

  /// The child widget (usually MaterialApp.router).
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Ensure router is created
    ref.watch(k1s0RouterProvider);
    return child;
  }
}

/// Creates overrides for route configuration.
///
/// Use this to provide your route configuration in ProviderScope.
List<Override> createNavigationOverrides({
  required RouteConfig config,
  List<RouteGuard> guards = const [],
}) {
  return [
    routeConfigProvider.overrideWithValue(config),
    guardRegistryProvider.overrideWith((ref) {
      final registry = RouteGuardRegistry();
      registry.registerAll(guards);
      return registry;
    }),
  ];
}
