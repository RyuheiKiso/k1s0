dependencies {
    implementation(project(":packages:k1s0-error"))
    implementation(project(":packages:k1s0-config"))
    implementation(project(":packages:k1s0-db"))
    implementation(project(":packages:k1s0-domain-event"))
    implementation(project(":packages:k1s0-observability"))

    implementation(libs.exposed.core)
    implementation(libs.exposed.jdbc)
    implementation(libs.exposed.json)
    implementation(libs.hikari)
    implementation(libs.kotlin.logging)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.micrometer.registry.prometheus)
    implementation(libs.lettuce.core)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
    testImplementation(libs.mockk)
    testImplementation(libs.kotlinx.coroutines.test)
}
