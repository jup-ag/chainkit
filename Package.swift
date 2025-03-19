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
            url: "https://github.com/jup-ag/chainkit/releases/download/1.2.17/ChainKitFFI-1.2.17.zip",
            checksum: "943a8eb08eb361c69c2afd3f51519dd133560e8df96cb50e30216bbe212ddbdb"
        )
    ]
)
