package dev.k1s0.android.config

import android.content.Context
import com.charleskorn.kaml.Yaml
import kotlinx.serialization.DeserializationStrategy
import java.io.InputStream

/**
 * Loads YAML configuration files from Android assets.
 *
 * Configuration files are expected under the `config/` directory in assets,
 * following the k1s0 convention of environment-based files:
 * `config/default.yaml`, `config/dev.yaml`, `config/stg.yaml`, `config/prod.yaml`.
 *
 * The loader merges `default.yaml` with the environment-specific file,
 * with environment values taking precedence.
 *
 * @param context The Android application context for asset access.
 * @param yaml The YAML parser instance. Defaults to the standard Yaml instance.
 */
class ConfigLoader(
    private val context: Context,
    private val yaml: Yaml = Yaml.default,
) {

    /**
     * Loads and deserializes configuration for the given environment.
     *
     * Reads `config/default.yaml` as the base, then overlays values from
     * `config/{environment}.yaml` if it exists.
     *
     * @param T The configuration data class type.
     * @param environment The target environment name (e.g. "dev", "stg", "prod").
     * @param deserializer The kotlinx.serialization deserializer for [T].
     * @return The deserialized configuration object.
     * @throws ConfigLoadException if the default config file cannot be read or parsed.
     */
    fun <T> load(
        environment: String,
        deserializer: DeserializationStrategy<T>,
    ): T {
        val defaultYaml = readAsset("config/default.yaml")
            ?: throw ConfigLoadException("config/default.yaml not found in assets")

        val envYaml = readAsset("config/$environment.yaml")

        val mergedYaml = if (envYaml != null) {
            mergeYamlStrings(defaultYaml, envYaml)
        } else {
            defaultYaml
        }

        return try {
            yaml.decodeFromString(deserializer, mergedYaml)
        } catch (e: Exception) {
            throw ConfigLoadException("Failed to parse configuration: ${e.message}", e)
        }
    }

    /**
     * Reads a raw string from an Android asset file.
     *
     * @param path The asset file path.
     * @return The file contents as a string, or null if the file does not exist.
     */
    private fun readAsset(path: String): String? {
        return try {
            val stream: InputStream = context.assets.open(path)
            stream.bufferedReader().use { it.readText() }
        } catch (_: Exception) {
            null
        }
    }

    /**
     * Merges two YAML strings by simple line-based concatenation.
     *
     * The environment YAML is appended after the default YAML,
     * so duplicate keys at the top level will be overridden by the
     * later (environment) values during deserialization.
     */
    private fun mergeYamlStrings(base: String, overlay: String): String {
        return "$base\n$overlay"
    }
}

/**
 * Exception thrown when configuration loading fails.
 *
 * @param message A description of the failure.
 * @param cause The underlying cause, if any.
 */
class ConfigLoadException(
    message: String,
    cause: Throwable? = null,
) : RuntimeException(message, cause)
