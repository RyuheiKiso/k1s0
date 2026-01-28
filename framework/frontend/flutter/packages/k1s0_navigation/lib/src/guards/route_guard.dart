import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

/// Base interface for route guards.
///
/// Route guards are used to protect routes from unauthorized access.
/// They can redirect to another route or allow access.
abstract class RouteGuard {
  /// The name of this guard for configuration.
  String get name;

  /// Checks if the current state passes the guard.
  ///
  /// Returns null if the guard passes, or a redirect location if it fails.
  String? check(BuildContext context, GoRouterState state);
}

/// A functional route guard.
///
/// This allows creating guards without implementing a class.
class FunctionalRouteGuard implements RouteGuard {
  /// Creates a functional route guard.
  const FunctionalRouteGuard({
    required this.name,
    required this.checkFn,
  });

  @override
  final String name;

  /// The check function.
  final String? Function(BuildContext context, GoRouterState state) checkFn;

  @override
  String? check(BuildContext context, GoRouterState state) => checkFn(context, state);
}

/// A composite guard that checks multiple guards.
///
/// All guards must pass for this guard to pass.
class CompositeRouteGuard implements RouteGuard {
  /// Creates a composite route guard.
  const CompositeRouteGuard({
    required this.name,
    required this.guards,
  });

  @override
  final String name;

  /// The list of guards to check.
  final List<RouteGuard> guards;

  @override
  String? check(BuildContext context, GoRouterState state) {
    for (final guard in guards) {
      final redirect = guard.check(context, state);
      if (redirect != null) {
        return redirect;
      }
    }
    return null;
  }
}

/// A guard that checks for a specific role.
class RoleGuard implements RouteGuard {
  /// Creates a role guard.
  const RoleGuard({
    required this.name,
    required this.requiredRoles,
    required this.currentRolesProvider,
    this.redirectTo = '/unauthorized',
  });

  @override
  final String name;

  /// The roles required to pass this guard.
  final Set<String> requiredRoles;

  /// Provider function to get current user roles.
  final Set<String> Function(BuildContext context) currentRolesProvider;

  /// Location to redirect to if guard fails.
  final String redirectTo;

  @override
  String? check(BuildContext context, GoRouterState state) {
    final currentRoles = currentRolesProvider(context);
    final hasRequiredRole = requiredRoles.any(currentRoles.contains);

    if (!hasRequiredRole) {
      return redirectTo;
    }

    return null;
  }
}

/// Registry for route guards.
///
/// This class manages the registration and lookup of route guards.
class RouteGuardRegistry {
  /// Creates a route guard registry.
  RouteGuardRegistry();

  final Map<String, RouteGuard> _guards = {};

  /// Registers a guard.
  void register(RouteGuard guard) {
    _guards[guard.name] = guard;
  }

  /// Registers multiple guards.
  void registerAll(List<RouteGuard> guards) {
    for (final guard in guards) {
      register(guard);
    }
  }

  /// Gets a guard by name.
  RouteGuard? get(String name) => _guards[name];

  /// Gets all guard callbacks for use with RouteEntry.
  Map<String, String? Function(BuildContext, GoRouterState)> getCallbacks() {
    return _guards.map((name, guard) => MapEntry(
      name,
      (context, state) => guard.check(context, state),
    ));
  }

  /// Checks if a guard is registered.
  bool contains(String name) => _guards.containsKey(name);

  /// Unregisters a guard.
  void unregister(String name) {
    _guards.remove(name);
  }

  /// Clears all guards.
  void clear() {
    _guards.clear();
  }
}
