#
# DEBUG: To run this makefile with a debug configuration, do:
# 	make apple CONFIGURATION="" FOLDER="debug"
#
# DEFAULT: By default, it assumes a release configuration, which is:
# 	make apple
#
# PROFILE: If you want to run the Xcode profiler (Instruments) you need to compile the x86_64 arch:
# 	make profile CONFIGURATION="" FOLDER="debug"
#
# CLEAN: When switching between compiled archs (from 'apple' to 'profile' for example), you might need to clean first:
# 	make clean
#

# Default target to build both Apple and Android
.PHONY: all
all: apple android

STATIC_LIB_NAME := libchainkit.a

# Default be build a release configuration
CONFIGURATION?="--release"
# Default folder is release
FOLDER?="release"

# x86_64 arch is needed to run the Xcode profiler (Instruments) on the simulator
# it's disabled by default to improve compile time in our fat binary
ENABLE_X86?=false

# Should we also build for simulator? Enabled by default but disabled on CI
ENABLE_SIMULATOR?=true

# Define the path to the package framework directory
PACKAGE_FRAMEWORK_PATH=platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework

# Android SDK path (can be overridden via environment variable)
ANDROID_HOME?=$(HOME)/Library/Android/sdk
# Android NDK version
ANDROID_NDK_VERSION?=28.0.12433566
# Android platform target
ANDROID_PLATFORM?=24
# Android NDK Path
ANDROID_NDK_HOME?=$(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)
# Android command line tools version
ANDROID_CMDLINE_TOOLS_VERSION?=11076708
# Android command line tools latest path
ANDROID_CMDLINE_TOOLS_PATH?=$(ANDROID_HOME)/cmdline-tools/latest

# Ensure ios directory exists
$(shell mkdir -p platforms/ios)

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

apple:
	@echo "------> Starting Apple build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> ENABLE_X86: $(ENABLE_X86), ENABLE_SIMULATOR: $(ENABLE_SIMULATOR)"
	$(MAKE) build-framework
	@echo "------> Compressing framework for SPM distribution..."
	./scripts/compress-frameworks.sh
	@echo "------> Apple build completed successfully!"

# Just compresses existing framework files without rebuilding (for testing/development)
compress-framework:
	@echo "------> Running standalone compression of framework files..."
	./scripts/compress-frameworks.sh
	@echo "------> Standalone framework compression completed!"

profile:
	@echo "------> Starting Apple profile build with x86_64 support..."
	$(MAKE) -e ENABLE_X86=1 build-framework
	@echo "------> Apple profile build completed successfully!"

# Android build target
android:
	@echo "------> Starting Android build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> NDK Version: $(ANDROID_NDK_VERSION), Platform: $(ANDROID_PLATFORM)"
	
	# Check Android SDK and install components if needed
	$(call check_android_sdk)
	$(call check_android_cmdline_tools)
	$(call check_android_ndk)
	
	@echo "------> Adding Android targets..."
	rustup target add \
		aarch64-linux-android \
		armv7-linux-androideabi \
		i686-linux-android \
		x86_64-linux-android
	
	@echo "------> Running tests..."
	cargo test
	
	@echo "------> Cleaning..."
	cargo clean
	
	@echo "------> Building libraries..."
	CARGO_PROFILE_RELEASE_STRIP=$(if $(findstring release,$(FOLDER)),true,false) \
	ANDROID_NDK_HOME=$(ANDROID_NDK_HOME) \
	cargo \
		--config target.x86_64-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.i686-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.armv7-linux-androideabi.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi$(ANDROID_PLATFORM)-clang\" \
		--config target.aarch64-linux-android.linker=\"$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android$(ANDROID_PLATFORM)-clang\" \
		ndk \
		--platform $(ANDROID_PLATFORM) \
		--target aarch64-linux-android \
		--target armv7-linux-androideabi \
		--target i686-linux-android \
		--target x86_64-linux-android \
		build $(CONFIGURATION)
	
	@echo "------> Creating directories..."
	mkdir -p platforms/android/chainkit/src/main/jniLibs
	mkdir -p platforms/android/chainkit/src/main/jniLibs/arm64-v8a
	mkdir -p platforms/android/chainkit/src/main/jniLibs/armeabi-v7a
	mkdir -p platforms/android/chainkit/src/main/jniLibs/x86
	mkdir -p platforms/android/chainkit/src/main/jniLibs/x86_64
	
	@echo "------> Copying libraries..."
	cp target/aarch64-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/arm64-v8a/libuniffi_ChainKit.so
	cp target/armv7-linux-androideabi/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/armeabi-v7a/libuniffi_ChainKit.so
	cp target/i686-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/x86/libuniffi_ChainKit.so
	cp target/x86_64-linux-android/$(FOLDER)/libchainkit.so platforms/android/chainkit/src/main/jniLibs/x86_64/libuniffi_ChainKit.so
	
	@echo "------> Copying shared libraries..."
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/arm64-v8a/libc++_shared.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/arm-linux-androideabi/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/armeabi-v7a/libc++_shared.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/i686-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86/libc++_shared.so
	cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86_64/libc++_shared.so
	
	@echo "------> Android build completed successfully!"

