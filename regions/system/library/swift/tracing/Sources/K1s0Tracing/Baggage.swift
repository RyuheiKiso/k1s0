public struct Baggage: Sendable {
    private var entries: [String: String] = [:]

    public init() {}

    public mutating func set(_ key: String, _ value: String) {
        entries[key] = value
    }

    public func get(_ key: String) -> String? {
        entries[key]
    }

    public func toHeader() -> String {
        entries.map { "\($0.key)=\($0.value)" }.joined(separator: ",")
    }

    public static func fromHeader(_ s: String) -> Baggage {
        var baggage = Baggage()
        guard !s.isEmpty else { return baggage }
        for item in s.split(separator: ",") {
            let parts = item.split(separator: "=", maxSplits: 1)
            if parts.count == 2 {
                let key = parts[0].trimmingCharacters(in: .whitespaces)
                let value = parts[1].trimmingCharacters(in: .whitespaces)
                baggage.set(key, value)
            }
        }
        return baggage
    }

    public var isEmpty: Bool {
        entries.isEmpty
    }
}
