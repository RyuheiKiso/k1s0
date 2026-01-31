dependencies {
    implementation(project(":packages:k1s0-config"))

    implementation(libs.kotlin.logging)
    implementation(libs.logback.classic)
    implementation(libs.opentelemetry.api)
    implementation(libs.opentelemetry.sdk)
    implementation(libs.opentelemetry.exporter.otlp)
    implementation(libs.opentelemetry.exporter.logging)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
}
