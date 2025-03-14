################################################################################
#
# ChainKit Makefile
#
# Primary targets:
#   make all      - Build both Apple and Android
#   make apple    - Build Apple frameworks (iOS)
#   make android  - Build Android libraries and AAR
#   make clean    - Clean all build artifacts
#   make release  - Create and upload a release (VERSION=x.y.z required)
#
################################################################################

################################################################################
# COMMON CONFIGURATION
################################################################################

# Source the Rust environment
RUST_ENV := source ./scripts/ensure_rust_env.sh

# Default configuration
CONFIGURATION ?= "--release"
FOLDER ?= "release"
STATIC_LIB_NAME := libchainkit.a

# Apple configuration
ENABLE_X86 ?= false
ENABLE_SIMULATOR ?= true

# Android configuration  
ANDROID_HOME ?= $(HOME)/Library/Android/sdk
ANDROID_NDK_VERSION ?= 28.0.12433566
ANDROID_PLATFORM ?= 28
ANDROID_NDK_HOME ?= $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)
ANDROID_CMDLINE_TOOLS_VERSION ?= 11076708
ANDROID_CMDLINE_TOOLS_PATH ?= $(ANDROID_HOME)/cmdline-tools/latest
ANDROID_ARCHS ?= aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Ensure directories exist
$(shell mkdir -p platforms/ios)

################################################################################
# PRIMARY TARGETS
################################################################################

# Default target to build both Apple and Android
.PHONY: all
all: apple android

# Build Apple frameworks
.PHONY: apple
apple:
	@echo "------> Starting Apple build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> ENABLE_X86: $(ENABLE_X86), ENABLE_SIMULATOR: $(ENABLE_SIMULATOR)"
	@echo "------> Checking for Rust toolchain..."
	@bash -c '$(RUST_ENV) && \
		echo "------> Building framework targets..." && \
		$(call build_apple_targets) && \
		echo "------> Generating Swift bindings..." && \
		cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/interface.udl --out-dir generated --language swift && \
		if [ -f generated/ChainKitFFI.modulemap ]; then \
			echo "------> Fixing modulemap..." && \
			awk "{gsub(/module ChainKitFFI/, \"framework module ChainKitFFI\"); print}" generated/ChainKitFFI.modulemap > generated/ChainKitFFI.modulemap.new && \
			mv generated/ChainKitFFI.modulemap.new generated/ChainKitFFI.modulemap && \
			echo "Modulemap successfully updated."; \
		else \
			echo "Warning: modulemap file not found at generated/ChainKitFFI.modulemap"; \
		fi && \
		echo "------> Assembling frameworks..." && \
		$(call assemble_apple_frameworks) && \
		echo "------> Creating XCFramework..." && \
		$(call create_xcframework) && \
		echo "------> Copying framework to Swift Package..." && \
		$(call copy_xcframework_to_package)'
	@echo "------> Apple build completed successfully!"

# Build Android libraries and AAR
.PHONY: android
android:
	@echo "------> Starting Android build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> NDK Version: $(ANDROID_NDK_VERSION), Platform: $(ANDROID_PLATFORM)"
	@echo "------> Building for architectures: $(ANDROID_ARCHS)"
	
	# Check Android SDK and install components
	$(call check_android_sdk)
	$(call check_android_cmdline_tools)
	$(call check_android_ndk)
	
	@echo "------> Checking for Rust toolchain..."
	bash -c '$(RUST_ENV) && \
		echo "------> Adding Android targets..." && \
		rustup target add $(ANDROID_ARCHS)'
	
	@echo "------> Running tests..."
	bash -c '$(RUST_ENV) && cargo test'
	
	# Build the libraries
	@echo "------> Building libraries..."
	bash -c '$(RUST_ENV) && $(call build_android_libs)'
	
	# Create directories for libraries
	@echo "------> Creating directories..."
	mkdir -p platforms/android/chainkit/src/main/jniLibs
	mkdir -p platforms/android/chainkit/src/main/jniLibs/arm64-v8a
	mkdir -p platforms/android/chainkit/src/main/jniLibs/armeabi-v7a
	mkdir -p platforms/android/chainkit/src/main/jniLibs/x86
	mkdir -p platforms/android/chainkit/src/main/jniLibs/x86_64
	
	# Copy libraries to appropriate directories
	@echo "------> Copying libraries..."
	cp target/aarch64-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/arm64-v8a/libuniffi_ChainKit.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/arm64-v8a/libc++_shared.so
	cp target/armv7-linux-androideabi/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/armeabi-v7a/libuniffi_ChainKit.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/arm-linux-androideabi/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/armeabi-v7a/libc++_shared.so
	cp target/i686-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/x86/libuniffi_ChainKit.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/i686-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86/libc++_shared.so
	cp target/x86_64-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/x86_64/libuniffi_ChainKit.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86_64/libc++_shared.so
	
	# Generate Kotlin bindings
	@echo "------> Generating Kotlin bindings..."
	bash -c '$(RUST_ENV) && cargo run --manifest-path Cargo.toml --features="uniffi/cli" --bin uniffi-bindgen generate src/interface.udl --language kotlin --out-dir platforms/android/chainkit/src/main/java'
	@echo "------> Kotlin bindings generated successfully!"
	
	# Build AAR
	@echo "------> Building Android AAR with Gradle..."
	@if [ -n "$(ANDROID_HOME)" ]; then \
		$(ANDROID_HOME)/cmdline-tools/latest/bin/sdkmanager --licenses || echo "Please accept all licenses manually for this build to succeed"; \
	fi
	@cd platforms/android && ./gradlew chainkit:assembleRelease
	@echo "------> Android AAR built successfully at platforms/android/chainkit/build/outputs/aar/chainkit-release.aar"
	
	@echo "------> Android build completed successfully!"

