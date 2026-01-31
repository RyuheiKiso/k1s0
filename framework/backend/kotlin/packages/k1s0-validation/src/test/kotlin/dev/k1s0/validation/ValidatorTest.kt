package dev.k1s0.validation

import io.kotest.matchers.collections.shouldContain
import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test

class ValidatorTest {

    @Test
    fun `valid input returns success`() {
        val result = Validator.validate {
            field("hello", Rules.notBlank("name"))
            field("hello", Rules.minLength("name", 3))
        }

        result.isValid shouldBe true
        result.errors shouldBe emptyList()
    }

    @Test
    fun `blank field returns error`() {
        val result = Validator.validate {
            field("", Rules.notBlank("name"))
        }

        result.isValid shouldBe false
        result.errors shouldContain "name must not be blank"
    }

    @Test
    fun `multiple validation errors are collected`() {
        val result = Validator.validate {
            field("", Rules.notBlank("name"))
            field(-1, Rules.positiveInt("age"))
        }

        result.isValid shouldBe false
        result.errors.size shouldBe 2
    }

    @Test
    fun `email validation rejects invalid format`() {
        val result = Validator.validate {
            field("not-an-email", Rules.email("email"))
        }

        result.isValid shouldBe false
        result.errors shouldContain "email must be a valid email"
    }

    @Test
    fun `custom check adds error when condition is true`() {
        val result = Validator.validate {
            check(true, "custom error")
        }

        result.isValid shouldBe false
        result.errors shouldContain "custom error"
    }
}
