# ChainKit Release Process

This document explains the simplified process for releasing the ChainKit framework using Swift Package Manager's remote binary target capability.

## Overview

The XCFramework can be packaged and uploaded to GitHub Releases in a single command. The process:

1. Packages the XCFramework and creates a zip archive
2. Uploads it to GitHub Releases with the specified version
3. Automatically updates Package.swift with the new URL and checksum

## Prerequisites

The XCFramework must be already built (by running `make apple`)

The release process automatically handles:
- Installing GitHub CLI if needed
- Authentication through your SSH credentials
- Creating GitHub releases
- Uploading the framework
- Computing the checksum
- Updating Package.swift

## Release Process

1. First build the framework if you haven't already:

```bash
make apple
```

2. Then create and upload the release:

```bash
make release VERSION=1.0.0
```

That's it! The framework is now available for Swift Package Manager to download, and your Package.swift has been automatically updated with the correct URL and checksum.