# Clean all build artifacts
.PHONY: clean
clean:
	@echo "------> Cleaning build artifacts..."
	@bash -c '$(RUST_ENV) && cargo clean'
	rm -rf generated
	rm -f platforms/ios/ChainKit/Sources/ChainKit.swift
	rm -rf platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework
	rm -rf platforms/android/chainkit/src/main/jniLibs
	@echo "------> Clean completed successfully!"

# Release a version
.PHONY: release
release:
	@echo "------> Creating and uploading release with version $(VERSION)"
	@if [ -z "$(VERSION)" ]; then \
		echo "ERROR: VERSION not specified"; \
		echo "Usage: make release VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "------> Android architectures: $(ANDROID_ARCHS)"
	@echo "------> NOTE: All architectures must build successfully for the release to complete."
	@echo "------> Checking for Rust toolchain..."
	@bash -c '$(RUST_ENV) && \
	echo "------> Creating GitHub release..." && \
	./scripts/create_github_release.sh $(VERSION) && \
	echo "------> Preparing and uploading XCFramework..." && \
	./scripts/prepare_xcframework_for_distribution.sh $(VERSION) && \
	echo "------> Preparing and uploading Android AAR..." && \
	./scripts/prepare_aar_for_distribution.sh $(VERSION)'
	@echo "------> Release v$(VERSION) completed!"

################################################################################
# UTILITY FUNCTIONS
################################################################################

# Build targets for all Apple architectures
define build_apple_targets
	echo "------> Building for architectures..."; \
	echo "------> Checking for installed Rust targets..."; \
	rustup target list --installed; \
	echo "------> Installing required targets..."; \
	rustup target add aarch64-apple-ios; \
	if $(ENABLE_SIMULATOR); then \
		echo "------> Installing simulator target..."; \
		rustup target add aarch64-apple-ios-sim; \
	fi; \
	if $(ENABLE_X86); then \
		echo "------> Installing x86_64 target..."; \
		rustup target add x86_64-apple-ios; \
	fi; \
	echo "------> Building for aarch64-apple-ios..."; \
	cargo build $(CONFIGURATION) --target aarch64-apple-ios || { echo "❌ Failed to build aarch64-apple-ios target"; exit 1; }; \
	if $(ENABLE_SIMULATOR); then \
		echo "------> Building for aarch64-apple-ios-sim..."; \
		cargo build $(CONFIGURATION) --target aarch64-apple-ios-sim || { echo "❌ Failed to build aarch64-apple-ios-sim target"; exit 1; }; \
	fi; \
	if $(ENABLE_X86); then \
		echo "------> Building for x86_64-apple-ios..."; \
		cargo build $(CONFIGURATION) --target x86_64-apple-ios || { echo "❌ Failed to build x86_64-apple-ios target"; exit 1; }; \
	fi; \
	echo "------> All targets built successfully!"
endef

