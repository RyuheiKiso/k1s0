// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-vault-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0VaultClient", targets: ["K1s0VaultClient"]),
    ],
    targets: [
        .target(
            name: "K1s0VaultClient",
            path: "Sources/K1s0VaultClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0VaultClientTests",
            dependencies: ["K1s0VaultClient"],
            path: "Tests/K1s0VaultClientTests"
        ),
    ]
)
