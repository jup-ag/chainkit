pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/jup-ag/chainkit")
            credentials {
                val githubToken = providers.gradleProperty("githubToken").orNull ?: System.getenv("GITHUB_TOKEN")
                username = System.getenv("GITHUB_ACTOR") ?: "github-actions"
                password = githubToken
            }
            content {
                includeGroup("ag.jup.chainkit")
            }
        }
    }
}

rootProject.name = "ChainKit"
include(":chainkit")
