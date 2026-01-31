package dev.k1s0.config

import com.charleskorn.kaml.Yaml
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.serialization.DeserializationStrategy
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

private val logger = KotlinLogging.logger {}

/**
 * Loads YAML configuration files following k1s0 conventions.
 *
 * Configuration files are loaded from a `config/` directory with environment-based
 * overlays. The loading order is:
 * 1. `config/default.yaml` (base configuration)
 * 2. `config/{env}.yaml` (environment-specific overrides)
 *
 * Environment variables are never used for configuration. All configuration
 * must come from YAML files.
 */
public object ConfigLoader {

    private val yaml = Yaml.default

    /**
     * Loads configuration from YAML files.
     *
     * @param T The configuration data class type (must be @Serializable).
     * @param deserializer The serialization strategy for type T.
     * @param env The environment name (default, dev, stg, prod).
     * @param configDir The directory containing config files. Defaults to "config".
     * @return The deserialized configuration object.
     */
    public fun <T> load(
        deserializer: DeserializationStrategy<T>,
        env: String = "default",
        configDir: String = "config",
    ): T {
        val basePath = Paths.get(configDir)
        val defaultFile = basePath.resolve("default.yaml")
        val envFile = basePath.resolve("$env.yaml")

        val content = buildString {
            if (Files.exists(defaultFile)) {
                logger.info { "Loading default config from $defaultFile" }
                append(Files.readString(defaultFile))
            }
            if (env != "default" && Files.exists(envFile)) {
                logger.info { "Loading $env config from $envFile" }
                append("\n")
                append(Files.readString(envFile))
            }
        }

        return yaml.decodeFromString(deserializer, content)
    }

    /**
     * Loads configuration from a specific file path.
     *
     * @param T The configuration data class type.
     * @param deserializer The serialization strategy for type T.
     * @param path The path to the YAML file.
     * @return The deserialized configuration object.
     */
    public fun <T> loadFrom(
        deserializer: DeserializationStrategy<T>,
        path: Path,
    ): T {
        val content = Files.readString(path)
        return yaml.decodeFromString(deserializer, content)
    }
}
