import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../config/route_config.dart';
import '../config/route_entry.dart';
import '../guards/route_guard.dart';

/// k1s0 Router
///
/// A config-driven router wrapper around go_router.
class K1s0Router {
  /// Creates a k1s0 router from configuration.
  K1s0Router.fromConfig(
    RouteConfig config, {
    RouteGuardRegistry? guardRegistry,
  }) {
    _guardRegistry = guardRegistry ?? RouteGuardRegistry();
    _router = _buildRouter(config);
  }

  late final RouteGuardRegistry _guardRegistry;
  late final GoRouter _router;

  /// Gets the underlying GoRouter instance.
  GoRouter get router => _router;

  /// Gets the router configuration for MaterialApp.router.
  RouterConfig<Object> get routerConfig => _router;

  /// Gets the guard registry.
  RouteGuardRegistry get guardRegistry => _guardRegistry;

  GoRouter _buildRouter(RouteConfig config) {
    final guardCallbacks = _guardRegistry.getCallbacks();

    final routes = config.routes
        .map((entry) => entry.toGoRoute(guardCallbacks: guardCallbacks))
        .toList();

    return GoRouter(
      routes: routes,
      initialLocation: config.initialLocation,
      redirect: config.redirect,
      refreshListenable: config.refreshListenable,
      errorBuilder: config.errorBuilder,
      errorPageBuilder: config.errorPageBuilder,
      observers: config.observers,
      debugLogDiagnostics: config.debugLogDiagnostics,
      routerNeglect: config.routerNeglect,
      navigatorKey: config.navigatorKey,
    );
  }

  /// Navigates to a location.
  void go(String location, {Object? extra}) {
    _router.go(location, extra: extra);
  }

  /// Navigates to a named route.
  void goNamed(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    _router.goNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pushes a location onto the stack.
  Future<T?> push<T>(String location, {Object? extra}) {
    return _router.push(location, extra: extra);
  }

  /// Pushes a named route onto the stack.
  Future<T?> pushNamed<T>(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    return _router.pushNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Replaces the current location.
  void pushReplacement(String location, {Object? extra}) {
    _router.pushReplacement(location, extra: extra);
  }

  /// Replaces with a named route.
  void pushReplacementNamed(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    _router.pushReplacementNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pops the current location.
  void pop<T>([T? result]) {
    _router.pop(result);
  }

  /// Whether the navigator can pop.
  bool canPop() {
    return _router.canPop();
  }

  /// Refreshes the router.
  void refresh() {
    _router.refresh();
  }

  /// Gets the current location.
  String get location => _router.routerDelegate.currentConfiguration.fullPath;

  /// Disposes the router.
  void dispose() {
    _router.dispose();
  }
}

/// Extension methods for BuildContext.
extension K1s0NavigationExtension on BuildContext {
  /// Gets the GoRouter instance.
  GoRouter get router => GoRouter.of(this);

  /// Navigates to a location.
  void go(String location, {Object? extra}) {
    GoRouter.of(this).go(location, extra: extra);
  }

  /// Navigates to a named route.
  void goNamed(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    GoRouter.of(this).goNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pushes a location onto the stack.
  Future<T?> push<T>(String location, {Object? extra}) {
    return GoRouter.of(this).push(location, extra: extra);
  }

  /// Pushes a named route onto the stack.
  Future<T?> pushNamed<T>(
    String name, {
    Map<String, String> pathParameters = const {},
    Map<String, dynamic> queryParameters = const {},
    Object? extra,
  }) {
    return GoRouter.of(this).pushNamed(
      name,
      pathParameters: pathParameters,
      queryParameters: queryParameters,
      extra: extra,
    );
  }

  /// Pops the current location.
  void pop<T>([T? result]) {
    GoRouter.of(this).pop(result);
  }

  /// Whether the navigator can pop.
  bool canPop() {
    return GoRouter.of(this).canPop();
  }
}
