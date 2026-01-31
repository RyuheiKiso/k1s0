package dev.k1s0.db

import kotlinx.coroutines.Dispatchers
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction

/**
 * Provides coroutine-friendly transaction management for Exposed.
 *
 * Wraps database operations in a suspended transaction that runs
 * on the IO dispatcher.
 */
public object TransactionManager {

    /**
     * Executes the given block within a database transaction.
     *
     * @param block The suspending block to execute within the transaction.
     * @return The result of the block.
     */
    public suspend fun <T> transaction(block: suspend () -> T): T =
        newSuspendedTransaction(Dispatchers.IO) { block() }
}
