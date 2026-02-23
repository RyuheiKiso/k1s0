// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-health",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Health", targets: ["K1s0Health"]),
    ],
    targets: [
        .target(
            name: "K1s0Health",
            path: "Sources/K1s0Health",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0HealthTests",
            dependencies: ["K1s0Health"],
            path: "Tests/K1s0HealthTests"
        ),
    ]
)
