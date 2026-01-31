package dev.k1s0.error

import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test

class K1s0ExceptionTest {

    @Test
    fun `toProblemDetails converts exception to RFC 7807 format`() {
        val exception = NotFoundException(
            serviceErrorCode = "user.not_found",
            detail = "User with ID 123 was not found",
            traceId = "trace-abc",
        )

        val problem = exception.toProblemDetails()

        problem.status shouldBe 404
        problem.title shouldBe "NOT_FOUND"
        problem.detail shouldBe "User with ID 123 was not found"
        problem.errorCode shouldBe "user.not_found"
        problem.traceId shouldBe "trace-abc"
    }

    @Test
    fun `ErrorCode maps to correct HTTP and gRPC status`() {
        ErrorCode.NOT_FOUND.httpStatus shouldBe 404
        ErrorCode.UNAUTHENTICATED.httpStatus shouldBe 401
        ErrorCode.INTERNAL.httpStatus shouldBe 500
        ErrorCode.CONFLICT.httpStatus shouldBe 409
    }

    @Test
    fun `ValidationException carries violations`() {
        val exception = ValidationException(
            serviceErrorCode = "order.validation_failed",
            detail = "Validation failed",
            violations = listOf("name is required", "amount must be positive"),
        )

        exception.violations.size shouldBe 2
        exception.errorCode shouldBe ErrorCode.VALIDATION_FAILED
    }
}
