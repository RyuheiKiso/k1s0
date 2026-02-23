// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-webhook-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0WebhookClient", targets: ["K1s0WebhookClient"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-crypto.git", from: "3.0.0"),
    ],
    targets: [
        .target(
            name: "K1s0WebhookClient",
            dependencies: [
                .product(name: "Crypto", package: "swift-crypto"),
            ],
            path: "Sources/K1s0WebhookClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0WebhookClientTests",
            dependencies: ["K1s0WebhookClient"],
            path: "Tests/K1s0WebhookClientTests"
        ),
    ]
)
