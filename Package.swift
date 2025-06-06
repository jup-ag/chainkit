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
            targets: ["ChainKit"]
        )
    ],
    targets: [
        .target(
            name: "ChainKit",
            path: "Sources/ChainKit"
        )
    ]
)
