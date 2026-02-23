// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-tracing",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Tracing", targets: ["K1s0Tracing"]),
    ],
    targets: [
        .target(
            name: "K1s0Tracing",
            path: "Sources/K1s0Tracing",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TracingTests",
            dependencies: ["K1s0Tracing"],
            path: "Tests/K1s0TracingTests"
        ),
    ]
)
