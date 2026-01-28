import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'route_entry.dart';

/// Configuration for the k1s0 router.
///
/// This class holds the configuration for building a GoRouter instance.
class RouteConfig {
  /// Creates a route configuration.
  const RouteConfig({
    required this.routes,
    this.initialLocation = '/',
    this.redirect,
    this.refreshListenable,
    this.errorBuilder,
    this.errorPageBuilder,
    this.observers = const [],
    this.debugLogDiagnostics = false,
    this.routerNeglect = false,
    this.navigatorKey,
  });

  /// The list of top-level routes.
  final List<RouteEntry> routes;

  /// The initial location to navigate to.
  final String initialLocation;

  /// Global redirect function.
  final String? Function(BuildContext context, GoRouterState state)? redirect;

  /// Listenable to trigger router refresh.
  final Listenable? refreshListenable;

  /// Error widget builder.
  final Widget Function(BuildContext context, GoRouterState state)? errorBuilder;

  /// Error page builder.
  final Page<void> Function(BuildContext context, GoRouterState state)? errorPageBuilder;

  /// Navigation observers.
  final List<NavigatorObserver> observers;

  /// Whether to log diagnostic information.
  final bool debugLogDiagnostics;

  /// Whether to neglect the router.
  final bool routerNeglect;

  /// Global navigator key.
  final GlobalKey<NavigatorState>? navigatorKey;

  /// Creates a copy with modified values.
  RouteConfig copyWith({
    List<RouteEntry>? routes,
    String? initialLocation,
    String? Function(BuildContext context, GoRouterState state)? redirect,
    Listenable? refreshListenable,
    Widget Function(BuildContext context, GoRouterState state)? errorBuilder,
    Page<void> Function(BuildContext context, GoRouterState state)? errorPageBuilder,
    List<NavigatorObserver>? observers,
    bool? debugLogDiagnostics,
    bool? routerNeglect,
    GlobalKey<NavigatorState>? navigatorKey,
  }) {
    return RouteConfig(
      routes: routes ?? this.routes,
      initialLocation: initialLocation ?? this.initialLocation,
      redirect: redirect ?? this.redirect,
      refreshListenable: refreshListenable ?? this.refreshListenable,
      errorBuilder: errorBuilder ?? this.errorBuilder,
      errorPageBuilder: errorPageBuilder ?? this.errorPageBuilder,
      observers: observers ?? this.observers,
      debugLogDiagnostics: debugLogDiagnostics ?? this.debugLogDiagnostics,
      routerNeglect: routerNeglect ?? this.routerNeglect,
      navigatorKey: navigatorKey ?? this.navigatorKey,
    );
  }
}

/// Builder for creating RouteConfig instances.
class RouteConfigBuilder {
  /// Creates a new RouteConfigBuilder.
  RouteConfigBuilder();

  final List<RouteEntry> _routes = [];
  String _initialLocation = '/';
  String? Function(BuildContext context, GoRouterState state)? _redirect;
  Listenable? _refreshListenable;
  Widget Function(BuildContext context, GoRouterState state)? _errorBuilder;
  bool _debugLogDiagnostics = false;
  GlobalKey<NavigatorState>? _navigatorKey;

  /// Adds a route.
  RouteConfigBuilder addRoute(RouteEntry route) {
    _routes.add(route);
    return this;
  }

  /// Adds multiple routes.
  RouteConfigBuilder addRoutes(List<RouteEntry> routes) {
    _routes.addAll(routes);
    return this;
  }

  /// Sets the initial location.
  RouteConfigBuilder initialLocation(String location) {
    _initialLocation = location;
    return this;
  }

  /// Sets the global redirect.
  RouteConfigBuilder redirect(
    String? Function(BuildContext context, GoRouterState state) redirect,
  ) {
    _redirect = redirect;
    return this;
  }

  /// Sets the refresh listenable.
  RouteConfigBuilder refreshListenable(Listenable listenable) {
    _refreshListenable = listenable;
    return this;
  }

  /// Sets the error builder.
  RouteConfigBuilder errorBuilder(
    Widget Function(BuildContext context, GoRouterState state) builder,
  ) {
    _errorBuilder = builder;
    return this;
  }

  /// Enables debug logging.
  RouteConfigBuilder enableDebugLogging() {
    _debugLogDiagnostics = true;
    return this;
  }

  /// Sets the navigator key.
  RouteConfigBuilder navigatorKey(GlobalKey<NavigatorState> key) {
    _navigatorKey = key;
    return this;
  }

  /// Builds the RouteConfig.
  RouteConfig build() {
    return RouteConfig(
      routes: List.unmodifiable(_routes),
      initialLocation: _initialLocation,
      redirect: _redirect,
      refreshListenable: _refreshListenable,
      errorBuilder: _errorBuilder,
      debugLogDiagnostics: _debugLogDiagnostics,
      navigatorKey: _navigatorKey,
    );
  }
}
