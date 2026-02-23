// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-kafka",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Kafka", targets: ["K1s0Kafka"]),
    ],
    targets: [
        .target(
            name: "K1s0Kafka",
            path: "Sources/K1s0Kafka",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0KafkaTests",
            dependencies: ["K1s0Kafka"],
            path: "Tests/K1s0KafkaTests"
        ),
    ]
)
