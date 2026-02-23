// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-websocket",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0WebSocket", targets: ["K1s0WebSocket"]),
    ],
    targets: [
        .target(
            name: "K1s0WebSocket",
            path: "Sources/K1s0WebSocket",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0WebSocketTests",
            dependencies: ["K1s0WebSocket"],
            path: "Tests/K1s0WebSocketTests"
        ),
    ]
)
