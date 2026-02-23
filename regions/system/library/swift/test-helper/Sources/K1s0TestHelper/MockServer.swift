import Foundation

/// モックルート定義。
public struct MockRoute: Sendable {
    public let method: String
    public let path: String
    public let status: Int
    public let body: String

    public init(method: String, path: String, status: Int, body: String) {
        self.method = method
        self.path = path
        self.status = status
        self.body = body
    }
}

/// モックサーバー (インメモリ)。
public final class MockServer: Sendable {
    private let routes: [MockRoute]
    private let _requests: LockedArray<(method: String, path: String)>

    public init(routes: [MockRoute]) {
        self.routes = routes
        self._requests = LockedArray()
    }

    /// 登録済みルートからレスポンスを取得する。
    public func handle(method: String, path: String) -> (status: Int, body: String)? {
        _requests.append((method: method, path: path))
        if let route = routes.first(where: { $0.method == method && $0.path == path }) {
            return (status: route.status, body: route.body)
        }
        return nil
    }

    /// 記録されたリクエスト数を返す。
    public var requestCount: Int { _requests.count }
}

/// スレッドセーフな配列。
final class LockedArray<T>: @unchecked Sendable {
    private var array: [T] = []
    private let lock = NSLock()

    var count: Int {
        lock.lock()
        defer { lock.unlock() }
        return array.count
    }

    func append(_ element: T) {
        lock.lock()
        defer { lock.unlock() }
        array.append(element)
    }
}

/// モックサーバービルダー。
public struct MockServerBuilder: Sendable {
    private let serverType: String
    private var routes: [MockRoute]

    private init(serverType: String) {
        self.serverType = serverType
        self.routes = []
    }

    public static func notificationServer() -> MockServerBuilder {
        MockServerBuilder(serverType: "notification")
    }

    public static func ratelimitServer() -> MockServerBuilder {
        MockServerBuilder(serverType: "ratelimit")
    }

    public static func tenantServer() -> MockServerBuilder {
        MockServerBuilder(serverType: "tenant")
    }

    public mutating func withHealthOk() -> MockServerBuilder {
        routes.append(MockRoute(method: "GET", path: "/health", status: 200, body: "{\"status\":\"ok\"}"))
        return self
    }

    public mutating func withSuccessResponse(path: String, body: String) -> MockServerBuilder {
        routes.append(MockRoute(method: "POST", path: path, status: 200, body: body))
        return self
    }

    public mutating func withErrorResponse(path: String, status: Int) -> MockServerBuilder {
        routes.append(MockRoute(method: "POST", path: path, status: status, body: "{\"error\":\"mock error\"}"))
        return self
    }

    public func build() -> MockServer {
        MockServer(routes: routes)
    }
}
