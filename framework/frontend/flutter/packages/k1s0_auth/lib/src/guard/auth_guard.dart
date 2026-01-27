import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../provider/auth_provider.dart';
import '../provider/auth_state.dart';
import '../types/auth_user.dart';

/// Authentication guard for GoRouter
///
/// Use this to protect routes that require authentication.
class AuthGuard {
  /// Creates an auth guard
  const AuthGuard({
    this.loginPath = '/login',
    this.homePath = '/',
    this.roles,
    this.permissions,
  });

  /// Path to redirect to when not authenticated
  final String loginPath;

  /// Path to redirect to when authenticated (for login page)
  final String homePath;

  /// Required roles (any of these)
  final List<String>? roles;

  /// Required permissions (all of these)
  final List<String>? permissions;

  /// Redirect function for GoRouter
  String? redirect(BuildContext context, GoRouterState state, AuthState auth) {
    final isLoggedIn = auth.isAuthenticated;
    final isLoggingIn = state.matchedLocation == loginPath;
    final isInitializing = !auth.isInitialized;

    // Don't redirect while initializing
    if (isInitializing) {
      return null;
    }

    // If not logged in and not on login page, redirect to login
    if (!isLoggedIn && !isLoggingIn) {
      return loginPath;
    }

    // If logged in and on login page, redirect to home
    if (isLoggedIn && isLoggingIn) {
      return homePath;
    }

    // Check role requirements
    if (isLoggedIn && roles != null && roles!.isNotEmpty) {
      final user = auth.user;
      if (user == null || !user.hasAnyRole(roles!)) {
        // User doesn't have required role
        return homePath;
      }
    }

    // Check permission requirements
    if (isLoggedIn && permissions != null && permissions!.isNotEmpty) {
      final user = auth.user;
      if (user == null || !user.hasAllPermissions(permissions!)) {
        // User doesn't have required permissions
        return homePath;
      }
    }

    return null;
  }
}

/// Extension to create a redirect function that uses AuthGuard
extension AuthGuardExtension on AuthGuard {
  /// Create a redirect function for GoRouter
  GoRouterRedirect createRedirect(WidgetRef ref) {
    return (context, state) {
      final auth = ref.read(authProvider);
      return redirect(context, state, auth);
    };
  }
}

/// Widget that shows content only when authenticated
class RequireAuth extends ConsumerWidget {
  /// Creates a RequireAuth widget
  const RequireAuth({
    required this.child,
    this.roles,
    this.permissions,
    this.loading,
    this.fallback,
    super.key,
  });

  /// Child widget to show when authenticated
  final Widget child;

  /// Required roles (any of these)
  final List<String>? roles;

  /// Required permissions (all of these)
  final List<String>? permissions;

  /// Widget to show while loading
  final Widget? loading;

  /// Widget to show when not authorized
  final Widget? fallback;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final auth = ref.watch(authProvider);

    // Show loading while initializing
    if (!auth.isInitialized || auth.isLoading) {
      return loading ?? const SizedBox.shrink();
    }

    // Not authenticated
    if (!auth.isAuthenticated || auth.user == null) {
      return fallback ?? const SizedBox.shrink();
    }

    // Check role requirements
    if (roles != null && roles!.isNotEmpty) {
      if (!auth.user!.hasAnyRole(roles!)) {
        return fallback ?? const SizedBox.shrink();
      }
    }

    // Check permission requirements
    if (permissions != null && permissions!.isNotEmpty) {
      if (!auth.user!.hasAllPermissions(permissions!)) {
        return fallback ?? const SizedBox.shrink();
      }
    }

    return child;
  }
}

/// Widget that shows content only when user has a specific role
class RequireRole extends ConsumerWidget {
  /// Creates a RequireRole widget
  const RequireRole({
    required this.roles,
    required this.child,
    this.fallback,
    super.key,
  });

  /// Required roles (any of these)
  final List<String> roles;

  /// Child widget to show when authorized
  final Widget child;

  /// Widget to show when not authorized
  final Widget? fallback;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final hasRole = ref.watch(hasAnyRoleProvider(roles));

    if (hasRole) {
      return child;
    }

    return fallback ?? const SizedBox.shrink();
  }
}

/// Widget that shows content only when user has a specific permission
class RequirePermission extends ConsumerWidget {
  /// Creates a RequirePermission widget
  const RequirePermission({
    required this.permissions,
    required this.child,
    this.requireAll = false,
    this.fallback,
    super.key,
  });

  /// Required permissions
  final List<String> permissions;

  /// Child widget to show when authorized
  final Widget child;

  /// Whether all permissions are required (default: any)
  final bool requireAll;

  /// Widget to show when not authorized
  final Widget? fallback;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final user = ref.watch(currentUserProvider);

    if (user == null) {
      return fallback ?? const SizedBox.shrink();
    }

    final hasPermission = requireAll
        ? user.hasAllPermissions(permissions)
        : user.hasAnyPermission(permissions);

    if (hasPermission) {
      return child;
    }

    return fallback ?? const SizedBox.shrink();
  }
}

/// Create a GoRouter redirect that uses authentication
GoRouterRedirect createAuthRedirect(
  WidgetRef ref, {
  String loginPath = '/login',
  String homePath = '/',
  List<String>? roles,
  List<String>? permissions,
}) {
  return (context, state) {
    final auth = ref.read(authProvider);
    final guard = AuthGuard(
      loginPath: loginPath,
      homePath: homePath,
      roles: roles,
      permissions: permissions,
    );
    return guard.redirect(context, state, auth);
  };
}
