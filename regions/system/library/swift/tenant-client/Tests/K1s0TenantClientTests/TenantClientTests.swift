import Testing
@testable import K1s0TenantClient
import Foundation

func makeTenant(id: String, status: TenantStatus = .active, plan: String = "basic") -> Tenant {
    Tenant(id: id, name: "Tenant \(id)", status: status, plan: plan, settings: ["max_users": "100"], createdAt: Date())
}

@Suite("TenantClient Tests")
struct TenantClientTests {
    @Test("テナントを取得できること")
    func testGetTenant() async throws {
        let client = InMemoryTenantClient([makeTenant(id: "T-001")])
        let tenant = try await client.getTenant(tenantId: "T-001")
        #expect(tenant.id == "T-001")
        #expect(tenant.status == .active)
    }

    @Test("存在しないテナントでエラーを返すこと")
    func testGetTenantNotFound() async {
        let client = InMemoryTenantClient()
        do {
            _ = try await client.getTenant(tenantId: "T-999")
            #expect(Bool(false), "Expected TenantError.notFound")
        } catch {
            #expect(error is TenantError)
        }
    }

    @Test("ステータスでフィルターできること")
    func testListTenantsFilterByStatus() async throws {
        let client = InMemoryTenantClient([
            makeTenant(id: "T-001", status: .active),
            makeTenant(id: "T-002", status: .suspended),
            makeTenant(id: "T-003", status: .active),
        ])
        let tenants = try await client.listTenants(filter: TenantFilter(status: .active))
        #expect(tenants.count == 2)
    }

    @Test("プランでフィルターできること")
    func testListTenantsFilterByPlan() async throws {
        let client = InMemoryTenantClient([
            makeTenant(id: "T-001", plan: "enterprise"),
            makeTenant(id: "T-002", plan: "basic"),
        ])
        let tenants = try await client.listTenants(filter: TenantFilter(plan: "enterprise"))
        #expect(tenants.count == 1)
        #expect(tenants[0].id == "T-001")
    }

    @Test("アクティブテナントをチェックできること")
    func testIsActiveTrue() async throws {
        let client = InMemoryTenantClient([makeTenant(id: "T-001", status: .active)])
        let active = try await client.isActive(tenantId: "T-001")
        #expect(active == true)
    }

    @Test("非アクティブテナントを検出できること")
    func testIsActiveFalse() async throws {
        let client = InMemoryTenantClient([makeTenant(id: "T-001", status: .suspended)])
        let active = try await client.isActive(tenantId: "T-001")
        #expect(active == false)
    }

    @Test("テナント設定を取得できること")
    func testGetSettings() async throws {
        let client = InMemoryTenantClient([makeTenant(id: "T-001")])
        let settings = try await client.getSettings(tenantId: "T-001")
        #expect(settings.get("max_users") == "100")
        #expect(settings.get("nonexistent") == nil)
    }

    @Test("addTenantでテナントを追加できること")
    func testAddTenant() async throws {
        let client = InMemoryTenantClient()
        await client.addTenant(makeTenant(id: "T-001"))
        let tenant = try await client.getTenant(tenantId: "T-001")
        #expect(tenant.id == "T-001")
    }

    @Test("TenantStatusの各バリアント")
    func testTenantStatusVariants() {
        #expect(TenantStatus.active.rawValue == "active")
        #expect(TenantStatus.suspended.rawValue == "suspended")
        #expect(TenantStatus.deleted.rawValue == "deleted")
    }

    @Test("TenantClientConfigのデフォルト値")
    func testConfigDefaults() {
        let config = TenantClientConfig(serverUrl: "http://localhost:8080")
        #expect(config.serverUrl == "http://localhost:8080")
        #expect(config.cacheMaxCapacity == 1000)
    }
}
