package dev.k1s0.consensus

import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

class SagaBuilderTest {

    @Test
    fun `saga DSL builds definition with correct name`() {
        val definition = saga("order-saga") {
            step("step-1") {
                execute { _ -> }
                compensate { _ -> }
            }
        }

        definition.name shouldBe "order-saga"
    }

    @Test
    fun `saga DSL builds steps in order`() {
        val definition = saga("test-saga") {
            step("first") {
                execute { _ -> }
                compensate { _ -> }
            }
            step("second") {
                execute { _ -> }
                compensate { _ -> }
            }
            step("third") {
                execute { _ -> }
                compensate { _ -> }
            }
        }

        definition.steps shouldHaveSize 3
        definition.steps[0].name shouldBe "first"
        definition.steps[1].name shouldBe "second"
        definition.steps[2].name shouldBe "third"
    }

    @Test
    fun `saga DSL applies retry policy`() {
        val definition = saga("retry-saga") {
            step("step-1") {
                execute { _ -> }
                compensate { _ -> }
            }
            retry {
                maxRetries = 5
                initialDelay = 200.milliseconds
                backoff = BackoffStrategy.LINEAR
                maxDelay = 5.seconds
            }
        }

        definition.retryPolicy.maxRetries shouldBe 5
        definition.retryPolicy.initialDelay shouldBe 200.milliseconds
        definition.retryPolicy.backoffStrategy shouldBe BackoffStrategy.LINEAR
        definition.retryPolicy.maxDelay shouldBe 5.seconds
    }

    @Test
    fun `saga DSL uses default retry policy when not specified`() {
        val definition = saga("default-retry") {
            step("step-1") {
                execute { _ -> }
                compensate { _ -> }
            }
        }

        definition.retryPolicy.maxRetries shouldBe 3
        definition.retryPolicy.backoffStrategy shouldBe BackoffStrategy.EXPONENTIAL
    }

    @Test
    fun `step without compensate still builds`() {
        val definition = saga("no-compensate") {
            step("fire-and-forget") {
                execute { ctx -> ctx["key"] = "value" }
            }
        }

        definition.steps shouldHaveSize 1
        definition.steps[0].name shouldBe "fire-and-forget"
    }
}
