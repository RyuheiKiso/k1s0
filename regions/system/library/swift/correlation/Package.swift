// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-correlation",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Correlation", targets: ["K1s0Correlation"]),
    ],
    targets: [
        .target(
            name: "K1s0Correlation",
            path: "Sources/K1s0Correlation",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CorrelationTests",
            dependencies: ["K1s0Correlation"],
            path: "Tests/K1s0CorrelationTests"
        ),
    ]
)
