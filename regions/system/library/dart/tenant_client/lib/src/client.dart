import 'tenant.dart';
import 'config.dart';
import 'error.dart';

abstract class TenantClient {
  Future<Tenant> getTenant(String tenantId);
  Future<List<Tenant>> listTenants(TenantFilter filter);
  Future<bool> isActive(String tenantId);
  Future<TenantSettings> getSettings(String tenantId);
  Future<Tenant> createTenant(CreateTenantRequest req);
  Future<TenantMember> addMember(String tenantId, String userId, String role);
  Future<void> removeMember(String tenantId, String userId);
  Future<List<TenantMember>> listMembers(String tenantId);
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId);
}

class InMemoryTenantClient implements TenantClient {
  final Map<String, Tenant> _tenants = {};
  final Map<String, List<TenantMember>> _members = {};
  final Map<String, ProvisioningStatus> _provisioning = {};

  InMemoryTenantClient([List<Tenant>? tenants]) {
    if (tenants != null) {
      for (final t in tenants) {
        _tenants[t.id] = t;
      }
    }
  }

  void addTenant(Tenant tenant) {
    _tenants[tenant.id] = tenant;
  }

  @override
  Future<Tenant> getTenant(String tenantId) async {
    final tenant = _tenants[tenantId];
    if (tenant == null)
      throw TenantError(
          'Tenant not found: $tenantId', TenantErrorCode.notFound);
    return tenant;
  }

  @override
  Future<List<Tenant>> listTenants(TenantFilter filter) async {
    var list = _tenants.values.toList();
    if (filter.status != null)
      list = list.where((t) => t.status == filter.status).toList();
    if (filter.plan != null)
      list = list.where((t) => t.plan == filter.plan).toList();
    return list;
  }

  @override
  Future<bool> isActive(String tenantId) async {
    return _tenants[tenantId]?.status == TenantStatus.active;
  }

  @override
  Future<TenantSettings> getSettings(String tenantId) async {
    final tenant = await getTenant(tenantId);
    return TenantSettings(tenant.settings);
  }

  @override
  Future<Tenant> createTenant(CreateTenantRequest req) async {
    final id = 'tenant-${DateTime.now().millisecondsSinceEpoch}';
    final tenant = Tenant(
      id: id,
      name: req.name,
      status: TenantStatus.active,
      plan: req.plan,
      settings: const {},
      createdAt: DateTime.now(),
    );
    _tenants[id] = tenant;
    _provisioning[id] = ProvisioningStatus.pending;
    return tenant;
  }

  @override
  Future<TenantMember> addMember(
      String tenantId, String userId, String role) async {
    if (!_tenants.containsKey(tenantId)) {
      throw TenantError(
          'Tenant not found: $tenantId', TenantErrorCode.notFound);
    }
    final member =
        TenantMember(userId: userId, role: role, joinedAt: DateTime.now());
    _members[tenantId] = [...(_members[tenantId] ?? []), member];
    return member;
  }

  @override
  Future<void> removeMember(String tenantId, String userId) async {
    _members[tenantId] =
        (_members[tenantId] ?? []).where((m) => m.userId != userId).toList();
  }

  @override
  Future<List<TenantMember>> listMembers(String tenantId) async {
    return List.from(_members[tenantId] ?? []);
  }

  @override
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId) async {
    final status = _provisioning[tenantId];
    if (status == null)
      throw TenantError(
          'Provisioning not found: $tenantId', TenantErrorCode.notFound);
    return status;
  }
}

/// gRPC 経由で tenant-server に接続するクライアント。
/// [config] の [TenantClientConfig.serverUrl] には "host:port" 形式のアドレスを指定する
/// （例: "tenant-server:8080"）。
class GrpcTenantClient implements TenantClient {
  final TenantClientConfig config;

  GrpcTenantClient(this.config);

  @override
  Future<Tenant> getTenant(String tenantId) async {
    throw const TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<List<Tenant>> listTenants(TenantFilter filter) async {
    throw const TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<bool> isActive(String tenantId) async {
    throw const TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<TenantSettings> getSettings(String tenantId) async {
    throw const TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<Tenant> createTenant(CreateTenantRequest req) {
    throw UnimplementedError('createTenant is not yet implemented');
  }

  @override
  Future<TenantMember> addMember(String tenantId, String userId, String role) {
    throw UnimplementedError('addMember is not yet implemented');
  }

  @override
  Future<void> removeMember(String tenantId, String userId) {
    throw UnimplementedError('removeMember is not yet implemented');
  }

  @override
  Future<List<TenantMember>> listMembers(String tenantId) {
    throw UnimplementedError('listMembers is not yet implemented');
  }

  @override
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId) {
    throw UnimplementedError('getProvisioningStatus is not yet implemented');
  }

  Future<void> close() async {
    // 接続クリーンアップ用プレースホルダー
  }
}
