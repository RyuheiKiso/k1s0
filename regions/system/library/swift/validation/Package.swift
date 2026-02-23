// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-validation",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Validation", targets: ["K1s0Validation"]),
    ],
    targets: [
        .target(
            name: "K1s0Validation",
            path: "Sources/K1s0Validation",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ValidationTests",
            dependencies: ["K1s0Validation"],
            path: "Tests/K1s0ValidationTests"
        ),
    ]
)
