// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-search-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0SearchClient", targets: ["K1s0SearchClient"]),
    ],
    targets: [
        .target(
            name: "K1s0SearchClient",
            path: "Sources/K1s0SearchClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SearchClientTests",
            dependencies: ["K1s0SearchClient"],
            path: "Tests/K1s0SearchClientTests"
        ),
    ]
)
