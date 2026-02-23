// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-session-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0SessionClient", targets: ["K1s0SessionClient"]),
    ],
    targets: [
        .target(
            name: "K1s0SessionClient",
            path: "Sources/K1s0SessionClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SessionClientTests",
            dependencies: ["K1s0SessionClient"],
            path: "Tests/K1s0SessionClientTests"
        ),
    ]
)
