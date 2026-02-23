// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-serviceauth",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0ServiceAuth", targets: ["K1s0ServiceAuth"]),
    ],
    targets: [
        .target(
            name: "K1s0ServiceAuth",
            path: "Sources/K1s0ServiceAuth",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ServiceAuthTests",
            dependencies: ["K1s0ServiceAuth"],
            path: "Tests/K1s0ServiceAuthTests"
        ),
    ]
)
