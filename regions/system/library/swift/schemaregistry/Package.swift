// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-schemaregistry",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0SchemaRegistry", targets: ["K1s0SchemaRegistry"]),
    ],
    targets: [
        .target(
            name: "K1s0SchemaRegistry",
            path: "Sources/K1s0SchemaRegistry",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SchemaRegistryTests",
            dependencies: ["K1s0SchemaRegistry"],
            path: "Tests/K1s0SchemaRegistryTests"
        ),
    ]
)
