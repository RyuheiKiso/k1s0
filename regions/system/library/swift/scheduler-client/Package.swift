// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-scheduler-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0SchedulerClient", targets: ["K1s0SchedulerClient"]),
    ],
    targets: [
        .target(
            name: "K1s0SchedulerClient",
            path: "Sources/K1s0SchedulerClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0SchedulerClientTests",
            dependencies: ["K1s0SchedulerClient"],
            path: "Tests/K1s0SchedulerClientTests"
        ),
    ]
)
