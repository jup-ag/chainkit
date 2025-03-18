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

### Build Outputs

After successful builds, the framework files will be located at:

- **iOS/macOS**: `platforms/ios`
  - Swift interface: `ChainKit.swift`
  - XCFramework: `ChainKitFFI.xcframework`

- **Android**: `platforms/android`
  - Native libraries for multiple architectures (arm64-v8a, armeabi-v7a, x86, x86_64)

## Release

**Create a new (or override an existing) release and upload (previously built) binary frameworks:**
```bash
make release VERSION=1.0.0
```

**Clean, build all, and release in one command:**
```bash
make yolo VERSION=1.0.0
```

Please see [RELEASE](https://github.com/jup-ag/chainkit/blob/main/RELEASE.md) for more details.

## Using ChainKit as a Dependency

### iOS (Swift Package Manager)

Add ChainKit as a standard Swift Package Manager dependency:

```swift
dependencies: [
    .package(url: "https://github.com/jup-ag/chainkit.git", from: "1.0.0")
]
```

### Android (Gradle)

Add ChainKit as a dependency using GitHub Packages:

1. Add the GitHub Packages repository to your `settings.gradle` or `settings.gradle.kts`:

```kotlin
dependencyResolutionManagement {
    repositories {
        // ... other repositories ...
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/jup-ag/chainkit")
            credentials {
                username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
                password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
```

2. Add the dependency to your module's `build.gradle` or `build.gradle.kts`:

```kotlin
dependencies {
    implementation("ag.jup.chainkit:chainkit:1.0.0")
}
```

3. Make sure to provide GitHub credentials either through environment variables or Gradle properties.
