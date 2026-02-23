// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-tenant-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0TenantClient", targets: ["K1s0TenantClient"]),
    ],
    targets: [
        .target(
            name: "K1s0TenantClient",
            path: "Sources/K1s0TenantClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0TenantClientTests",
            dependencies: ["K1s0TenantClient"],
            path: "Tests/K1s0TenantClientTests"
        ),
    ]
)
