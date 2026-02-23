// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-circuit-breaker",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0CircuitBreaker", targets: ["K1s0CircuitBreaker"]),
    ],
    targets: [
        .target(
            name: "K1s0CircuitBreaker",
            path: "Sources/K1s0CircuitBreaker",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CircuitBreakerTests",
            dependencies: ["K1s0CircuitBreaker"],
            path: "Tests/K1s0CircuitBreakerTests"
        ),
    ]
)
