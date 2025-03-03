// swift-tools-version:5.9

import PackageDescription
import Foundation

let frameworkBasePath = "platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"

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
            path: frameworkBasePath
        )
    ]
)

// MARK: - Framework Extraction

print("📦 Running XCFramework extraction script...")

// Determine the absolute path to the script
let currentDirectory = FileManager.default.currentDirectoryPath
let scriptPath = "\(currentDirectory)/scripts/extract_frameworks.sh"

// Check if the script exists
if FileManager.default.fileExists(atPath: scriptPath) {
    print("📄 Found extraction script at: \(scriptPath)")
    
    // Create a process to run the script
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/bin/bash")
    process.arguments = [scriptPath]
    
    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = pipe
    
    do {
        try process.run()
        process.waitUntilExit()
        
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        if let output = String(data: data, encoding: .utf8) {
            print("📋 Script output: \(output)")
        }
        
        if process.terminationStatus == 0 {
            print("✅ Framework extraction completed successfully")
        } else {
            print("⚠️ Extraction script exited with status: \(process.terminationStatus)")
        }
    } catch {
        print("⚠️ Error running extraction script: \(error.localizedDescription)")
    }
} else {
    print("⚠️ Extraction script not found at: \(scriptPath)")
}
