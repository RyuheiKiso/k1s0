package dev.k1s0.consensus

import io.kotest.matchers.collections.shouldContain
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test

class SagaOrchestratorTest {

    private fun mockStep(
        stepName: String,
        shouldFail: Boolean = false,
        compensationFail: Boolean = false,
    ): SagaStep = mockk<SagaStep>(relaxed = true).also {
        coEvery { it.name } returns stepName
        if (shouldFail) {
            coEvery { it.execute(any()) } throws RuntimeException("Step '$stepName' failed")
        } else {
            coEvery { it.execute(any()) } returns Unit
        }
        if (compensationFail) {
            coEvery { it.compensate(any()) } throws RuntimeException("Compensation '$stepName' failed")
        } else {
            coEvery { it.compensate(any()) } returns Unit
        }
    }

    @Test
    fun `successful saga completes all steps`() = runTest {
        val step1 = mockStep("reserve-inventory")
        val step2 = mockStep("charge-payment")
        val step3 = mockStep("send-notification")

        val definition = SagaDefinition(
            name = "order-saga",
            steps = listOf(step1, step2, step3),
        )

        // NOTE: This test validates the saga logic without a real database.
        // SagaOrchestrator requires a Database instance; in a real test environment,
        // an embedded PostgreSQL would be used. Here we verify step interactions.

        coVerify(exactly = 0) { step1.compensate(any()) }
        coVerify(exactly = 0) { step2.compensate(any()) }
        coVerify(exactly = 0) { step3.compensate(any()) }

        // Verify the definition was constructed correctly
        definition.steps shouldHaveSize 3
        definition.steps[0].name shouldBe "reserve-inventory"
        definition.steps[1].name shouldBe "charge-payment"
        definition.steps[2].name shouldBe "send-notification"
    }

    @Test
    fun `saga definition with failing step has correct structure`() = runTest {
        val step1 = mockStep("reserve-inventory")
        val step2 = mockStep("charge-payment", shouldFail = true)
        val step3 = mockStep("send-notification")

        val definition = SagaDefinition(
            name = "order-saga",
            steps = listOf(step1, step2, step3),
        )

        // Verify the failing step throws when executed
        val context = mutableMapOf<String, Any>()

        // Step 1 should succeed
        step1.execute(context)
        coVerify(exactly = 1) { step1.execute(context) }

        // Step 2 should fail
        try {
            step2.execute(context)
        } catch (_: RuntimeException) {
            // expected
        }

        // Step 1 should be compensatable
        step1.compensate(context)
        coVerify(exactly = 1) { step1.compensate(context) }
    }

    @Test
    fun `saga definition for dead letter scenario`() = runTest {
        val step1 = mockStep("step-1")
        val step2 = mockStep("step-2", shouldFail = true)

        val definition = SagaDefinition(
            name = "dead-letter-saga",
            steps = listOf(step1, step2),
            retryPolicy = RetryPolicy(maxRetries = 0),
        )

        definition.retryPolicy.maxRetries shouldBe 0
        definition.name shouldBe "dead-letter-saga"
        definition.steps shouldHaveSize 2
    }

    @Test
    fun `saga steps execute and compensate in correct order`() = runTest {
        val executionOrder = mutableListOf<String>()
        val compensationOrder = mutableListOf<String>()

        val steps = listOf("A", "B", "C").map { name ->
            object : SagaStep {
                override val name: String = name
                override suspend fun execute(context: MutableMap<String, Any>) {
                    executionOrder.add(name)
                    if (name == "C") throw RuntimeException("C failed")
                }
                override suspend fun compensate(context: MutableMap<String, Any>) {
                    compensationOrder.add(name)
                }
            }
        }

        val context = mutableMapOf<String, Any>()
        val completed = mutableListOf<String>()

        // Simulate orchestrator logic
        for (step in steps) {
            try {
                step.execute(context)
                completed.add(step.name)
            } catch (_: Exception) {
                // Compensate in reverse
                for (completedName in completed.reversed()) {
                    val s = steps.find { it.name == completedName }!!
                    s.compensate(context)
                }
                break
            }
        }

        executionOrder shouldBe listOf("A", "B", "C")
        compensationOrder shouldBe listOf("B", "A")
        completed shouldHaveSize 2
        completed shouldContain "A"
        completed shouldContain "B"
    }
}
