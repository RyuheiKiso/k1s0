pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolution {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "k1s0-android-framework"

include(":packages:k1s0-navigation")
include(":packages:k1s0-config")
include(":packages:k1s0-http")
include(":packages:k1s0-ui")
include(":packages:k1s0-auth")
include(":packages:k1s0-observability")
include(":packages:k1s0-state")
include(":packages:k1s0-realtime")
