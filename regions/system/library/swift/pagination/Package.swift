// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-pagination",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Pagination", targets: ["K1s0Pagination"]),
    ],
    targets: [
        .target(
            name: "K1s0Pagination",
            path: "Sources/K1s0Pagination",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0PaginationTests",
            dependencies: ["K1s0Pagination"],
            path: "Tests/K1s0PaginationTests"
        ),
    ]
)
