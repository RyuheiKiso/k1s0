// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-distributed-lock",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0DistributedLock", targets: ["K1s0DistributedLock"]),
    ],
    targets: [
        .target(
            name: "K1s0DistributedLock",
            path: "Sources/K1s0DistributedLock",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0DistributedLockTests",
            dependencies: ["K1s0DistributedLock"],
            path: "Tests/K1s0DistributedLockTests"
        ),
    ]
)
