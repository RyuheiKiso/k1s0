import 'package:test/test.dart';

import 'package:k1s0_tenant_client/tenant_client.dart';

Tenant makeTenant(String id,
    {TenantStatus status = TenantStatus.active, String plan = 'basic'}) {
  return Tenant(
    id: id,
    name: 'Tenant $id',
    status: status,
    plan: plan,
    settings: {'max_users': '100'},
    createdAt: DateTime.now(),
  );
}

void main() {
  group('InMemoryTenantClient', () {
    test('テナントを取得できる', () async {
      final client = InMemoryTenantClient([makeTenant('T-001')]);
      final tenant = await client.getTenant('T-001');
      expect(tenant.id, equals('T-001'));
      expect(tenant.status, equals(TenantStatus.active));
    });

    test('存在しないテナントでエラーを返す', () async {
      final client = InMemoryTenantClient();
      expect(
        () async => await client.getTenant('T-999'),
        throwsA(isA<TenantError>()),
      );
    });

    test('ステータスでフィルターできる', () async {
      final client = InMemoryTenantClient([
        makeTenant('T-001', status: TenantStatus.active),
        makeTenant('T-002', status: TenantStatus.suspended),
        makeTenant('T-003', status: TenantStatus.active),
      ]);
      final tenants = await client
          .listTenants(const TenantFilter(status: TenantStatus.active));
      expect(tenants, hasLength(2));
    });

    test('プランでフィルターできる', () async {
      final client = InMemoryTenantClient([
        makeTenant('T-001', plan: 'enterprise'),
        makeTenant('T-002', plan: 'basic'),
      ]);
      final tenants =
          await client.listTenants(const TenantFilter(plan: 'enterprise'));
      expect(tenants, hasLength(1));
      expect(tenants[0].id, equals('T-001'));
    });

    test('アクティブテナントをチェックできる', () async {
      final client = InMemoryTenantClient(
          [makeTenant('T-001', status: TenantStatus.active)]);
      expect(await client.isActive('T-001'), isTrue);
    });

    test('非アクティブテナントを検出できる', () async {
      final client = InMemoryTenantClient(
          [makeTenant('T-001', status: TenantStatus.suspended)]);
      expect(await client.isActive('T-001'), isFalse);
    });

    test('テナント設定を取得できる', () async {
      final client = InMemoryTenantClient([makeTenant('T-001')]);
      final settings = await client.getSettings('T-001');
      expect(settings.get('max_users'), equals('100'));
      expect(settings.get('nonexistent'), isNull);
    });

    test('addTenantでテナントを追加できる', () async {
      final client = InMemoryTenantClient();
      client.addTenant(makeTenant('T-001'));
      final tenant = await client.getTenant('T-001');
      expect(tenant.id, equals('T-001'));
    });

    test('createTenant creates active tenant', () async {
      final client = InMemoryTenantClient();
      final tenant = await client.createTenant(
        const CreateTenantRequest(name: 'Test Corp', plan: 'enterprise'),
      );
      expect(tenant.name, equals('Test Corp'));
      expect(tenant.status, equals(TenantStatus.active));
      expect(tenant.plan, equals('enterprise'));
    });

    test('addMember and listMembers work correctly', () async {
      final client = InMemoryTenantClient();
      final tenant = await client.createTenant(
        const CreateTenantRequest(name: 'T1', plan: 'pro'),
      );

      await client.addMember(tenant.id, 'user-1', 'admin');
      await client.addMember(tenant.id, 'user-2', 'member');

      final members = await client.listMembers(tenant.id);
      expect(members, hasLength(2));

      await client.removeMember(tenant.id, 'user-1');
      final updated = await client.listMembers(tenant.id);
      expect(updated, hasLength(1));
      expect(updated.first.userId, equals('user-2'));
    });

    test('getProvisioningStatus returns pending after creation', () async {
      final client = InMemoryTenantClient();
      final tenant = await client.createTenant(
        const CreateTenantRequest(name: 'T2', plan: 'starter'),
      );
      final status = await client.getProvisioningStatus(tenant.id);
      expect(status, equals(ProvisioningStatus.pending));
    });
  });

  group('TenantSettings', () {
    test('値を取得できる', () {
      const settings = TenantSettings({'key': 'value'});
      expect(settings.get('key'), equals('value'));
      expect(settings.get('missing'), isNull);
    });
  });

  group('TenantError', () {
    test('エラーメッセージとコードを保持する', () {
      const error = TenantError('not found', TenantErrorCode.notFound);
      expect(error.message, equals('not found'));
      expect(error.code, equals(TenantErrorCode.notFound));
    });
  });
}
