import 'package:dio/dio.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import 'navigation_types.dart';

enum NavigationMode { remote, local }

typedef ComponentRegistry
    = Map<String, Widget Function(BuildContext, GoRouterState)>;

class NavigationInterpreter {
  const NavigationInterpreter({
    required this.mode,
    required this.componentRegistry,
    this.dio,
    this.remoteUrl = '/api/v1/navigation',
    this.localConfigAsset = 'assets/navigation.yaml',
  });

  final NavigationMode mode;
  final ComponentRegistry componentRegistry;
  final Dio? dio;
  final String remoteUrl;
  final String localConfigAsset;

  Future<GoRouter> build() async {
    final nav = await _fetchNavigation();
    return _buildGoRouter(nav);
  }

  Future<NavigationResponse> _fetchNavigation() async {
    if (mode == NavigationMode.local) {
      final yaml = await rootBundle.loadString(localConfigAsset);
      return NavigationResponse.fromYaml(yaml);
    }
    final response = await dio!.get<Map<String, dynamic>>(remoteUrl);
    return NavigationResponse.fromJson(response.data!);
  }

  GoRouter _buildGoRouter(NavigationResponse nav) {
    final guardMap = {for (final g in nav.guards) g.id: g};
    final routes = nav.routes.map((r) => _buildRoute(r, guardMap)).toList();
    return GoRouter(routes: routes);
  }

  GoRoute _buildRoute(
    NavigationRoute route,
    Map<String, NavigationGuard> guardMap,
  ) {
    if (route.redirectTo != null) {
      return GoRoute(
        path: route.path,
        redirect: (_, __) => route.redirectTo,
      );
    }

    return GoRoute(
      path: route.path,
      builder: (context, state) {
        final builder = componentRegistry[route.componentId];
        if (builder == null) {
          return const Center(child: Text('Component not found'));
        }
        return builder(context, state);
      },
      routes: route.children.map((c) => _buildRoute(c, guardMap)).toList(),
    );
  }
}
