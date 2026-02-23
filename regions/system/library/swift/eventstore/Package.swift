// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-eventstore",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0EventStore", targets: ["K1s0EventStore"]),
    ],
    targets: [
        .target(
            name: "K1s0EventStore",
            path: "Sources/K1s0EventStore",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EventStoreTests",
            dependencies: ["K1s0EventStore"],
            path: "Tests/K1s0EventStoreTests"
        ),
    ]
)
