package dev.k1s0.observability

import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test

class LoggingSetupTest {

    @Test
    fun `initialize sets system properties`() {
        LoggingSetup.initialize("test-service", "dev")

        System.getProperty("SERVICE_NAME") shouldBe "test-service"
        System.getProperty("SERVICE_ENV") shouldBe "dev"
    }
}
