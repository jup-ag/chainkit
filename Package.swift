// swift-tools-version:5.9

import PackageDescription
import Foundation

// MARK: - Package Definition
let package = Package(
    name: "ChainKit",
    platforms: [
        .iOS(.v13)
    ],
    products: [
        .library(
            name: "ChainKit",
            targets: ["ChainKit"]),
    ],
    targets: [
        .target(
            name: "ChainKit",
            dependencies: ["ChainKitFFI"],
            path: "platforms/ios/ChainKit/Sources"
        ),
        .binaryTarget(
            name: "ChainKitFFI",
            url: "https://github.com/chainkit/chainkit/releases/download/v1.2.10/ChainKitFFI-1.2.10.zip",
            checksum: "9ce596f39bcb2e8b93724e44ab0ea67fa5e03b8b14e6773e6a1045934358860f"
        )
    ]
)
