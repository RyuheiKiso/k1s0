package dev.k1s0.config

import io.github.oshai.kotlinlogging.KotlinLogging
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

private val logger = KotlinLogging.logger {}

/**
 * Resolves secrets from file-based references.
 *
 * Following k1s0 conventions, secrets are never hardcoded in configuration files.
 * Instead, configuration values use a `_file` suffix to reference external files
 * containing the actual secret values (e.g., Kubernetes secrets mounted as files).
 *
 * Example YAML:
 * ```yaml
 * database:
 *   password_file: /var/run/secrets/k1s0/db_password
 * ```
 */
public object SecretResolver {

    /**
     * Reads a secret value from a file path.
     *
     * @param filePath The path to the file containing the secret.
     * @return The secret value with leading/trailing whitespace trimmed.
     * @throws IllegalArgumentException if the file does not exist.
     */
    public fun resolve(filePath: String): String {
        val path: Path = Paths.get(filePath)
        require(Files.exists(path)) { "Secret file not found: $filePath" }
        logger.debug { "Resolving secret from $filePath" }
        return Files.readString(path).trim()
    }

    /**
     * Reads a secret value from a file path, returning null if the file does not exist.
     *
     * @param filePath The path to the file containing the secret.
     * @return The secret value, or null if the file does not exist.
     */
    public fun resolveOrNull(filePath: String): String? {
        val path: Path = Paths.get(filePath)
        if (!Files.exists(path)) {
            logger.debug { "Secret file not found, returning null: $filePath" }
            return null
        }
        return Files.readString(path).trim()
    }
}
