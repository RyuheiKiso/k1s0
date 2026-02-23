// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-messaging",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Messaging", targets: ["K1s0Messaging"]),
    ],
    targets: [
        .target(
            name: "K1s0Messaging",
            path: "Sources/K1s0Messaging",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0MessagingTests",
            dependencies: ["K1s0Messaging"],
            path: "Tests/K1s0MessagingTests"
        ),
    ]
)
