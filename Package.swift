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
            url: "https://github.com/jup-ag/chainkit/releases/download/1.2.19/ChainKitFFI-1.2.19.zip",
            checksum: "6a0bbf5be572da26bddcb6a0cc2848330542b19ea97eb64fb23ecb798c212675"
        )
    ]
)
