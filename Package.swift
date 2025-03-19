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
            url: "https://github.com/jup-ag/chainkit/releases/download/1.2.12/ChainKitFFI-1.2.12.zip",
            checksum: "61337509e61dac6caa891848527408958883633fe5981188233d8a215410200e"
        )
    ]
)
