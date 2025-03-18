# ChainKit Release Process

This document describes the release process for ChainKit, including how to create a new release with iOS XCFramework and Android AAR distributions.

## Prerequisites

### Rust Setup

ChainKit uses Rust for its core functionality. The build system will automatically check for and set up Rust if needed, but you can also install it manually:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Android SDK Setup

For Android builds, you need the Android SDK. The build system will automatically set up the necessary components, but you should set the `ANDROID_HOME` environment variable:

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
# Or wherever your Android SDK is installed
```

## Creating a New Release

To create a new release, run:

```bash
make release VERSION=x.y.z
```

This will:

1. Create a new GitHub release with the specified version
2. Package and upload the iOS XCFramework to the release
3. Package and upload the Android AAR to the release
4. Publish the Android AAR to GitHub Packages

For convenience, you can also use the "yolo" target which cleans, builds all platforms, and creates a release in one command:

```bash
make yolo VERSION=x.y.z
```

## Release Scripts

The release process is divided into three scripts:

1. `scripts/create_github_release.sh` - Creates a new GitHub release
2. `scripts/prepare_xcframework_for_distribution.sh` - Packages and uploads the iOS XCFramework
3. `scripts/prepare_aar_for_distribution.sh` - Packages, uploads, and publishes the Android AAR

You can run these scripts individually if needed, but normally they are called by the `make release` command.

## Using the Android Library with GitHub Packages

To use the Android library from GitHub Packages in your project:

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
    implementation("ag.jup.chainkit:chainkit:x.y.z") // Replace x.y.z with the desired version
}
```

3. Configure GitHub credentials in one of the following ways:

   - **Environment Variables**: Set `GITHUB_ACTOR` (your GitHub username) and `GITHUB_TOKEN` (your Personal Access Token with `read:packages` scope)
   
   - **Gradle Properties**: Add the following to your `~/.gradle/gradle.properties` file:
     ```
     gpr.user=YOUR_GITHUB_USERNAME
     gpr.key=YOUR_GITHUB_TOKEN
     ```

   - **Project Properties**: Add the credentials to your project's `gradle.properties` file (not recommended for public repositories)

## Using the iOS Swift Package

The release process automatically updates the `Package.swift` file with the latest version information. 

To use the Swift Package in your project:

1. In Xcode, go to File > Add Package Dependencies...
2. Enter the repository URL: `https://github.com/jup-ag/chainkit`
3. Choose the appropriate version constraint
4. Select the ChainKit package and click Add Package

## Troubleshooting

### Rust Environment Issues

The build system is designed to automatically detect and set up Rust if needed. If you encounter any issues with Rust commands not being found, you can:

1. Manually install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. Ensure Rust is in your PATH:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. You can also run any of the make commands with an explicit PATH:
   ```bash
   PATH="$HOME/.cargo/bin:$PATH" make android
   ```

### Android SDK Issues

If you encounter issues with the Android SDK:

1. Make sure the ANDROID_HOME environment variable is set:
   ```bash
   export ANDROID_HOME=$HOME/Library/Android/sdk
   ```

2. You can specify NDK version explicitly:
   ```bash
   make android ANDROID_NDK_VERSION=28.0.12433566
   ```

3. For more verbose output during the build:
   ```bash
   CARGO_LOG=debug make android
   ```

> **Note**: All Android architectures (aarch64, armv7, i686, x86_64) must build successfully for 
> the build process to complete. If you encounter build failures for specific architectures, 
> you'll need to debug and resolve those issues. The build system is configured to fail if any
> architecture fails to build to ensure consistent support across all target platforms.