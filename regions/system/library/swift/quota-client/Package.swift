// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-quota-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0QuotaClient", targets: ["K1s0QuotaClient"]),
    ],
    targets: [
        .target(
            name: "K1s0QuotaClient",
            path: "Sources/K1s0QuotaClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0QuotaClientTests",
            dependencies: ["K1s0QuotaClient"],
            path: "Tests/K1s0QuotaClientTests"
        ),
    ]
)
