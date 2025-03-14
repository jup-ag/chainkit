plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    `maven-publish`
}

android {
    namespace = "uniffi"
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
    publishing {
        singleVariant("release") {
            withSourcesJar()
            withJavadocJar()
        }
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

// Add GitHub Packages publishing configuration
val libraryVersion = project.findProperty("libraryVersion") as String? ?: "0.0.1"
val githubToken = project.findProperty("githubToken") as String? ?: System.getenv("GITHUB_TOKEN")

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "ag.jup.chainkit"
            artifactId = "chainkit"
            version = libraryVersion

            afterEvaluate {
                from(components["release"])
            }

            pom {
                name.set("ChainKit")
                description.set("Android library for interacting with blockchain APIs")
                url.set("https://github.com/jup-ag/chainkit")
                
                licenses {
                    license {
                        name.set("The Apache License, Version 2.0")
                        url.set("http://www.apache.org/licenses/LICENSE-2.0.txt")
                    }
                }
                
                developers {
                    developer {
                        id.set("jupag")
                        name.set("Jupiter Team")
                        email.set("dev@jup.ag")
                    }
                }
                
                scm {
                    connection.set("scm:git:git://github.com/jup-ag/chainkit.git")
                    developerConnection.set("scm:git:ssh://github.com/jup-ag/chainkit.git")
                    url.set("https://github.com/jup-ag/chainkit")
                }
            }
        }
    }
    
    repositories {
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/jup-ag/chainkit")
            credentials {
                username = "github-actions"
                password = githubToken
            }
        }
    }
}
