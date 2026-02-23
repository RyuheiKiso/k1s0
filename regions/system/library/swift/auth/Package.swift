// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-auth",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Auth", targets: ["K1s0Auth"]),
    ],
    targets: [
        .target(
            name: "K1s0Auth",
            path: "Sources/K1s0Auth",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0AuthTests",
            dependencies: ["K1s0Auth"],
            path: "Tests/K1s0AuthTests"
        ),
    ]
)
