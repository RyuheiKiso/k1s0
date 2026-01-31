plugins {
    alias(libs.plugins.kotlin.serialization)
}

dependencies {
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlin.logging)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
}
