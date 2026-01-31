package dev.k1s0.config

import io.kotest.matchers.shouldBe
import kotlinx.serialization.Serializable
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.nio.file.Files
import java.nio.file.Path

class ConfigLoaderTest {

    @Serializable
    data class AppConfig(
        val server: ServerConfig = ServerConfig(),
    )

    @Serializable
    data class ServerConfig(
        val port: Int = 8080,
        val host: String = "0.0.0.0",
    )

    @Test
    fun `load reads default yaml config`(@TempDir tempDir: Path) {
        val configDir = tempDir.resolve("config")
        Files.createDirectories(configDir)
        Files.writeString(
            configDir.resolve("default.yaml"),
            """
            server:
              port: 9090
              host: localhost
            """.trimIndent(),
        )

        val config = ConfigLoader.load(
            AppConfig.serializer(),
            configDir = configDir.toString(),
        )

        config.server.port shouldBe 9090
        config.server.host shouldBe "localhost"
    }

    @Test
    fun `SecretResolver reads secret from file`(@TempDir tempDir: Path) {
        val secretFile = tempDir.resolve("db_password")
        Files.writeString(secretFile, "  s3cret!  \n")

        val secret = SecretResolver.resolve(secretFile.toString())

        secret shouldBe "s3cret!"
    }

    @Test
    fun `SecretResolver returns null for missing file`() {
        val result = SecretResolver.resolveOrNull("/nonexistent/path/secret")

        result shouldBe null
    }
}
