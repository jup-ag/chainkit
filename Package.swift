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
            url: "https://github.com/jup-ag/chainkit/releases/download/1.2.7/ChainKitFFI-1.2.7.zip",
            checksum: "ceef125df61d93720d169d55e7526993117ab5d8f0ce28563533b05465d98a13"
        )
    ]
)
