plugins {
    alias(libs.plugins.kotlin.serialization)
}

dependencies {
    implementation(project(":packages:k1s0-config"))

    implementation(libs.lettuce.core)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlin.logging)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
    testImplementation(libs.kotlinx.coroutines.test)
}
