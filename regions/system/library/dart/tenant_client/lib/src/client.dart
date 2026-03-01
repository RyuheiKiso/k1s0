import 'dart:convert';

import 'package:http/http.dart' as http;

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

/// HTTP REST 経由で tenant-server に接続するクライアント。
/// [config] の [TenantClientConfig.serverUrl] には "http://host:port" 形式のベース URL を指定する
/// （例: "http://tenant-server:8080"）。
class HttpTenantClient implements TenantClient {
  final TenantClientConfig config;
  final http.Client _http;

  HttpTenantClient(this.config, {http.Client? httpClient})
      : _http = httpClient ?? http.Client();

  String _url(String path) => '${config.serverUrl}$path';

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
      };

  // ---------- JSON helpers ----------

  Tenant _tenantFromJson(Map<String, dynamic> json) {
    return Tenant(
      id: json['id'] as String,
      name: (json['display_name'] ?? json['name']) as String,
      status: _tenantStatusFromString(json['status'] as String),
      plan: json['plan'] as String,
      settings: Map<String, String>.from(json['settings'] as Map? ?? {}),
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  TenantStatus _tenantStatusFromString(String s) {
    switch (s.toLowerCase()) {
      case 'active':
        return TenantStatus.active;
      case 'suspended':
        return TenantStatus.suspended;
      case 'deleted':
        return TenantStatus.deleted;
      default:
        return TenantStatus.active;
    }
  }

  TenantMember _memberFromJson(Map<String, dynamic> json) {
    return TenantMember(
      userId: json['user_id'] as String,
      role: json['role'] as String,
      joinedAt: DateTime.parse(json['joined_at'] as String),
    );
  }

  ProvisioningStatus _provisioningStatusFromString(String s) {
    switch (s.toLowerCase()) {
      case 'pending':
        return ProvisioningStatus.pending;
      case 'in_progress':
        return ProvisioningStatus.inProgress;
      case 'completed':
        return ProvisioningStatus.completed;
      default:
        return ProvisioningStatus.failed;
    }
  }

  // ---------- error helpers ----------

  Never _throwForStatus(http.Response response, {String? context}) {
    final label = context != null ? ' ($context)' : '';
    if (response.statusCode == 404) {
      throw TenantError(
          'Not found$label', TenantErrorCode.notFound);
    }
    throw TenantError(
        'Server error ${response.statusCode}$label', TenantErrorCode.serverError);
  }

  // ---------- TenantClient implementation ----------

  @override
  Future<Tenant> getTenant(String tenantId) async {
    final response = await _http.get(
      Uri.parse(_url('/api/v1/tenants/$tenantId')),
      headers: _headers,
    );
    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return _tenantFromJson(json['tenant'] as Map<String, dynamic>);
    }
    _throwForStatus(response, context: tenantId);
  }

  @override
  Future<List<Tenant>> listTenants(TenantFilter filter) async {
    final params = <String, String>{};
    if (filter.status != null) {
      params['status'] = filter.status!.name;
    }
    if (filter.plan != null) {
      params['plan'] = filter.plan!;
    }
    final uri = Uri.parse(_url('/api/v1/tenants'))
        .replace(queryParameters: params.isEmpty ? null : params);
    final response = await _http.get(uri, headers: _headers);
    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final tenants = (json['tenants'] as List)
          .map((t) => _tenantFromJson(t as Map<String, dynamic>))
          .toList();
      return tenants;
    }
    _throwForStatus(response, context: 'listTenants');
  }

  @override
  Future<bool> isActive(String tenantId) async {
    final tenant = await getTenant(tenantId);
    return tenant.status == TenantStatus.active;
  }

  @override
  Future<TenantSettings> getSettings(String tenantId) async {
    final response = await _http.get(
      Uri.parse(_url('/api/v1/tenants/$tenantId/settings')),
      headers: _headers,
    );
    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final rawSettings = json['settings'] as Map? ?? {};
      final settings = Map<String, String>.from(rawSettings);
      return TenantSettings(settings);
    }
    _throwForStatus(response, context: tenantId);
  }

  @override
  Future<Tenant> createTenant(CreateTenantRequest req) async {
    final body = <String, dynamic>{
      'name': req.name,
      'plan': req.plan,
      if (req.adminUserId != null) 'admin_user_id': req.adminUserId,
    };
    final response = await _http.post(
      Uri.parse(_url('/api/v1/tenants')),
      headers: _headers,
      body: jsonEncode(body),
    );
    if (response.statusCode == 200 || response.statusCode == 201) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return _tenantFromJson(json['tenant'] as Map<String, dynamic>);
    }
    _throwForStatus(response, context: 'createTenant');
  }

  @override
  Future<TenantMember> addMember(
      String tenantId, String userId, String role) async {
    final body = <String, dynamic>{
      'user_id': userId,
      'role': role,
    };
    final response = await _http.post(
      Uri.parse(_url('/api/v1/tenants/$tenantId/members')),
      headers: _headers,
      body: jsonEncode(body),
    );
    if (response.statusCode == 200 || response.statusCode == 201) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return _memberFromJson(json['member'] as Map<String, dynamic>);
    }
    _throwForStatus(response, context: 'addMember($tenantId, $userId)');
  }

  @override
  Future<void> removeMember(String tenantId, String userId) async {
    final response = await _http.delete(
      Uri.parse(_url('/api/v1/tenants/$tenantId/members/$userId')),
      headers: _headers,
    );
    if (response.statusCode == 200 ||
        response.statusCode == 204 ||
        response.statusCode == 404) {
      // 204 No Content および 404（既に存在しない）はどちらも成功扱い
      return;
    }
    _throwForStatus(response, context: 'removeMember($tenantId, $userId)');
  }

  @override
  Future<List<TenantMember>> listMembers(String tenantId) async {
    final response = await _http.get(
      Uri.parse(_url('/api/v1/tenants/$tenantId/members')),
      headers: _headers,
    );
    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final members = (json['members'] as List)
          .map((m) => _memberFromJson(m as Map<String, dynamic>))
          .toList();
      return members;
    }
    _throwForStatus(response, context: 'listMembers($tenantId)');
  }

  @override
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId) async {
    final response = await _http.get(
      Uri.parse(_url('/api/v1/tenants/$tenantId/provisioning-status')),
      headers: _headers,
    );
    if (response.statusCode == 404) {
      // エンドポイントが存在しない場合はプロビジョニング完了とみなす
      return ProvisioningStatus.completed;
    }
    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return _provisioningStatusFromString(json['status'] as String);
    }
    _throwForStatus(response, context: 'getProvisioningStatus($tenantId)');
  }

  /// HTTP クライアントを閉じる。アプリ終了時などに呼び出すこと。
  Future<void> close() async {
    _http.close();
  }
}
