// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-audit-client",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0AuditClient", targets: ["K1s0AuditClient"]),
    ],
    targets: [
        .target(
            name: "K1s0AuditClient",
            path: "Sources/K1s0AuditClient",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0AuditClientTests",
            dependencies: ["K1s0AuditClient"],
            path: "Tests/K1s0AuditClientTests"
        ),
    ]
)
