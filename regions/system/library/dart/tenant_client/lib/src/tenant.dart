enum TenantStatus { active, suspended, deleted }

class Tenant {
  final String id;
  final String name;
  final TenantStatus status;
  final String plan;
  final Map<String, String> settings;
  final DateTime createdAt;

  const Tenant({
    required this.id,
    required this.name,
    required this.status,
    required this.plan,
    required this.settings,
    required this.createdAt,
  });
}

class TenantFilter {
  final TenantStatus? status;
  final String? plan;

  const TenantFilter({this.status, this.plan});
}

class TenantSettings {
  final Map<String, String> values;

  const TenantSettings(this.values);

  String? get(String key) => values[key];
}
