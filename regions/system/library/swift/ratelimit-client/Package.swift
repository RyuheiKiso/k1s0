// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-ratelimit-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0RateLimitClient", targets: ["K1s0RateLimitClient"]),
    ],
    targets: [
        .target(
            name: "K1s0RateLimitClient",
            path: "Sources/K1s0RateLimitClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0RateLimitClientTests",
            dependencies: ["K1s0RateLimitClient"],
            path: "Tests/K1s0RateLimitClientTests"
        ),
    ]
)
