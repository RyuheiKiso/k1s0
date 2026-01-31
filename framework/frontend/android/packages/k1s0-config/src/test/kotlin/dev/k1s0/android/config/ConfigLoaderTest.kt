package dev.k1s0.android.config

import android.content.Context
import android.content.res.AssetManager
import io.mockk.every
import io.mockk.mockk
import kotlinx.serialization.Serializable
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Test
import java.io.ByteArrayInputStream
import java.io.FileNotFoundException

@Serializable
data class TestConfig(
    val app_name: String = "",
    val port: Int = 8080,
)

class ConfigLoaderTest {

    private fun mockContext(files: Map<String, String>): Context {
        val assetManager = mockk<AssetManager>()
        val context = mockk<Context>()
        every { context.assets } returns assetManager

        files.forEach { (path, content) ->
            every { assetManager.open(path) } returns ByteArrayInputStream(content.toByteArray())
        }

        // Default: throw for unknown files
        every { assetManager.open(match { it !in files }) } throws FileNotFoundException()

        return context
    }

    @Test
    fun `loads default config successfully`() {
        val yaml = """
            app_name: test-app
            port: 3000
        """.trimIndent()

        val context = mockContext(mapOf("config/default.yaml" to yaml))
        val loader = ConfigLoader(context)

        val config = loader.load("dev", TestConfig.serializer())
        assertEquals("test-app", config.app_name)
        assertEquals(3000, config.port)
    }

    @Test
    fun `throws when default config missing`() {
        val context = mockContext(emptyMap())
        val loader = ConfigLoader(context)

        assertThrows(ConfigLoadException::class.java) {
            loader.load("dev", TestConfig.serializer())
        }
    }

    @Test
    fun `environment config overrides default`() {
        val defaultYaml = """
            app_name: default-app
            port: 8080
        """.trimIndent()
        val devYaml = """
            app_name: dev-app
        """.trimIndent()

        val context = mockContext(
            mapOf(
                "config/default.yaml" to defaultYaml,
                "config/dev.yaml" to devYaml,
            )
        )
        val loader = ConfigLoader(context)

        val config = loader.load("dev", TestConfig.serializer())
        assertEquals("dev-app", config.app_name)
    }
}