build-framework:
	@echo "------> Building framework targets..."
	@make build-targets
	@echo "------> Generating Swift bindings..."
	@make bindgen-swift
	@echo "------> Assembling frameworks..."
	@make assemble-frameworks
	@echo "------> Creating XCFramework..."
	@make xcframework
	@echo "------> Copying framework to Swift Package..."
	@make cp-xcframework-source
	@echo "------> Applying Swift linting fixes..."
	@make apple-swiftlint

clean:
	@echo "------> Cleaning build artifacts..."
	cargo clean
	rm -rf generated
	rm -f platforms/ios/ChainKit/Sources/ChainKit/ChainKit.swift
	rm -rf platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework
	rm -rf platforms/android/chainkit/src/main/jniLibs
	@echo "------> Clean completed successfully!"

# Build targets for all architectures
build-targets:
	@echo "------> Building for architectures..."
	if $(ENABLE_X86); then echo "------> Building for x86_64-apple-ios..."; cargo build $(CONFIGURATION) --target x86_64-apple-ios; fi
	if $(ENABLE_SIMULATOR); then echo "------> Building for aarch64-apple-ios-sim..."; cargo build $(CONFIGURATION) --target aarch64-apple-ios-sim; fi
	@echo "------> Building for aarch64-apple-ios..."
	cargo build $(CONFIGURATION) --target aarch64-apple-ios

# Use Uniffi to build the glue ChainKit.swift as well as the Modulemap
bindgen-swift:
	@echo "------> Generating Swift bindings from UDL..."
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/interface.udl --out-dir generated --language swift
	@echo "------> Fixing modulemap..."
	sed -i '' 's/module\ ChainKitFFI/framework\ module\ ChainKitFFI/' generated/ChainKitFFI.modulemap

bindgen-kotlin:
	@echo "------> Generating Kotlin bindings from UDL..."
	uniffi-bindgen generate src/hello.udl --language kotlin -o platforms/android/UniffiRustExample/app/src/main/java
	@echo "------> Fixing Kotlin bindings..."
	sed -i '' 's/return "uniffi_Hello"/return "hello"/' platforms/android/UniffiRustExample/app/src/main/java/uniffi/Hello/Hello.kt

