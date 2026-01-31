package dev.k1s0.android.http

import kotlinx.serialization.json.Json
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class K1s0HttpClientTest {

    @Test
    fun `ApiResponse Success holds data and status code`() {
        val response = ApiResponse.Success(data = "hello", statusCode = 200)
        assertEquals("hello", response.data)
        assertEquals(200, response.statusCode)
    }

    @Test
    fun `ApiResponse Error holds status code and body`() {
        val response = ApiResponse.Error(statusCode = 404, errorBody = "not found")
        assertEquals(404, response.statusCode)
        assertEquals("not found", response.errorBody)
    }

    @Test
    fun `ApiResponse Exception holds throwable`() {
        val ex = RuntimeException("network error")
        val response = ApiResponse.Exception(exception = ex)
        assertEquals("network error", response.message)
        assertEquals(ex, response.exception)
    }

    @Test
    fun `ApiErrorDetail deserializes from JSON`() {
        val json = Json { ignoreUnknownKeys = true }
        val raw = """
            {
                "status": 404,
                "title": "Not Found",
                "detail": "User not found",
                "errorCode": "user.not_found",
                "traceId": "abc123"
            }
        """.trimIndent()

        val detail = json.decodeFromString<ApiErrorDetail>(raw)
        assertEquals(404, detail.status)
        assertEquals("Not Found", detail.title)
        assertEquals("User not found", detail.detail)
        assertEquals("user.not_found", detail.errorCode)
        assertEquals("abc123", detail.traceId)
    }

    @Test
    fun `ApiErrorDetail deserializes with optional fields missing`() {
        val json = Json { ignoreUnknownKeys = true }
        val raw = """{"status": 500, "title": "Internal Error"}"""

        val detail = json.decodeFromString<ApiErrorDetail>(raw)
        assertEquals(500, detail.status)
        assertNull(detail.detail)
        assertNull(detail.errorCode)
    }

    @Test
    fun `ApiResponse types are exhaustive in when expression`() {
        val responses: List<ApiResponse<String>> = listOf(
            ApiResponse.Success("ok"),
            ApiResponse.Error(400),
            ApiResponse.Exception(RuntimeException()),
        )

        responses.forEach { response ->
            val result = when (response) {
                is ApiResponse.Success -> "success"
                is ApiResponse.Error -> "error"
                is ApiResponse.Exception -> "exception"
            }
            assertNotNull(result)
        }
    }
}
