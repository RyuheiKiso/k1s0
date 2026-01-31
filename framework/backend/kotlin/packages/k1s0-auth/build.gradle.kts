plugins {
    alias(libs.plugins.kotlin.serialization)
}

dependencies {
    implementation(project(":packages:k1s0-error"))
    implementation(project(":packages:k1s0-config"))
    implementation(project(":packages:k1s0-observability"))

    implementation(libs.nimbus.jose.jwt)
    implementation(libs.ktor.server.core)
    implementation(libs.ktor.server.auth)
    implementation(libs.ktor.server.auth.jwt)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlin.logging)
    implementation(libs.kotlinx.coroutines.core)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
    testImplementation(libs.mockk)
}