# Take the different targets, and put them into ChainKit.framework files per architecture
assemble-frameworks:
	@echo "------> Removing existing frameworks..."
	find . -type d -name ChainKitFFI.framework -exec rm -rf {} \; || echo "rm failed"
	if ${ENABLE_X86}; then \
		@echo "------> Assembling x86_64 framework..."; \
		cd target/x86_64-apple-ios/$(FOLDER) && mkdir -p ChainKitFFI.framework && cd ChainKitFFI.framework && \
			mkdir Headers Modules && cp ../../../../generated/ChainKitFFI.modulemap ./Modules/module.modulemap && \
			cp ../../../../generated/ChainKitFFI.h ./Headers/ChainKitFFI.h && cp ../$(STATIC_LIB_NAME) ./ChainKitFFI && \
			cp ../../../../resources/Info.plist ./; \
	fi;
	if ${ENABLE_SIMULATOR}; then \
		@echo "------> Assembling simulator framework..."; \
	  cd target/aarch64-apple-ios-sim/$(FOLDER) && mkdir -p ChainKitFFI.framework && cd ChainKitFFI.framework && \
	  	mkdir Headers Modules Resources && cp ../../../../generated/ChainKitFFI.modulemap ./Modules/module.modulemap && \
	  	cp ../../../../generated/ChainKitFFI.h ./Headers/ChainKitFFI.h && cp ../$(STATIC_LIB_NAME) ./ChainKitFFI && \
	  	cp ../../../../resources/Info.plist ./ && cp ../../../../resources/Info.plist ./Resources; \
	fi;
	@echo "------> Assembling iOS framework..."
	cd target/aarch64-apple-ios/$(FOLDER) && mkdir -p ChainKitFFI.framework && cd ChainKitFFI.framework && \
		mkdir Headers Modules && cp ../../../../generated/ChainKitFFI.modulemap ./Modules/module.modulemap && \
		cp ../../../../generated/ChainKitFFI.h ./Headers/ChainKitFFI.h && cp ../$(STATIC_LIB_NAME) ./ChainKitFFI && \
		cp ../../../../resources/Info.plist ./;

xcframework:
	@echo "------> Creating XCFramework..."
	rm -rf target/ChainKitFFI.xcframework || echo "skip removing"
	$(call combine_targets)

# Copy the xcframework and the source into the Swift Package folder
cp-xcframework-source:
	@echo "------> Creating Swift Package directories..."
	mkdir -p platforms/ios/ChainKit/Sources
	@echo "------> Copying XCFramework to Swift Package..."
	cp -r target/ChainKitFFI.xcframework platforms/ios/ChainKit/Sources/
	@echo "------> Copying Swift bindings to Swift Package..."
	cp generated/ChainKit.swift platforms/ios/ChainKit/Sources/ChainKit

# Fix the generated sourcecode so that our linter doesn't complain
apple-swiftlint:
	@echo "------> Applying Swift linting fixes..."
	python3 resources/mutations.py

fuck:
	$(call combine_targets)

# Based on the environment variables, either build
# - Simulator
# - X86
# - Arch64
define combine_targets
	@echo "------> Combining targets for XCFramework..."
	@echo "------> ENABLE_X86 set to: $(ENABLE_X86)"
	@echo "------> ENABLE_SIMULATOR set to: $(ENABLE_SIMULATOR)"
	if ${ENABLE_X86} || ${ENABLE_SIMULATOR}; then \
		if ${ENABLE_X86}; then \
		  @echo "------> Creating fat binary with x86_64 and arm64 simulator..."; \
		  lipo -create \
		    target/x86_64-apple-ios/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
		  	target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
		  	-output target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI; \
		else \
		  @echo "------> Creating simulator binary..."; \
		  lipo -create \
			  target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI \
				-output target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework/ChainKitFFI; \
		fi; \
		@echo "------> Creating XCFramework with device and simulator..."; \
	  xcodebuild -create-xcframework \
		  -framework target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework \
			-framework target/aarch64-apple-ios-sim/$(FOLDER)/ChainKitFFI.framework \
			-output target/ChainKitFFI.xcframework; \
	else \
	  @echo "------> Creating XCFramework with device only..."; \
	  xcodebuild -create-xcframework \
		  -framework target/aarch64-apple-ios/$(FOLDER)/ChainKitFFI.framework \
			-output target/ChainKitFFI.xcframework; \
	fi;
endef
