import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'route_guard.dart';

/// Authentication guard for protecting routes.
///
/// This guard checks if the user is authenticated before allowing access to
/// protected routes.
class AuthGuard implements RouteGuard {
  /// Creates an authentication guard.
  const AuthGuard({
    this.name = 'auth',
    required this.isAuthenticated,
    this.loginPath = '/login',
    this.returnToParameter = 'returnTo',
    this.excludedPaths = const ['/login', '/register', '/forgot-password'],
  });

  @override
  final String name;

  /// Provider function to check if user is authenticated.
  final bool Function(BuildContext context) isAuthenticated;

  /// Path to redirect to if not authenticated.
  final String loginPath;

  /// Query parameter name for return URL.
  final String returnToParameter;

  /// Paths that don't require authentication.
  final List<String> excludedPaths;

  @override
  String? check(BuildContext context, GoRouterState state) {
    // Skip guard for excluded paths
    if (excludedPaths.contains(state.matchedLocation)) {
      return null;
    }

    // Check authentication
    if (!isAuthenticated(context)) {
      // Build return URL
      final returnTo = Uri.encodeComponent(state.uri.toString());
      return '$loginPath?$returnToParameter=$returnTo';
    }

    return null;
  }
}

/// A stateful authentication guard using a notifier.
///
/// This guard listens to authentication state changes and refreshes the router.
class AuthStateGuard implements RouteGuard {
  /// Creates a stateful authentication guard.
  AuthStateGuard({
    this.name = 'authState',
    required this.authNotifier,
    this.loginPath = '/login',
    this.returnToParameter = 'returnTo',
    this.excludedPaths = const ['/login', '/register', '/forgot-password'],
  });

  @override
  final String name;

  /// Notifier that provides authentication state.
  final AuthStateNotifier authNotifier;

  /// Path to redirect to if not authenticated.
  final String loginPath;

  /// Query parameter name for return URL.
  final String returnToParameter;

  /// Paths that don't require authentication.
  final List<String> excludedPaths;

  @override
  String? check(BuildContext context, GoRouterState state) {
    // Skip guard for excluded paths
    if (excludedPaths.contains(state.matchedLocation)) {
      return null;
    }

    // Check authentication
    if (!authNotifier.isAuthenticated) {
      // Build return URL
      final returnTo = Uri.encodeComponent(state.uri.toString());
      return '$loginPath?$returnToParameter=$returnTo';
    }

    return null;
  }

  /// Gets the listenable for router refresh.
  Listenable get refreshListenable => authNotifier;
}

/// Notifier for authentication state.
///
/// Implement this class to provide authentication state to the router.
abstract class AuthStateNotifier extends ChangeNotifier {
  /// Whether the user is currently authenticated.
  bool get isAuthenticated;
}

/// Simple implementation of AuthStateNotifier.
class SimpleAuthStateNotifier extends AuthStateNotifier {
  SimpleAuthStateNotifier({bool isAuthenticated = false})
      : _isAuthenticated = isAuthenticated;

  bool _isAuthenticated;

  @override
  bool get isAuthenticated => _isAuthenticated;

  /// Sets the authentication state.
  set isAuthenticated(bool value) {
    if (_isAuthenticated != value) {
      _isAuthenticated = value;
      notifyListeners();
    }
  }

  /// Logs the user in.
  void login() {
    isAuthenticated = true;
  }

  /// Logs the user out.
  void logout() {
    isAuthenticated = false;
  }
}
