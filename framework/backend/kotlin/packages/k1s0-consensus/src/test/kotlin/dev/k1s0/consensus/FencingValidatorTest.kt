package dev.k1s0.consensus

import io.kotest.assertions.throwables.shouldThrow
import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class FencingValidatorTest {

    private lateinit var validator: FencingValidator

    @BeforeEach
    fun setUp() {
        validator = FencingValidator()
    }

    @Test
    fun `validate accepts monotonically increasing tokens`() {
        validator.validate(1L) shouldBe true
        validator.validate(2L) shouldBe true
        validator.validate(5L) shouldBe true
        validator.validate(100L) shouldBe true
    }

    @Test
    fun `validate accepts equal token (idempotent)`() {
        validator.validate(5L) shouldBe true
        validator.validate(5L) shouldBe true
    }

    @Test
    fun `validate rejects stale token`() {
        validator.validate(10L) shouldBe true

        val error = shouldThrow<ConsensusError.FenceTokenViolation> {
            validator.validate(5L)
        }
        error.expectedMinimum shouldBe 10L
        error.actual shouldBe 5L
    }

    @Test
    fun `highestToken returns the highest seen token`() {
        validator.highestToken() shouldBe 0L
        validator.validate(3L)
        validator.highestToken() shouldBe 3L
        validator.validate(7L)
        validator.highestToken() shouldBe 7L
    }

    @Test
    fun `reset clears the highest token`() {
        validator.validate(10L)
        validator.highestToken() shouldBe 10L
        validator.reset()
        validator.highestToken() shouldBe 0L
        validator.validate(1L) shouldBe true
    }

    @Test
    fun `validate accepts first token of zero`() {
        validator.validate(0L) shouldBe true
        validator.highestToken() shouldBe 0L
    }

    @Test
    fun `validate rejects negative token after positive`() {
        validator.validate(1L) shouldBe true

        shouldThrow<ConsensusError.FenceTokenViolation> {
            validator.validate(-1L)
        }
    }
}
