import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

/// A route entry in the navigation configuration.
///
/// This class represents a single route with its path, builder, and optional guards.
class RouteEntry {
  /// Creates a route entry.
  const RouteEntry({
    required this.path,
    required this.name,
    this.builder,
    this.pageBuilder,
    this.redirect,
    this.guards = const [],
    this.children = const [],
    this.parentNavigatorKey,
    this.extra,
  }) : assert(builder != null || pageBuilder != null,
      'Either builder or pageBuilder must be provided');

  /// The path pattern for this route.
  ///
  /// Example: '/users/:id', '/settings'
  final String path;

  /// The name of this route for named navigation.
  final String name;

  /// Builder for the widget to display.
  final Widget Function(BuildContext context, GoRouterState state)? builder;

  /// Custom page builder for advanced transitions.
  final Page<void> Function(BuildContext context, GoRouterState state)? pageBuilder;

  /// Redirect function for this route.
  final String? Function(BuildContext context, GoRouterState state)? redirect;

  /// Guards to check before navigating to this route.
  final List<String> guards;

  /// Child routes nested under this route.
  final List<RouteEntry> children;

  /// Navigator key for nested navigation.
  final GlobalKey<NavigatorState>? parentNavigatorKey;

  /// Extra configuration data.
  final Map<String, dynamic>? extra;

  /// Converts this entry to a GoRoute.
  GoRoute toGoRoute({
    required Map<String, RouteGuardCallback> guardCallbacks,
  }) {
    return GoRoute(
      path: path,
      name: name,
      parentNavigatorKey: parentNavigatorKey,
      redirect: (context, state) {
        // Check guards
        for (final guardName in guards) {
          final callback = guardCallbacks[guardName];
          if (callback != null) {
            final redirect = callback(context, state);
            if (redirect != null) {
              return redirect;
            }
          }
        }
        // Check route-specific redirect
        return redirect?.call(context, state);
      },
      builder: builder != null
          ? (context, state) => builder!(context, state)
          : null,
      pageBuilder: pageBuilder != null
          ? (context, state) => pageBuilder!(context, state)
          : null,
      routes: children
          .map((child) => child.toGoRoute(guardCallbacks: guardCallbacks))
          .toList(),
    );
  }
}

/// Callback type for route guards.
typedef RouteGuardCallback = String? Function(BuildContext context, GoRouterState state);
