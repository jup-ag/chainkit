plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

group = "ag.jup.jupiter.android.library.chainkit"
version = "1.0.2"

android {
    namespace = "ag.jup.jupiter.android.library.chainkit"
    compileSdk = 35

    defaultConfig {
        minSdk = 28

        ndk {
            debugSymbolLevel = "FULL"
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64") // If using native `.so` files
        }
    }
    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
}

dependencies {
    implementation(libs.jna) {
        artifact {
            name = "jna"
            type = "aar"
        }
    }
}
