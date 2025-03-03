// swift-tools-version:5.9
// swiftlint:disable all

import PackageDescription
import Foundation

// MARK: - Framework Extraction Logic
let frameworkBasePath = "platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"

// Make sure the framework directory exists
try? FileManager.default.createDirectory(atPath: frameworkBasePath, withIntermediateDirectories: true)

// Extract all zip files found in the framework directory
do {
    let fileManager = FileManager.default
    
    // Get contents of framework directory
    let contents = try fileManager.contentsOfDirectory(atPath: frameworkBasePath)
    
    // Find all zip files
    let zipFiles = contents.filter { $0.lowercased().hasSuffix(".zip") }
    
    if zipFiles.isEmpty {
        print("ℹ️ No zip files found in \(frameworkBasePath)")
    } else {
        print("ℹ️ Found \(zipFiles.count) zip file(s) to check")
        
        // Process each zip file
        for zipFileName in zipFiles {
            let zipPath = "\(frameworkBasePath)/\(zipFileName)"
            
            // Determine expected directory name by removing .zip extension
            let expectedDirName = zipFileName.replacingOccurrences(of: ".zip", with: "", options: [.caseInsensitive])
            let targetDirPath = "\(frameworkBasePath)/\(expectedDirName)"
            
            // Get zip file attributes to check modification time
            let zipAttributes = try? fileManager.attributesOfItem(atPath: zipPath)
            let zipModDate = zipAttributes?[.modificationDate] as? Date
            
            // Force extraction in these cases:
            // 1. Directory doesn't exist
            // 2. Zip file is newer than last extraction (or we can't determine dates)
            var shouldExtract = !fileManager.fileExists(atPath: targetDirPath)
            
            // Check if zip is newer than directory (if both exist)
            if !shouldExtract, let zipModDate = zipModDate {
                let dirAttributes = try? fileManager.attributesOfItem(atPath: targetDirPath)
                let dirModDate = dirAttributes?[.modificationDate] as? Date
                
                // Extract if zip is newer or we can't determine directory date
                if let dirModDate = dirModDate {
                    shouldExtract = zipModDate > dirModDate
                    if shouldExtract {
                        print("🔄 Zip file is newer than existing directory, re-extracting: \(zipPath)")
                    }
                } else {
                    shouldExtract = true
                }
            }
            
            // Skip extraction if not needed
            if !shouldExtract {
                print("✅ Directory up to date, skipping extraction: \(targetDirPath)")
                continue
            }
            
            // Remove existing directory if it exists to ensure clean extraction
            if fileManager.fileExists(atPath: targetDirPath) {
                try? fileManager.removeItem(atPath: targetDirPath)
                print("🗑️ Removed existing directory for clean extraction: \(targetDirPath)")
            }
            
            // Proceed with extraction
            let extractionProcess = Process()
            extractionProcess.executableURL = URL(fileURLWithPath: "/usr/bin/unzip")
            extractionProcess.arguments = [
                "-o",                        // Overwrite existing files
                zipPath,                     // Path to zip file
                "-d", frameworkBasePath      // Extraction directory
            ]
            
            print("📦 Extracting \(zipPath)...")
            try extractionProcess.run()
            extractionProcess.waitUntilExit()
            
            if extractionProcess.terminationStatus == 0 {
                print("✅ Successfully extracted \(zipPath)")
                
                // Update directory modification time to match zip file
                // This helps us track when the directory was last extracted
                if let zipModDate = zipModDate {
                    try? fileManager.setAttributes([.modificationDate: zipModDate], ofItemAtPath: targetDirPath)
                }
            } else {
                print("⚠️ Error extracting \(zipPath), status: \(extractionProcess.terminationStatus)")
            }
        }
    }
} catch {
    print("⚠️ Error processing framework directory: \(error.localizedDescription)")
}

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
