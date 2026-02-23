// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-outbox",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Outbox", targets: ["K1s0Outbox"]),
    ],
    targets: [
        .target(
            name: "K1s0Outbox",
            path: "Sources/K1s0Outbox",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0OutboxTests",
            dependencies: ["K1s0Outbox"],
            path: "Tests/K1s0OutboxTests"
        ),
    ]
)
