package dev.k1s0.grpc.client

import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test

class ChannelFactoryTest {

    @Test
    fun `create returns a channel that is not shutdown`() {
        val channel = ChannelFactory.create("localhost", 50051, usePlaintext = true)

        channel.isShutdown shouldBe false

        ChannelFactory.shutdown(channel)
        channel.isShutdown shouldBe true
    }
}
