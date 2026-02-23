import Foundation

public struct GraphQlQuery: Sendable {
    public let query: String
    public let variables: [String: any Sendable]?
    public let operationName: String?

    public init(query: String, variables: [String: any Sendable]? = nil, operationName: String? = nil) {
        self.query = query
        self.variables = variables
        self.operationName = operationName
    }
}

public struct GraphQlError: Sendable {
    public let message: String
    public let locations: [ErrorLocation]?
    public let path: [String]?

    public init(message: String, locations: [ErrorLocation]? = nil, path: [String]? = nil) {
        self.message = message
        self.locations = locations
        self.path = path
    }
}

public struct ErrorLocation: Sendable {
    public let line: Int
    public let column: Int

    public init(line: Int, column: Int) {
        self.line = line
        self.column = column
    }
}

public struct GraphQlResponse<T: Sendable>: Sendable {
    public let data: T?
    public let errors: [GraphQlError]?

    public var hasErrors: Bool { !(errors?.isEmpty ?? true) }

    public init(data: T? = nil, errors: [GraphQlError]? = nil) {
        self.data = data
        self.errors = errors
    }
}

public enum GraphQlClientError: Error, Sendable {
    case operationNotFound(name: String)
    case typeMismatch
    case unknownOperation
}
