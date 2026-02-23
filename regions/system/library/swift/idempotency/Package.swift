// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-idempotency",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Idempotency", targets: ["K1s0Idempotency"]),
    ],
    targets: [
        .target(
            name: "K1s0Idempotency",
            path: "Sources/K1s0Idempotency",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0IdempotencyTests",
            dependencies: ["K1s0Idempotency"],
            path: "Tests/K1s0IdempotencyTests"
        ),
    ]
)
