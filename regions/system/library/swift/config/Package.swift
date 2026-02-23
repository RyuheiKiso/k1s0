// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-config",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Config", targets: ["K1s0Config"]),
    ],
    targets: [
        .target(
            name: "K1s0Config",
            path: "Sources/K1s0Config",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ConfigTests",
            dependencies: ["K1s0Config"],
            path: "Tests/K1s0ConfigTests"
        ),
    ]
)
