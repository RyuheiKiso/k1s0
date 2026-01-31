package dev.k1s0.grpc.server

import io.kotest.matchers.shouldNotBe
import org.junit.jupiter.api.Test

class GrpcServerTest {

    @Test
    fun `GrpcServer builder accepts services and interceptors`() {
        val server = GrpcServer(0)
            .addInterceptor(ErrorInterceptor())
            .addInterceptor(TracingInterceptor())

        server shouldNotBe null
    }
}
