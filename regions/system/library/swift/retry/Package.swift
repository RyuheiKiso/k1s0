// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-retry",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Retry", targets: ["K1s0Retry"]),
    ],
    targets: [
        .target(
            name: "K1s0Retry",
            path: "Sources/K1s0Retry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0RetryTests",
            dependencies: ["K1s0Retry"],
            path: "Tests/K1s0RetryTests"
        ),
    ]
)
