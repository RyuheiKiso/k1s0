// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-dlq",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Dlq", targets: ["K1s0Dlq"]),
    ],
    targets: [
        .target(
            name: "K1s0Dlq",
            path: "Sources/K1s0Dlq",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0DlqTests",
            dependencies: ["K1s0Dlq"],
            path: "Tests/K1s0DlqTests"
        ),
    ]
)
