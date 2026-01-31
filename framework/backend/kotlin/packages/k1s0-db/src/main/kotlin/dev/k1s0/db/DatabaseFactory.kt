package dev.k1s0.db

import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import io.github.oshai.kotlinlogging.KotlinLogging
import org.jetbrains.exposed.sql.Database

private val logger = KotlinLogging.logger {}

/**
 * Database connection configuration.
 *
 * @property jdbcUrl The JDBC connection URL.
 * @property username The database username.
 * @property passwordFile Path to the file containing the database password.
 * @property maximumPoolSize Maximum number of connections in the pool.
 * @property minimumIdle Minimum number of idle connections.
 */
public data class DatabaseConfig(
    val jdbcUrl: String,
    val username: String,
    val passwordFile: String? = null,
    val password: String? = null,
    val maximumPoolSize: Int = 10,
    val minimumIdle: Int = 2,
)

/**
 * Factory for creating database connections using Exposed and HikariCP.
 *
 * Provides connection pooling and lifecycle management for PostgreSQL databases.
 */
public object DatabaseFactory {

    private var dataSource: HikariDataSource? = null

    /**
     * Initializes the database connection pool.
     *
     * @param config The database connection configuration.
     * @return The configured Exposed [Database] instance.
     */
    public fun initialize(config: DatabaseConfig): Database {
        val hikariConfig = HikariConfig().apply {
            jdbcUrl = config.jdbcUrl
            username = config.username
            password = config.password ?: config.passwordFile?.let {
                dev.k1s0.config.SecretResolver.resolve(it)
            }
            maximumPoolSize = config.maximumPoolSize
            minimumIdle = config.minimumIdle
            isAutoCommit = false
        }

        dataSource = HikariDataSource(hikariConfig)
        logger.info { "Database connection pool initialized: ${config.jdbcUrl}" }
        return Database.connect(dataSource!!)
    }

    /** Closes the connection pool. */
    public fun close() {
        dataSource?.close()
        dataSource = null
        logger.info { "Database connection pool closed" }
    }
}
