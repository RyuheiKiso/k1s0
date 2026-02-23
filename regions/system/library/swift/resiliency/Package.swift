// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-resiliency",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Resiliency", targets: ["K1s0Resiliency"]),
    ],
    targets: [
        .target(
            name: "K1s0Resiliency",
            path: "Sources/K1s0Resiliency",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0ResiliencyTests",
            dependencies: ["K1s0Resiliency"],
            path: "Tests/K1s0ResiliencyTests"
        ),
    ]
)
