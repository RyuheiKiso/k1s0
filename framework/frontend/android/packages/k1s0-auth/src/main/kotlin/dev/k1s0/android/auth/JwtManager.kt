package dev.k1s0.android.auth

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long
import java.util.Base64

private val Context.tokenDataStore: DataStore<Preferences> by preferencesDataStore(name = "k1s0_auth_tokens")

/**
 * Manages JWT token storage, retrieval, and lifecycle.
 *
 * Persists tokens securely using Android DataStore and provides
 * reactive access to the current authentication state.
 *
 * @param context The Android application context.
 */
class JwtManager(private val context: Context) {

    private val accessTokenKey = stringPreferencesKey("access_token")
    private val refreshTokenKey = stringPreferencesKey("refresh_token")

    private val _authState = MutableStateFlow<AuthState>(AuthState.Unauthenticated)

    /** Observable authentication state. */
    val authState: StateFlow<AuthState> = _authState.asStateFlow()

    /** Flow of the current access token, or null if not authenticated. */
    val accessToken: Flow<String?> = context.tokenDataStore.data.map { prefs ->
        prefs[accessTokenKey]
    }

    /**
     * Stores new tokens and updates the auth state to [AuthState.Authenticated].
     *
     * @param accessToken The JWT access token.
     * @param refreshToken The optional refresh token.
     */
    suspend fun setTokens(accessToken: String, refreshToken: String? = null) {
        context.tokenDataStore.edit { prefs ->
            prefs[accessTokenKey] = accessToken
            if (refreshToken != null) {
                prefs[refreshTokenKey] = refreshToken
            }
        }
        val userId = extractClaim(accessToken, "sub")
        _authState.value = AuthState.Authenticated(
            accessToken = accessToken,
            refreshToken = refreshToken,
            userId = userId,
        )
    }

    /**
     * Clears stored tokens and sets the auth state to [AuthState.Unauthenticated].
     */
    suspend fun clearTokens() {
        context.tokenDataStore.edit { prefs ->
            prefs.remove(accessTokenKey)
            prefs.remove(refreshTokenKey)
        }
        _authState.value = AuthState.Unauthenticated
    }

    /**
     * Retrieves the current access token from storage.
     *
     * @return The access token string, or null if not stored.
     */
    suspend fun getAccessToken(): String? {
        return context.tokenDataStore.data.first()[accessTokenKey]
    }

    /**
     * Checks whether the given JWT token is expired.
     *
     * @param token The JWT token to check.
     * @return True if the token is expired or unparsable, false otherwise.
     */
    fun isTokenExpired(token: String): Boolean {
        val exp = extractClaim(token, "exp") ?: return true
        val expSeconds = exp.toLongOrNull() ?: return true
        return System.currentTimeMillis() / 1000 > expSeconds
    }

    /**
     * Extracts a claim value from a JWT token payload.
     *
     * @param token The JWT token.
     * @param claim The claim key to extract.
     * @return The claim value as a string, or null if not found.
     */
    fun extractClaim(token: String, claim: String): String? {
        return try {
            val parts = token.split(".")
            if (parts.size != 3) return null
            val payload = String(Base64.getUrlDecoder().decode(parts[1]))
            val json = Json.parseToJsonElement(payload) as? JsonObject
            json?.get(claim)?.jsonPrimitive?.content
        } catch (_: Exception) {
            null
        }
    }
}
