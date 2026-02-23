// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-cache",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Cache", targets: ["K1s0Cache"]),
    ],
    targets: [
        .target(
            name: "K1s0Cache",
            path: "Sources/K1s0Cache",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0CacheTests",
            dependencies: ["K1s0Cache"],
            path: "Tests/K1s0CacheTests"
        ),
    ]
)
