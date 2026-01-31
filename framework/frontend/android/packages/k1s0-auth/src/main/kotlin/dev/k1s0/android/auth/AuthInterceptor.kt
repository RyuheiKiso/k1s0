package dev.k1s0.android.auth

import io.ktor.client.plugins.api.*
import io.ktor.client.request.*
import io.ktor.http.*

/**
 * Ktor client plugin that automatically attaches JWT auth tokens to requests.
 *
 * Retrieves the current access token from [JwtManager] and adds it
 * as a Bearer token in the Authorization header.
 *
 * @param jwtManager The [JwtManager] instance providing the current token.
 * @return A Ktor client plugin configuration.
 */
fun authPlugin(jwtManager: JwtManager) = createClientPlugin("K1s0AuthPlugin") {
    onRequest { request, _ ->
        val token = jwtManager.getAccessToken()
        if (token != null && !jwtManager.isTokenExpired(token)) {
            request.header(HttpHeaders.Authorization, "Bearer $token")
        }
    }
}
