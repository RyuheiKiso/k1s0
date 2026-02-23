import 'package:yaml/yaml.dart';

enum GuardType {
  authRequired,
  roleRequired,
  redirectIfAuthenticated;

  static GuardType fromString(String value) => switch (value) {
        'auth_required' => GuardType.authRequired,
        'role_required' => GuardType.roleRequired,
        'redirect_if_authenticated' => GuardType.redirectIfAuthenticated,
        _ => GuardType.authRequired,
      };
}

enum TransitionType { fade, slide, modal }

enum ParamType { string, int, uuid }

class NavigationParam {
  const NavigationParam({required this.name, required this.type});

  final String name;
  final ParamType type;

  factory NavigationParam.fromJson(Map<String, dynamic> json) =>
      NavigationParam(
        name: json['name'] as String,
        type: switch (json['type']) {
          'int' => ParamType.int,
          'uuid' => ParamType.uuid,
          _ => ParamType.string,
        },
      );
}

class NavigationGuard {
  const NavigationGuard({
    required this.id,
    required this.type,
    required this.redirectTo,
    this.roles = const [],
  });

  final String id;
  final GuardType type;
  final String redirectTo;
  final List<String> roles;

  factory NavigationGuard.fromJson(Map<String, dynamic> json) =>
      NavigationGuard(
        id: json['id'] as String,
        type: GuardType.fromString(json['type'] as String),
        redirectTo: json['redirect_to'] as String,
        roles: (json['roles'] as List<dynamic>?)?.cast<String>() ?? [],
      );
}

class NavigationRoute {
  const NavigationRoute({
    required this.id,
    required this.path,
    this.componentId,
    this.guards = const [],
    this.transition,
    this.redirectTo,
    this.children = const [],
    this.params = const [],
  });

  final String id;
  final String path;
  final String? componentId;
  final List<String> guards;
  final String? transition;
  final String? redirectTo;
  final List<NavigationRoute> children;
  final List<NavigationParam> params;

  factory NavigationRoute.fromJson(Map<String, dynamic> json) =>
      NavigationRoute(
        id: json['id'] as String,
        path: json['path'] as String,
        componentId: json['component_id'] as String?,
        guards: (json['guards'] as List<dynamic>?)?.cast<String>() ?? [],
        transition: json['transition'] as String?,
        redirectTo: json['redirect_to'] as String?,
        children: (json['children'] as List<dynamic>?)
                ?.map(
                    (c) => NavigationRoute.fromJson(c as Map<String, dynamic>))
                .toList() ??
            [],
        params: (json['params'] as List<dynamic>?)
                ?.map(
                    (p) => NavigationParam.fromJson(p as Map<String, dynamic>))
                .toList() ??
            [],
      );
}

class NavigationResponse {
  const NavigationResponse({required this.routes, required this.guards});

  final List<NavigationRoute> routes;
  final List<NavigationGuard> guards;

  factory NavigationResponse.fromJson(Map<String, dynamic> json) =>
      NavigationResponse(
        routes: (json['routes'] as List<dynamic>)
            .map((r) => NavigationRoute.fromJson(r as Map<String, dynamic>))
            .toList(),
        guards: (json['guards'] as List<dynamic>? ?? [])
            .map((g) => NavigationGuard.fromJson(g as Map<String, dynamic>))
            .toList(),
      );

  factory NavigationResponse.fromYaml(String yamlStr) {
    final yaml = loadYaml(yamlStr);
    return NavigationResponse.fromJson(
        _convertYaml(yaml) as Map<String, dynamic>);
  }

  static dynamic _convertYaml(dynamic yaml) {
    if (yaml is YamlMap) {
      return Map<String, dynamic>.fromEntries(
        yaml.entries
            .map((e) => MapEntry(e.key.toString(), _convertYaml(e.value))),
      );
    } else if (yaml is YamlList) {
      return yaml.map(_convertYaml).toList();
    }
    return yaml;
  }
}