# Assemble frameworks for each architecture
define assemble_apple_frameworks
	echo "------> Removing existing frameworks..."; \
	find . -type d -name ChainKitFFI.framework -exec rm -rf {} \; 2>/dev/null || echo "No existing frameworks found"; \
	echo "------> Checking for static libraries..."; \
	echo "------> Looking for iOS static library at: target/aarch64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME)"; \
	ls -la target/aarch64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME) || echo "❌ ERROR: iOS static library not found"; \
	if $(ENABLE_SIMULATOR); then \
		echo "------> Looking for simulator static library at: target/aarch64-apple-ios-sim/$(FOLDER)/$(STATIC_LIB_NAME)"; \
		ls -la target/aarch64-apple-ios-sim/$(FOLDER)/$(STATIC_LIB_NAME) || echo "❌ ERROR: simulator static library not found"; \
	fi; \
	if $(ENABLE_X86); then \
		echo "------> Looking for x86_64 static library at: target/x86_64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME)"; \
		ls -la target/x86_64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME) || echo "❌ ERROR: x86_64 static library not found"; \
	fi; \
	echo "------> Checking for generated files..."; \
	ls -la generated/ || echo "❌ ERROR: Generated files not found"; \
	echo "------> Checking for resources..."; \
	ls -la resources/ || echo "❌ ERROR: Resource files not found"; \
	ROOT_DIR=$$(pwd); \
	if $(ENABLE_X86) && [ -f "target/x86_64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME)" ]; then \
		echo "------> Assembling x86_64 framework..."; \
		cd target/x86_64-apple-ios/$(FOLDER) && mkdir -p ChainKitFFI.framework && cd ChainKitFFI.framework && \
			mkdir -p Headers Modules && cp $$ROOT_DIR/generated/ChainKitFFI.modulemap ./Modules/module.modulemap && \
			cp $$ROOT_DIR/generated/ChainKitFFI.h ./Headers/ChainKitFFI.h && cp ../$(STATIC_LIB_NAME) ./ChainKitFFI && \
			cp $$ROOT_DIR/resources/Info.plist ./ && \
			echo "✅ Successfully created x86_64 framework"; \
		cd $$ROOT_DIR; \
	fi; \
	if $(ENABLE_SIMULATOR) && [ -f "target/aarch64-apple-ios-sim/$(FOLDER)/$(STATIC_LIB_NAME)" ]; then \
		echo "------> Assembling simulator framework..."; \
		cd target/aarch64-apple-ios-sim/$(FOLDER) && mkdir -p ChainKitFFI.framework && cd ChainKitFFI.framework && \
			mkdir -p Headers Modules Resources && cp $$ROOT_DIR/generated/ChainKitFFI.modulemap ./Modules/module.modulemap && \
			cp $$ROOT_DIR/generated/ChainKitFFI.h ./Headers/ChainKitFFI.h && cp ../$(STATIC_LIB_NAME) ./ChainKitFFI && \
			cp $$ROOT_DIR/resources/Info.plist ./ && cp $$ROOT_DIR/resources/Info.plist ./Resources && \
			echo "✅ Successfully created simulator framework"; \
		cd $$ROOT_DIR; \
	fi; \
	echo "------> Assembling iOS framework..."; \
	echo "------> Current directory: $$(pwd)"; \
	echo "------> Checking iOS static library:"; \
	ls -la target/aarch64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME); \
	mkdir -p target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/Headers; \
	mkdir -p target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/Modules; \
	cp generated/ChainKitFFI.modulemap target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/Modules/module.modulemap; \
	cp generated/ChainKitFFI.h target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/Headers/; \
	cp target/aarch64-apple-ios/$(FOLDER)/$(STATIC_LIB_NAME) target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI; \
	cp resources/Info.plist target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework/; \
	echo "✅ Successfully created iOS framework"; \
	echo "------> Frameworks assembled. Checking..."; \
	find . -name "ChainKitFFI.framework" | sort
endef

# Create XCFramework from component frameworks
define create_xcframework
	echo "------> Creating XCFramework..."; \
	rm -rf target/ChainKitFFI.xcframework || echo "skip removing"; \
	echo "------> Combining targets for XCFramework..."; \
	mkdir -p target/ChainKitFFI.xcframework; \
	echo "------> ENABLE_X86 set to: $(ENABLE_X86)"; \
	echo "------> ENABLE_SIMULATOR set to: $(ENABLE_SIMULATOR)"; \
	if [ -d "target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework" ]; then \
		if $(ENABLE_X86) || $(ENABLE_SIMULATOR); then \
			if $(ENABLE_X86) && [ -d "target/x86_64-apple-ios/$(FOLDER)/ChainKitFFI.framework" ] && [ -d "target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework" ]; then \
				echo "------> Creating fat binary with x86_64 and arm64 simulator..."; \
				lipo -create \
					target/x86_64-apple-ios/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
					target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
					-output target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI; \
			elif [ -d "target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework" ]; then \
				echo "------> Creating simulator binary..."; \
				lipo -create \
					target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
					-output target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI; \
			fi; \
			if [ -d "target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework" ]; then \
				echo "------> Creating XCFramework with device and simulator..."; \
				xcodebuild -create-xcframework \
					-framework target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework \
					-framework target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework \
					-output target/ChainKitFFI.xcframework; \
			else \
				echo "------> Creating XCFramework with device only (simulator not found)..."; \
				xcodebuild -create-xcframework \
					-framework target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework \
					-output target/ChainKitFFI.xcframework; \
			fi; \
		else \
			echo "------> Creating XCFramework with device only..."; \
			xcodebuild -create-xcframework \
				-framework target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework \
				-output target/ChainKitFFI.xcframework; \
		fi; \
	fi
