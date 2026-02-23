// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-graphql-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0GraphQlClient", targets: ["K1s0GraphQlClient"]),
    ],
    targets: [
        .target(
            name: "K1s0GraphQlClient",
            path: "Sources/K1s0GraphQlClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0GraphQlClientTests",
            dependencies: ["K1s0GraphQlClient"],
            path: "Tests/K1s0GraphQlClientTests"
        ),
    ]
)
