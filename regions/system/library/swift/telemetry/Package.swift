// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-telemetry",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Telemetry", targets: ["K1s0Telemetry"]),
    ],
    targets: [
        .target(
            name: "K1s0Telemetry",
            path: "Sources/K1s0Telemetry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TelemetryTests",
            dependencies: ["K1s0Telemetry"],
            path: "Tests/K1s0TelemetryTests"
        ),
    ]
)
