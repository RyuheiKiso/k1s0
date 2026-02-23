// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-file-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0FileClient", targets: ["K1s0FileClient"]),
    ],
    targets: [
        .target(
            name: "K1s0FileClient",
            path: "Sources/K1s0FileClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0FileClientTests",
            dependencies: ["K1s0FileClient"],
            path: "Tests/K1s0FileClientTests"
        ),
    ]
)
