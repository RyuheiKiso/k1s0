// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-featureflag",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0FeatureFlag", targets: ["K1s0FeatureFlag"]),
    ],
    targets: [
        .target(
            name: "K1s0FeatureFlag",
            path: "Sources/K1s0FeatureFlag",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0FeatureFlagTests",
            dependencies: ["K1s0FeatureFlag"],
            path: "Tests/K1s0FeatureFlagTests"
        ),
    ]
)
