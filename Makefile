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

# Android NDK version
ANDROID_NDK_VERSION?=28.0.12433566
# Android platform target
ANDROID_PLATFORM?=24

# Ensure ios directory exists
$(shell mkdir -p platforms/ios)

apple:
	@echo "------> Starting Apple build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> ENABLE_X86: $(ENABLE_X86), ENABLE_SIMULATOR: $(ENABLE_SIMULATOR)"
	$(MAKE) build-framework
	@echo "------> Apple build completed successfully!"

profile:
	@echo "------> Starting Apple profile build with x86_64 support..."
	$(MAKE) -e ENABLE_X86=1 build-framework
	@echo "------> Apple profile build completed successfully!"

# Android build target
android:
	@echo "------> Starting Android build..."
	@echo "------> Configuration: $(CONFIGURATION), Folder: $(FOLDER)"
	@echo "------> NDK Version: $(ANDROID_NDK_VERSION), Platform: $(ANDROID_PLATFORM)"
	
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
	cargo \
		--config target.x86_64-linux-android.linker=\"$(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.i686-linux-android.linker=\"$(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android$(ANDROID_PLATFORM)-clang\" \
		--config target.armv7-linux-androideabi.linker=\"$(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi$(ANDROID_PLATFORM)-clang\" \
		--config target.aarch64-linux-android.linker=\"$(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android$(ANDROID_PLATFORM)-clang\" \
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
	cp $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/arm64-v8a/libc++_shared.so
	cp $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/arm-linux-androideabi/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/armeabi-v7a/libc++_shared.so
	cp $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/i686-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86/libc++_shared.so
	cp $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so platforms/android/chainkit/src/main/jniLibs/x86_64/libc++_shared.so
	
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
