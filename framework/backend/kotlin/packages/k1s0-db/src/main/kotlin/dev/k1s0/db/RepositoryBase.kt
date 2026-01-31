package dev.k1s0.db

/**
 * Base class for repository implementations using Exposed.
 *
 * Provides access to the [TransactionManager] for executing
 * database operations within transactions.
 */
public abstract class RepositoryBase {

    /**
     * Executes the given block within a database transaction.
     *
     * @param block The suspending block to execute.
     * @return The result of the block.
     */
    protected suspend fun <T> dbQuery(block: suspend () -> T): T =
        TransactionManager.transaction(block)
}
