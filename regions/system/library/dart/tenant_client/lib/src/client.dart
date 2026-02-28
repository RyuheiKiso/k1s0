import 'tenant.dart';
import 'config.dart';
import 'error.dart';

abstract class TenantClient {
  Future<Tenant> getTenant(String tenantId);
  Future<List<Tenant>> listTenants(TenantFilter filter);
  Future<bool> isActive(String tenantId);
  Future<TenantSettings> getSettings(String tenantId);
}

class InMemoryTenantClient implements TenantClient {
  final List<Tenant> _tenants = [];

  InMemoryTenantClient([List<Tenant>? tenants]) {
    if (tenants != null) {
      _tenants.addAll(tenants);
    }
  }

  void addTenant(Tenant tenant) => _tenants.add(tenant);

  @override
  Future<Tenant> getTenant(String tenantId) async {
    for (final t in _tenants) {
      if (t.id == tenantId) return t;
    }
    throw TenantError('Tenant not found: $tenantId', TenantErrorCode.notFound);
  }

  @override
  Future<List<Tenant>> listTenants(TenantFilter filter) async {
    return _tenants.where((t) {
      if (filter.status != null && t.status != filter.status) return false;
      if (filter.plan != null && t.plan != filter.plan) return false;
      return true;
    }).toList();
  }

  @override
  Future<bool> isActive(String tenantId) async {
    final tenant = await getTenant(tenantId);
    return tenant.status == TenantStatus.active;
  }

  @override
  Future<TenantSettings> getSettings(String tenantId) async {
    final tenant = await getTenant(tenantId);
    return TenantSettings(tenant.settings);
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
    throw TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<List<Tenant>> listTenants(TenantFilter filter) async {
    throw TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<bool> isActive(String tenantId) async {
    throw TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  @override
  Future<TenantSettings> getSettings(String tenantId) async {
    throw TenantError(
      'gRPC client not yet connected',
      TenantErrorCode.serverError,
    );
  }

  Future<void> close() async {
    // 接続クリーンアップ用プレースホルダー
  }
}
