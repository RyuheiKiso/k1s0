package dev.k1s0.android.http

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.json.Json

/**
 * Pre-configured Ktor HTTP client wrapper for k1s0 Android applications.
 *
 * Provides a configured [HttpClient] with JSON serialization, content negotiation,
 * default timeouts, and optional auth/logging interceptors.
 *
 * @param baseUrl The base URL for all API requests.
 * @param json The [Json] instance for serialization configuration.
 * @param tokenProvider Optional suspend function to provide auth tokens.
 * @param logger Optional logging function for request/response logging.
 * @param connectTimeoutMs Connection timeout in milliseconds. Defaults to 10000.
 * @param requestTimeoutMs Request timeout in milliseconds. Defaults to 30000.
 */
class K1s0HttpClient(
    private val baseUrl: String,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        isLenient = true
        encodeDefaults = true
    },
    tokenProvider: (suspend () -> String?)? = null,
    logger: ((String) -> Unit)? = null,
    connectTimeoutMs: Long = 10_000L,
    requestTimeoutMs: Long = 30_000L,
) {

    /** The underlying Ktor [HttpClient] instance. */
    val client: HttpClient = HttpClient(OkHttp) {
        install(ContentNegotiation) {
            json(this@K1s0HttpClient.json)
        }

        install(HttpTimeout) {
            connectTimeoutMillis = connectTimeoutMs
            requestTimeoutMillis = requestTimeoutMs
        }

        defaultRequest {
            url(baseUrl)
            contentType(ContentType.Application.Json)
        }

        if (tokenProvider != null) {
            install(authInterceptorPlugin(tokenProvider))
        }

        if (logger != null) {
            install(loggingInterceptorPlugin(logger))
        }
    }

    /**
     * Performs a GET request and wraps the result in an [ApiResponse].
     *
     * @param T The expected response body type.
     * @param path The URL path relative to [baseUrl].
     * @param block Optional request configuration block.
     * @return An [ApiResponse] wrapping the result.
     */
    suspend inline fun <reified T> get(
        path: String,
        block: HttpRequestBuilder.() -> Unit = {},
    ): ApiResponse<T> = safeRequest {
        client.get(path, block)
    }

    /**
     * Performs a POST request and wraps the result in an [ApiResponse].
     *
     * @param T The expected response body type.
     * @param path The URL path relative to [baseUrl].
     * @param block Optional request configuration block.
     * @return An [ApiResponse] wrapping the result.
     */
    suspend inline fun <reified T> post(
        path: String,
        block: HttpRequestBuilder.() -> Unit = {},
    ): ApiResponse<T> = safeRequest {
        client.post(path, block)
    }

    /**
     * Performs a PUT request and wraps the result in an [ApiResponse].
     *
     * @param T The expected response body type.
     * @param path The URL path relative to [baseUrl].
     * @param block Optional request configuration block.
     * @return An [ApiResponse] wrapping the result.
     */
    suspend inline fun <reified T> put(
        path: String,
        block: HttpRequestBuilder.() -> Unit = {},
    ): ApiResponse<T> = safeRequest {
        client.put(path, block)
    }

    /**
     * Performs a DELETE request and wraps the result in an [ApiResponse].
     *
     * @param T The expected response body type.
     * @param path The URL path relative to [baseUrl].
     * @param block Optional request configuration block.
     * @return An [ApiResponse] wrapping the result.
     */
    suspend inline fun <reified T> delete(
        path: String,
        block: HttpRequestBuilder.() -> Unit = {},
    ): ApiResponse<T> = safeRequest {
        client.delete(path, block)
    }

    /**
     * Wraps an HTTP call in a try-catch, converting the response to [ApiResponse].
     */
    suspend inline fun <reified T> safeRequest(
        crossinline call: suspend () -> HttpResponse,
    ): ApiResponse<T> {
        return try {
            val response = call()
            if (response.status.isSuccess()) {
                ApiResponse.Success(
                    data = response.body<T>(),
                    statusCode = response.status.value,
                )
            } else {
                val body = response.bodyAsText()
                val detail = try {
                    json.decodeFromString<ApiErrorDetail>(body)
                } catch (_: Exception) {
                    null
                }
                ApiResponse.Error(
                    statusCode = response.status.value,
                    errorBody = body,
                    errorDetail = detail,
                )
            }
        } catch (e: Exception) {
            ApiResponse.Exception(exception = e)
        }
    }

    /** Closes the underlying HTTP client and releases resources. */
    fun close() {
        client.close()
    }
}
