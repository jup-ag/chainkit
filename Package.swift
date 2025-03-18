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
            url: "https://github.com/jup-ag/chainkit/releases/download/1.2.6/ChainKitFFI-1.2.6.zip",
            checksum: "ffaf3fa3c147c794065fbc8e6c3a9837d3ff04be017c34a96aa1da8389adc00f"
        )
    ]
)
