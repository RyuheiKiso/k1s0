// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-test-helper",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0TestHelper", targets: ["K1s0TestHelper"]),
    ],
    targets: [
        .target(
            name: "K1s0TestHelper",
            path: "Sources/K1s0TestHelper",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TestHelperTests",
            dependencies: ["K1s0TestHelper"],
            path: "Tests/K1s0TestHelperTests"
        ),
    ]
)
