rootProject.name = "k1s0-kotlin"

enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

dependencyResolution {
    versionCatalogs {
        create("libs") {
            from(files("gradle/libs.versions.toml"))
        }
    }
}

include(
    ":packages:k1s0-error",
    ":packages:k1s0-config",
    ":packages:k1s0-validation",
    ":packages:k1s0-observability",
    ":packages:k1s0-grpc-server",
    ":packages:k1s0-grpc-client",
    ":packages:k1s0-health",
    ":packages:k1s0-db",
    ":packages:k1s0-domain-event",
    ":packages:k1s0-resilience",
    ":packages:k1s0-cache",
    ":packages:k1s0-auth",
)
