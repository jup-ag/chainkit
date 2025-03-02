# ChainKit

ChainKit is a cross-platform blockchain utility library providing native interfaces for both iOS/macOS and Android platforms. Built with Rust at its core, ChainKit offers high-performance cryptographic and blockchain utilities with native bindings for Swift and Kotlin.

## Building the Framework

ChainKit can be built for Apple platforms (iOS/macOS) and Android.

### Prerequisites

- Rust and Cargo
- Xcode and iOS SDK (for Apple platforms)

### Build Commands

**Build for both platforms:**
```bash
make
```

**Build for Apple platforms only:**
```bash
make apple
```

**Build for Android only:**
```bash
make android
```

**Clean build artifacts:**
```bash
make clean
```

**Profile build (with x86_64 support for Instruments):**
```bash
make profile
```

### Build Outputs

After successful builds, the framework files will be located at:

- **iOS/macOS**: `platforms/ios`
  - Swift interface: `ChainKit.swift`
  - XCFramework: `ChainKitFFI.xcframework`

- **Android**: `platforms/android`
  - Native libraries for multiple architectures (arm64-v8a, armeabi-v7a, x86, x86_64)

## Using ChainKit as a Dependency

### iOS (Swift Package Manager)

Add ChainKit as a standard Swift Package Manager dependency:

```swift
dependencies: [
    .package(url: "https://github.com/jup-ag/chainkit.git", from: "1.0.0")
]
```

### Android (Gradle)

TODO: Add Gradle Instructions

## Advanced Configuration

For advanced build configurations, check the Makefile comments for options like:
- Debug versus release builds
- Enabling x86_64 architecture for Apple platforms
- Simulator support settings
