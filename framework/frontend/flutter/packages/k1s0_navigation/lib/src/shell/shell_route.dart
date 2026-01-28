import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../config/route_entry.dart';

/// Shell route configuration for nested navigation.
///
/// Shell routes allow you to create a persistent layout (like a bottom navigation
/// bar or sidebar) that wraps around child routes.
class K1s0ShellRoute {
  /// Creates a shell route.
  const K1s0ShellRoute({
    required this.builder,
    required this.routes,
    this.navigatorKey,
    this.observers = const [],
    this.restorationScopeId,
  });

  /// Builder for the shell (layout) widget.
  ///
  /// The [child] parameter is the widget for the current matched child route.
  final Widget Function(BuildContext context, GoRouterState state, Widget child) builder;

  /// The child routes.
  final List<RouteEntry> routes;

  /// Optional navigator key for this shell's navigator.
  final GlobalKey<NavigatorState>? navigatorKey;

  /// Navigation observers for this shell's navigator.
  final List<NavigatorObserver> observers;

  /// Restoration scope ID for state restoration.
  final String? restorationScopeId;

  /// Converts this to a ShellRoute.
  ShellRoute toShellRoute({
    required Map<String, RouteGuardCallback> guardCallbacks,
  }) {
    return ShellRoute(
      navigatorKey: navigatorKey,
      builder: (context, state, child) => builder(context, state, child),
      observers: observers,
      restorationScopeId: restorationScopeId,
      routes: routes
          .map((entry) => entry.toGoRoute(guardCallbacks: guardCallbacks))
          .toList(),
    );
  }
}

/// Stateful shell route for indexed navigation (bottom navigation, tabs).
class K1s0StatefulShellRoute {
  /// Creates a stateful shell route.
  const K1s0StatefulShellRoute({
    required this.branches,
    this.builder,
    this.navigatorContainerBuilder,
    this.restorationScopeId,
  }) : assert(builder != null || navigatorContainerBuilder != null);

  /// The branches (tabs) for this shell.
  final List<K1s0ShellBranch> branches;

  /// Builder for the shell widget.
  final Widget Function(
    BuildContext context,
    GoRouterState state,
    StatefulNavigationShell navigationShell,
  )? builder;

  /// Builder for the navigator container.
  final Widget Function(
    BuildContext context,
    StatefulNavigationShell navigationShell,
    List<Widget> children,
  )? navigatorContainerBuilder;

  /// Restoration scope ID.
  final String? restorationScopeId;

  /// Converts this to a StatefulShellRoute.
  StatefulShellRoute toStatefulShellRoute({
    required Map<String, RouteGuardCallback> guardCallbacks,
  }) {
    if (navigatorContainerBuilder != null) {
      return StatefulShellRoute.indexedStack(
        branches: branches
            .map((branch) => branch.toStatefulShellBranch(guardCallbacks: guardCallbacks))
            .toList(),
        builder: builder,
        restorationScopeId: restorationScopeId,
      );
    }

    return StatefulShellRoute.indexedStack(
      branches: branches
          .map((branch) => branch.toStatefulShellBranch(guardCallbacks: guardCallbacks))
          .toList(),
      builder: builder,
      restorationScopeId: restorationScopeId,
    );
  }
}

/// A branch in a stateful shell route.
class K1s0ShellBranch {
  /// Creates a shell branch.
  const K1s0ShellBranch({
    required this.routes,
    this.navigatorKey,
    this.initialLocation,
    this.restorationScopeId,
    this.observers = const [],
  });

  /// The routes in this branch.
  final List<RouteEntry> routes;

  /// Navigator key for this branch.
  final GlobalKey<NavigatorState>? navigatorKey;

  /// Initial location for this branch.
  final String? initialLocation;

  /// Restoration scope ID.
  final String? restorationScopeId;

  /// Navigation observers for this branch.
  final List<NavigatorObserver> observers;

  /// Converts this to a StatefulShellBranch.
  StatefulShellBranch toStatefulShellBranch({
    required Map<String, RouteGuardCallback> guardCallbacks,
  }) {
    return StatefulShellBranch(
      navigatorKey: navigatorKey,
      initialLocation: initialLocation,
      restorationScopeId: restorationScopeId,
      observers: observers,
      routes: routes
          .map((entry) => entry.toGoRoute(guardCallbacks: guardCallbacks))
          .toList(),
    );
  }
}

/// Scaffold with bottom navigation for stateful shell routes.
///
/// This is a convenience widget for creating a scaffold with bottom navigation
/// that works with StatefulShellRoute.
class K1s0NavigationScaffold extends StatelessWidget {
  /// Creates a navigation scaffold.
  const K1s0NavigationScaffold({
    super.key,
    required this.navigationShell,
    required this.destinations,
    this.appBar,
    this.floatingActionButton,
    this.drawer,
    this.endDrawer,
    this.backgroundColor,
    this.navigationBarBackgroundColor,
  });

  /// The stateful navigation shell.
  final StatefulNavigationShell navigationShell;

  /// The navigation destinations.
  final List<NavigationDestination> destinations;

  /// Optional app bar.
  final PreferredSizeWidget? appBar;

  /// Optional floating action button.
  final Widget? floatingActionButton;

  /// Optional drawer.
  final Widget? drawer;

  /// Optional end drawer.
  final Widget? endDrawer;

  /// Background color for the scaffold.
  final Color? backgroundColor;

  /// Background color for the navigation bar.
  final Color? navigationBarBackgroundColor;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: appBar,
      body: navigationShell,
      floatingActionButton: floatingActionButton,
      drawer: drawer,
      endDrawer: endDrawer,
      backgroundColor: backgroundColor,
      bottomNavigationBar: NavigationBar(
        backgroundColor: navigationBarBackgroundColor,
        selectedIndex: navigationShell.currentIndex,
        onDestinationSelected: (index) => navigationShell.goBranch(
          index,
          initialLocation: index == navigationShell.currentIndex,
        ),
        destinations: destinations,
      ),
    );
  }
}
