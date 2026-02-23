// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-event-bus",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0EventBus", targets: ["K1s0EventBus"]),
    ],
    targets: [
        .target(
            name: "K1s0EventBus",
            path: "Sources/K1s0EventBus",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EventBusTests",
            dependencies: ["K1s0EventBus"],
            path: "Tests/K1s0EventBusTests"
        ),
    ]
)
