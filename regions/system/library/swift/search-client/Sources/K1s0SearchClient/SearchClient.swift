import Foundation

public struct Filter: Sendable {
    public let field: String
    public let `operator`: String
    public let value: any Sendable
    public let valueTo: (any Sendable)?

    public init(field: String, operator: String, value: any Sendable, valueTo: (any Sendable)? = nil) {
        self.field = field
        self.operator = `operator`
        self.value = value
        self.valueTo = valueTo
    }

    public static func eq(_ field: String, _ value: any Sendable) -> Filter {
        Filter(field: field, operator: "eq", value: value)
    }

    public static func lt(_ field: String, _ value: any Sendable) -> Filter {
        Filter(field: field, operator: "lt", value: value)
    }

    public static func gt(_ field: String, _ value: any Sendable) -> Filter {
        Filter(field: field, operator: "gt", value: value)
    }

    public static func range(_ field: String, _ from: any Sendable, _ to: any Sendable) -> Filter {
        Filter(field: field, operator: "range", value: from, valueTo: to)
    }
}

public struct FacetBucket: Sendable {
    public let value: String
    public let count: UInt64

    public init(value: String, count: UInt64) {
        self.value = value
        self.count = count
    }
}

public struct SearchQuery: Sendable {
    public let query: String
    public let filters: [Filter]
    public let facets: [String]
    public let page: UInt32
    public let size: UInt32

    public init(query: String, filters: [Filter] = [], facets: [String] = [], page: UInt32 = 0, size: UInt32 = 20) {
        self.query = query
        self.filters = filters
        self.facets = facets
        self.page = page
        self.size = size
    }
}

public struct SearchResult<T: Sendable>: Sendable {
    public let hits: [T]
    public let total: UInt64
    public let facets: [String: [FacetBucket]]
    public let tookMs: UInt64

    public init(hits: [T], total: UInt64, facets: [String: [FacetBucket]], tookMs: UInt64) {
        self.hits = hits
        self.total = total
        self.facets = facets
        self.tookMs = tookMs
    }
}

public struct IndexDocument: Sendable {
    public let id: String
    public let fields: [String: any Sendable]

    public init(id: String, fields: [String: any Sendable]) {
        self.id = id
        self.fields = fields
    }
}

public struct IndexResult: Sendable {
    public let id: String
    public let version: Int64

    public init(id: String, version: Int64) {
        self.id = id
        self.version = version
    }
}

public struct BulkFailure: Sendable {
    public let id: String
    public let error: String

    public init(id: String, error: String) {
        self.id = id
        self.error = error
    }
}

public struct BulkResult: Sendable {
    public let successCount: Int
    public let failedCount: Int
    public let failures: [BulkFailure]

    public init(successCount: Int, failedCount: Int, failures: [BulkFailure]) {
        self.successCount = successCount
        self.failedCount = failedCount
        self.failures = failures
    }
}

public struct FieldMapping: Sendable {
    public let fieldType: String
    public let indexed: Bool

    public init(fieldType: String, indexed: Bool = true) {
        self.fieldType = fieldType
        self.indexed = indexed
    }
}

public struct IndexMapping: Sendable {
    public var fields: [String: FieldMapping]

    public init(fields: [String: FieldMapping] = [:]) {
        self.fields = fields
    }

    public func withField(_ name: String, _ fieldType: String) -> IndexMapping {
        var copy = self
        copy.fields[name] = FieldMapping(fieldType: fieldType)
        return copy
    }
}

public enum SearchError: Error, Sendable {
    case indexNotFound(name: String)
    case invalidQuery(reason: String)
    case serverError(message: String)
    case timeout
}

public protocol SearchClientProtocol: Sendable {
    func indexDocument(index: String, doc: IndexDocument) async throws -> IndexResult
    func bulkIndex(index: String, docs: [IndexDocument]) async throws -> BulkResult
    func search(index: String, query: SearchQuery) async throws -> SearchResult<[String: String]>
    func deleteDocument(index: String, id: String) async throws
    func createIndex(name: String, mapping: IndexMapping) async throws
}

public actor InMemorySearchClient: SearchClientProtocol {
    private var indexes: [String: [IndexDocument]] = [:]

    public init() {}

    public func createIndex(name: String, mapping: IndexMapping) async throws {
        indexes[name] = []
    }

    public func indexDocument(index: String, doc: IndexDocument) async throws -> IndexResult {
        guard indexes[index] != nil else {
            throw SearchError.indexNotFound(name: index)
        }
        indexes[index]!.append(doc)
        return IndexResult(id: doc.id, version: Int64(indexes[index]!.count))
    }

    public func bulkIndex(index: String, docs: [IndexDocument]) async throws -> BulkResult {
        guard indexes[index] != nil else {
            throw SearchError.indexNotFound(name: index)
        }
        indexes[index]!.append(contentsOf: docs)
        return BulkResult(successCount: docs.count, failedCount: 0, failures: [])
    }

    public func search(index: String, query: SearchQuery) async throws -> SearchResult<[String: String]> {
        guard let docs = indexes[index] else {
            throw SearchError.indexNotFound(name: index)
        }

        let filtered: [IndexDocument]
        if query.query.isEmpty {
            filtered = docs
        } else {
            filtered = docs.filter { doc in
                doc.fields.values.contains { value in
                    if let str = value as? String {
                        return str.contains(query.query)
                    }
                    return false
                }
            }
        }

        let start = Int(query.page) * Int(query.size)
        let end = min(start + Int(query.size), filtered.count)
        let pageStart = min(start, filtered.count)
        let paged = Array(filtered[pageStart..<end])

        let hits: [[String: String]] = paged.map { doc in
            var hit: [String: String] = ["id": doc.id]
            for (key, value) in doc.fields {
                hit[key] = "\(value)"
            }
            return hit
        }

        var facets: [String: [FacetBucket]] = [:]
        for f in query.facets {
            facets[f] = [FacetBucket(value: "default", count: UInt64(hits.count))]
        }

        return SearchResult(
            hits: hits,
            total: UInt64(filtered.count),
            facets: facets,
            tookMs: 1
        )
    }

    public func deleteDocument(index: String, id: String) async throws {
        indexes[index]?.removeAll { $0.id == id }
    }

    public func documentCount(_ index: String) -> Int {
        indexes[index]?.count ?? 0
    }
}
