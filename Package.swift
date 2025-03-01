// swift-tools-version:5.9
// swiftlint:disable all

import PackageDescription

let package = Package(
    name: "ChainKit",
    products: [
        .library(
            name: "ChainKit",
            targets: ["ChainKit", "ChainKitFFI"]
        )
    ],
    targets: [
        .target(
            name: "ChainKit",
            dependencies: ["ChainKitFFI"],
            path: "platforms/ios/ChainKit/Sources"
        ),
        .binaryTarget(
            name: "ChainKitFFI", 
            path: "platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"
        ),
    ]
)
