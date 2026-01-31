package dev.k1s0.android.http

import kotlinx.serialization.Serializable

/**
 * Sealed class representing the result of an API call.
 *
 * Provides a type-safe way to handle success, error, and exception outcomes
 * from HTTP requests without throwing exceptions in the call site.
 *
 * @param T The type of the response body on success.
 */
sealed class ApiResponse<out T> {

    /**
     * A successful API response.
     *
     * @property data The deserialized response body.
     * @property statusCode The HTTP status code.
     */
    data class Success<T>(
        val data: T,
        val statusCode: Int = 200,
    ) : ApiResponse<T>()

    /**
     * An API error response (HTTP 4xx/5xx).
     *
     * @property statusCode The HTTP status code.
     * @property errorBody The raw error response body, if available.
     * @property errorDetail Parsed error detail following RFC 7807, if available.
     */
    data class Error(
        val statusCode: Int,
        val errorBody: String? = null,
        val errorDetail: ApiErrorDetail? = null,
    ) : ApiResponse<Nothing>()

    /**
     * An exception occurred during the request (network error, timeout, etc.).
     *
     * @property exception The throwable that caused the failure.
     * @property message A human-readable error message.
     */
    data class Exception(
        val exception: Throwable,
        val message: String = exception.message ?: "Unknown error",
    ) : ApiResponse<Nothing>()
}

/**
 * RFC 7807 Problem Details error structure.
 *
 * @property status The HTTP status code.
 * @property title A short human-readable summary.
 * @property detail A detailed human-readable explanation.
 * @property errorCode The k1s0 error code (e.g. "user.not_found").
 * @property traceId The distributed trace ID for correlation.
 */
@Serializable
data class ApiErrorDetail(
    val status: Int,
    val title: String,
    val detail: String? = null,
    val errorCode: String? = null,
    val traceId: String? = null,
)
