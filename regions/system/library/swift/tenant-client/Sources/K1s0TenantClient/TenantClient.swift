import Foundation

public enum TenantStatus: String, Sendable {
    case active
    case suspended
    case deleted
}

public struct Tenant: Sendable {
    public let id: String
    public let name: String
    public let status: TenantStatus
    public let plan: String
    public let settings: [String: String]
    public let createdAt: Date

    public init(id: String, name: String, status: TenantStatus, plan: String, settings: [String: String], createdAt: Date) {
        self.id = id
        self.name = name
        self.status = status
        self.plan = plan
        self.settings = settings
        self.createdAt = createdAt
    }
}

public struct TenantFilter: Sendable {
    public let status: TenantStatus?
    public let plan: String?

    public init(status: TenantStatus? = nil, plan: String? = nil) {
        self.status = status
        self.plan = plan
    }
}

public struct TenantSettings: Sendable {
    public let values: [String: String]

    public init(values: [String: String]) {
        self.values = values
    }

    public func get(_ key: String) -> String? {
        values[key]
    }
}

public struct TenantClientConfig: Sendable {
    public let serverUrl: String
    public let cacheTtl: Duration
    public let cacheMaxCapacity: Int

    public init(serverUrl: String, cacheTtl: Duration = .seconds(300), cacheMaxCapacity: Int = 1000) {
        self.serverUrl = serverUrl
        self.cacheTtl = cacheTtl
        self.cacheMaxCapacity = cacheMaxCapacity
    }
}

public enum TenantError: Error, Sendable {
    case notFound(tenantId: String)
    case suspended(tenantId: String)
    case serverError(message: String)
    case timeout
}

public protocol TenantClient: Sendable {
    func getTenant(tenantId: String) async throws -> Tenant
    func listTenants(filter: TenantFilter) async throws -> [Tenant]
    func isActive(tenantId: String) async throws -> Bool
    func getSettings(tenantId: String) async throws -> TenantSettings
}

public actor InMemoryTenantClient: TenantClient {
    private var tenants: [Tenant] = []

    public init(_ tenants: [Tenant] = []) {
        self.tenants = tenants
    }

    public func addTenant(_ tenant: Tenant) {
        tenants.append(tenant)
    }

    public func allTenants() -> [Tenant] {
        tenants
    }

    public func getTenant(tenantId: String) async throws -> Tenant {
        guard let tenant = tenants.first(where: { $0.id == tenantId }) else {
            throw TenantError.notFound(tenantId: tenantId)
        }
        return tenant
    }

    public func listTenants(filter: TenantFilter) async throws -> [Tenant] {
        tenants.filter { t in
            if let status = filter.status, t.status != status { return false }
            if let plan = filter.plan, t.plan != plan { return false }
            return true
        }
    }

    public func isActive(tenantId: String) async throws -> Bool {
        let tenant = try await getTenant(tenantId: tenantId)
        return tenant.status == .active
    }

    public func getSettings(tenantId: String) async throws -> TenantSettings {
        let tenant = try await getTenant(tenantId: tenantId)
        return TenantSettings(values: tenant.settings)
    }
}
