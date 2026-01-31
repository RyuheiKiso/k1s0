package dev.k1s0.android.http

import io.ktor.client.plugins.api.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*

/**
 * Ktor client plugin that attaches an Authorization Bearer token to outgoing requests.
 *
 * The [tokenProvider] function is invoked for each request to retrieve the current token.
 * If the provider returns null, no Authorization header is added.
 *
 * @param tokenProvider A suspend function returning the current bearer token, or null.
 */
fun authInterceptorPlugin(tokenProvider: suspend () -> String?) = createClientPlugin("K1s0AuthInterceptor") {
    onRequest { request, _ ->
        val token = tokenProvider()
        if (token != null) {
            request.header(HttpHeaders.Authorization, "Bearer $token")
        }
    }
}

/**
 * Ktor client plugin that logs HTTP request and response details.
 *
 * Logs the method, URL, status code, and duration of each request.
 *
 * @param logger A logging function that receives formatted log messages.
 */
fun loggingInterceptorPlugin(logger: (String) -> Unit) = createClientPlugin("K1s0LoggingInterceptor") {
    onRequest { request, _ ->
        logger(">> ${request.method.value} ${request.url.buildString()}")
    }

    onResponse { response: HttpResponse ->
        logger("<< ${response.status.value} ${response.request.url.buildString()}")
    }
}
