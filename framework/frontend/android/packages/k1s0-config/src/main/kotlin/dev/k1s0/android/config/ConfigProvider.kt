package dev.k1s0.android.config

import kotlinx.serialization.DeserializationStrategy
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

/**
 * Koin-integrated configuration provider.
 *
 * Provides a convenient way to access typed configuration objects
 * within the Koin dependency injection graph. The [ConfigLoader] is
 * injected automatically.
 *
 * Usage with Koin module:
 * ```kotlin
 * val configModule = module {
 *     single { ConfigLoader(androidContext()) }
 *     single { ConfigProvider(environment = "dev") }
 * }
 * ```
 *
 * @property environment The target environment name.
 */
class ConfigProvider(
    private val environment: String,
) : KoinComponent {

    private val configLoader: ConfigLoader by inject()

    /**
     * Retrieves a typed configuration object for the current environment.
     *
     * @param T The configuration data class type.
     * @param deserializer The kotlinx.serialization deserializer for [T].
     * @return The deserialized configuration object.
     */
    fun <T> get(deserializer: DeserializationStrategy<T>): T {
        return configLoader.load(environment, deserializer)
    }
}
