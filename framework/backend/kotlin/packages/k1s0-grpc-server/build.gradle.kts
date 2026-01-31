dependencies {
    implementation(project(":packages:k1s0-error"))
    implementation(project(":packages:k1s0-observability"))

    implementation(libs.grpc.kotlin.stub)
    implementation(libs.grpc.netty.shaded)
    implementation(libs.grpc.protobuf)
    implementation(libs.grpc.stub)
    implementation(libs.kotlin.logging)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.opentelemetry.api)

    testImplementation(libs.junit.jupiter)
    testImplementation(libs.kotest.assertions.core)
    testImplementation(libs.grpc.testing)
    testImplementation(libs.kotlinx.coroutines.test)
}