endef

# Copy the XCFramework to the Swift Package
define copy_xcframework_to_package
	echo "------> Copying XCFramework to Swift Package..."; \
	mkdir -p platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework; \
	rsync -a target/ChainKitFFI.xcframework/ platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework/ || true; \
	echo "------> Copying Swift bindings to Swift Package..."; \
	cp generated/ChainKit.swift platforms/ios/ChainKit/Sources/ 2>/dev/null || touch platforms/ios/ChainKit/Sources/ChainKit.swift
endef

# Check if Android SDK is installed
define check_android_sdk
	@if [ -z "$(ANDROID_HOME)" ]; then \
		echo "ERROR: ANDROID_HOME environment variable is not set"; \
		echo "Please set ANDROID_HOME to the location of your Android SDK"; \
		exit 1; \
	fi
	@if [ ! -d "$(ANDROID_HOME)" ]; then \
		echo "------> Creating Android SDK directory at $(ANDROID_HOME)"; \
		mkdir -p "$(ANDROID_HOME)"; \
	fi
	@echo "------> Using Android SDK at $(ANDROID_HOME)"
endef

# Check if Android command line tools are installed
define check_android_cmdline_tools
	@if [ ! -d "$(ANDROID_CMDLINE_TOOLS_PATH)" ]; then \
		echo "------> Android command line tools not found at $(ANDROID_CMDLINE_TOOLS_PATH)"; \
		echo "------> Installing Android command line tools..."; \
		CMDLINE_TOOLS_ZIP="commandlinetools-mac-$(ANDROID_CMDLINE_TOOLS_VERSION)_latest.zip"; \
		DOWNLOAD_URL="https://dl.google.com/android/repository/$$CMDLINE_TOOLS_ZIP"; \
		TMP_DIR=$$(mktemp -d); \
		curl -L $$DOWNLOAD_URL -o $$TMP_DIR/$$CMDLINE_TOOLS_ZIP; \
		mkdir -p "$(ANDROID_HOME)/cmdline-tools"; \
		unzip -q $$TMP_DIR/$$CMDLINE_TOOLS_ZIP -d $$TMP_DIR; \
		mv $$TMP_DIR/cmdline-tools "$(ANDROID_HOME)/cmdline-tools/latest"; \
		rm -rf $$TMP_DIR; \
		if [ ! -d "$(ANDROID_CMDLINE_TOOLS_PATH)" ]; then \
			echo "ERROR: Failed to install Android command line tools"; \
			exit 1; \
		fi; \
		echo "------> Android command line tools installed successfully"; \
	else \
		echo "------> Using Android command line tools from $(ANDROID_CMDLINE_TOOLS_PATH)"; \
	fi
endef

# Check if Android NDK is installed
define check_android_ndk
	@if [ ! -d "$(ANDROID_NDK_HOME)" ]; then \
		echo "------> Android NDK $(ANDROID_NDK_VERSION) not found at $(ANDROID_NDK_HOME)"; \
		echo "------> Installing Android NDK $(ANDROID_NDK_VERSION)..."; \
		"$(ANDROID_CMDLINE_TOOLS_PATH)/bin/sdkmanager" --install "ndk;$(ANDROID_NDK_VERSION)"; \
		if [ ! -d "$(ANDROID_NDK_HOME)" ]; then \
			echo "ERROR: Failed to install Android NDK $(ANDROID_NDK_VERSION)"; \
			exit 1; \
		fi; \
		echo "------> Android NDK $(ANDROID_NDK_VERSION) installed successfully"; \
	else \
		echo "------> Using Android NDK $(ANDROID_NDK_VERSION) from $(ANDROID_NDK_HOME)"; \
	fi
endef

# Build Android libraries
define build_android_libs
	set -x && \
	CARGO_PROFILE_RELEASE_STRIP=$(if $(findstring release,$(FOLDER)),true,false) \
	ANDROID_NDK_HOME=$(ANDROID_NDK_HOME) \
	cargo \
		--verbose \
		--config target.x86_64-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.i686-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.armv7-linux-androideabi.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi$(ANDROID_PLATFORM)-clang\" \
		--config target.aarch64-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android$(ANDROID_PLATFORM)-clang\" \
		ndk \
		--platform $(ANDROID_PLATFORM) \
		$(addprefix --target ,$(ANDROID_ARCHS)) \
		build $(CONFIGURATION) || { echo "❌ ERROR: Cargo build failed with exit code $$?"; exit 1; }
endef