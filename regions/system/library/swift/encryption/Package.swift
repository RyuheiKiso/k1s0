// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-encryption",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0Encryption", targets: ["K1s0Encryption"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-crypto.git", from: "3.0.0"),
    ],
    targets: [
        .target(
            name: "K1s0Encryption",
            dependencies: [
                .product(name: "Crypto", package: "swift-crypto"),
            ],
            path: "Sources/K1s0Encryption",
            swiftSettings: [.swiftLanguageMode(.v6)]
        ),
        .testTarget(
            name: "K1s0EncryptionTests",
            dependencies: [
                "K1s0Encryption",
                .product(name: "Crypto", package: "swift-crypto"),
            ],
            path: "Tests/K1s0EncryptionTests"
        ),
    ]
)
