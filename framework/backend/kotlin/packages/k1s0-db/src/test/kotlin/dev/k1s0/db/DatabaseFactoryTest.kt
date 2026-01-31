package dev.k1s0.db

import io.kotest.matchers.shouldNotBe
import org.junit.jupiter.api.Test

class DatabaseFactoryTest {

    @Test
    fun `DatabaseConfig can be constructed with defaults`() {
        val config = DatabaseConfig(
            jdbcUrl = "jdbc:postgresql://localhost:5432/test",
            username = "test",
            password = "test",
        )

        config.maximumPoolSize shouldNotBe 0
        config.minimumIdle shouldNotBe 0
    }
}
