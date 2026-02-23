// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-notification-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0NotificationClient", targets: ["K1s0NotificationClient"]),
    ],
    targets: [
        .target(
            name: "K1s0NotificationClient",
            path: "Sources/K1s0NotificationClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0NotificationClientTests",
            dependencies: ["K1s0NotificationClient"],
            path: "Tests/K1s0NotificationClientTests"
        ),
    ]
)
