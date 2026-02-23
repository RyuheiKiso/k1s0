// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-saga",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Saga", targets: ["K1s0Saga"]),
    ],
    targets: [
        .target(
            name: "K1s0Saga",
            path: "Sources/K1s0Saga",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SagaTests",
            dependencies: ["K1s0Saga"],
            path: "Tests/K1s0SagaTests"
        ),
    ]
)
