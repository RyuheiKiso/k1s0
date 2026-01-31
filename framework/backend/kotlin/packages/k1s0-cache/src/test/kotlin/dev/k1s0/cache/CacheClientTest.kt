package dev.k1s0.cache

import io.kotest.matchers.shouldNotBe
import org.junit.jupiter.api.Test

class CacheClientTest {

    @Test
    fun `CacheClient can be instantiated with URI`() {
        val client = CacheClient("redis://localhost:6379")

        client shouldNotBe null
    }
}
