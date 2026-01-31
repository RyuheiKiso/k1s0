package dev.k1s0.consensus

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.Dispatchers
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction
import java.time.Instant
import java.util.UUID
import kotlin.time.Duration.Companion.milliseconds

private val logger = KotlinLogging.logger {}

/**
 * Orchestrator-based saga execution engine with PostgreSQL persistence.
 *
 * Executes saga steps sequentially. On failure, runs compensation steps
 * in reverse order. Failed sagas are retried according to the retry policy.
 * After exhausting retries, sagas are moved to the dead letter queue.
 *
 * @property database The Exposed database instance.
 * @property sagaConfig Saga configuration.
 * @property metrics Optional saga metrics.
 */
public class SagaOrchestrator(
    private val database: Database,
    private val sagaConfig: SagaConfig = SagaConfig(),
    private val metrics: SagaMetrics? = null,
) {
    private val json = Json { ignoreUnknownKeys = true }

    /**
     * Executes a saga definition.
     *
     * @param definition The saga definition to execute.
     * @param initialContext Optional initial context data.
     * @return The result of the saga execution.
     */
    public suspend fun execute(
        definition: SagaDefinition,
        initialContext: Map<String, Any> = emptyMap(),
    ): SagaResult {
        val sagaId = UUID.randomUUID().toString()
        val context = initialContext.toMutableMap()
        val completedSteps = mutableListOf<String>()

        logger.info { "Starting saga '${definition.name}' (id=$sagaId)" }
        metrics?.sagaStarted(definition.name)

        persistSagaInstance(sagaId, definition.name, SagaStatus.RUNNING, 0, context)

        for ((index, step) in definition.steps.withIndex()) {
            try {
                logger.debug { "Executing step '${step.name}' in saga '$sagaId'" }
                updateSagaStep(sagaId, index, SagaStatus.RUNNING)
                step.execute(context)
                completedSteps.add(step.name)
                logger.debug { "Step '${step.name}' completed in saga '$sagaId'" }
            } catch (e: Exception) {
                logger.error(e) { "Step '${step.name}' failed in saga '$sagaId'" }
                metrics?.sagaStepFailed(definition.name, step.name)

                val compensatedSteps = compensate(sagaId, definition, completedSteps, context)

                val retryCount = getRetryCount(sagaId)
                if (retryCount < definition.retryPolicy.maxRetries) {
                    updateSagaRetry(sagaId, retryCount + 1, e.message)
                    logger.info { "Saga '$sagaId' will be retried (attempt ${retryCount + 1}/${definition.retryPolicy.maxRetries})" }
                } else if (sagaConfig.deadLetterEnabled) {
                    moveToDeadLetter(sagaId, e.message)
                    metrics?.sagaDeadLettered(definition.name)
                    logger.warn { "Saga '$sagaId' moved to dead letter queue after ${definition.retryPolicy.maxRetries} retries" }
                    return SagaResult(
                        sagaId = sagaId,
                        status = SagaStatus.DEAD_LETTER,
                        context = context,
                        error = e.message,
                        completedSteps = completedSteps,
                        compensatedSteps = compensatedSteps,
                    )
                }

                return SagaResult(
                    sagaId = sagaId,
                    status = if (compensatedSteps.isNotEmpty()) SagaStatus.COMPENSATED else SagaStatus.FAILED,
                    context = context,
                    error = e.message,
                    completedSteps = completedSteps,
                    compensatedSteps = compensatedSteps,
                )
            }
        }

        updateSagaStatus(sagaId, SagaStatus.COMPLETED)
        metrics?.sagaCompleted(definition.name)
        logger.info { "Saga '${definition.name}' completed (id=$sagaId)" }

        return SagaResult(
            sagaId = sagaId,
            status = SagaStatus.COMPLETED,
            context = context,
            completedSteps = completedSteps,
        )
    }

    /**
     * Resumes a saga from its persisted state.
     *
     * @param sagaId The ID of the saga to resume.
     * @param definition The saga definition.
     * @return The result of the resumed saga execution.
     */
    public suspend fun resume(sagaId: String, definition: SagaDefinition): SagaResult? {
        val instance = getSagaInstance(sagaId) ?: return null

        if (instance.status == SagaStatus.COMPLETED || instance.status == SagaStatus.DEAD_LETTER) {
            logger.info { "Saga '$sagaId' is already in terminal state: ${instance.status}" }
            return SagaResult(
                sagaId = sagaId,
                status = instance.status,
                context = emptyMap(),
                error = instance.error,
            )
        }

        logger.info { "Resuming saga '$sagaId' from step ${instance.currentStep}" }
        val context = mutableMapOf<String, Any>()
        val completedSteps = mutableListOf<String>()

        // Skip already completed steps
        for (i in 0 until instance.currentStep) {
            completedSteps.add(definition.steps[i].name)
        }

        // Continue from the failed step
        for (i in instance.currentStep until definition.steps.size) {
            val step = definition.steps[i]
            try {
                step.execute(context)
                completedSteps.add(step.name)
            } catch (e: Exception) {
                logger.error(e) { "Resumed step '${step.name}' failed in saga '$sagaId'" }
                val compensated = compensate(sagaId, definition, completedSteps, context)
                return SagaResult(
                    sagaId = sagaId,
                    status = SagaStatus.COMPENSATED,
                    context = context,
                    error = e.message,
                    completedSteps = completedSteps,
                    compensatedSteps = compensated,
                )
            }
        }

        updateSagaStatus(sagaId, SagaStatus.COMPLETED)
        return SagaResult(
            sagaId = sagaId,
            status = SagaStatus.COMPLETED,
            context = context,
            completedSteps = completedSteps,
        )
    }

    /**
     * Retrieves all sagas in the dead letter queue.
     *
     * @return List of saga instances in dead letter state.
     */
    public suspend fun deadLetters(): List<SagaInstance> = newSuspendedTransaction(Dispatchers.IO, database) {
        val sql = """
            SELECT saga_id, saga_name, status, current_step, context, error, retry_count, created_at, updated_at
            FROM ${sagaConfig.tableName}
            WHERE status = 'DEAD_LETTER'
            ORDER BY updated_at DESC
        """.trimIndent()

        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        val rs = stmt.executeQuery().resultSet

        val results = mutableListOf<SagaInstance>()
        while (rs.next()) {
            results.add(
                SagaInstance(
                    sagaId = rs.getString("saga_id"),
                    sagaName = rs.getString("saga_name"),
                    status = SagaStatus.valueOf(rs.getString("status")),
                    currentStep = rs.getInt("current_step"),
                    context = rs.getString("context"),
                    error = rs.getString("error"),
                    retryCount = rs.getInt("retry_count"),
                    createdAt = Instant.parse(rs.getString("created_at")),
                    updatedAt = Instant.parse(rs.getString("updated_at")),
                ),
            )
        }
        results
    }

    private suspend fun compensate(
        sagaId: String,
        definition: SagaDefinition,
        completedSteps: List<String>,
        context: MutableMap<String, Any>,
    ): List<String> {
        val compensatedSteps = mutableListOf<String>()
        updateSagaStatus(sagaId, SagaStatus.COMPENSATING)

        // Compensate in reverse order
        for (stepName in completedSteps.reversed()) {
            val step = definition.steps.find { it.name == stepName } ?: continue
            try {
                logger.debug { "Compensating step '${step.name}' in saga '$sagaId'" }
                step.compensate(context)
                compensatedSteps.add(step.name)
            } catch (e: Exception) {
                logger.error(e) { "Compensation failed for step '${step.name}' in saga '$sagaId'" }
                metrics?.sagaCompensationFailed(definition.name, step.name)
                updateSagaStatus(sagaId, SagaStatus.FAILED)
                throw ConsensusError.CompensationFailed(sagaId, step.name, cause = e)
            }
        }

        updateSagaStatus(sagaId, SagaStatus.COMPENSATED)
        return compensatedSteps
    }

    private suspend fun persistSagaInstance(
        sagaId: String,
        sagaName: String,
        status: SagaStatus,
        currentStep: Int,
        context: Map<String, Any>,
    ) = newSuspendedTransaction(Dispatchers.IO, database) {
        val now = Instant.now().toString()
        val sql = """
            INSERT INTO ${sagaConfig.tableName} (saga_id, saga_name, status, current_step, context, retry_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 0, ?, ?)
        """.trimIndent()

        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        stmt[1] = sagaId
        stmt[2] = sagaName
        stmt[3] = status.name
        stmt[4] = currentStep
        stmt[5] = "{}"
        stmt[6] = now
        stmt[7] = now
        stmt.executeUpdate()
    }

    private suspend fun updateSagaStatus(sagaId: String, status: SagaStatus) =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val sql = "UPDATE ${sagaConfig.tableName} SET status = ?, updated_at = ? WHERE saga_id = ?"
            val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
            val stmt = conn.prepareStatement(sql, false)
            stmt[1] = status.name
            stmt[2] = Instant.now().toString()
            stmt[3] = sagaId
            stmt.executeUpdate()
        }

    private suspend fun updateSagaStep(sagaId: String, step: Int, status: SagaStatus) =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val sql = "UPDATE ${sagaConfig.tableName} SET current_step = ?, status = ?, updated_at = ? WHERE saga_id = ?"
            val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
            val stmt = conn.prepareStatement(sql, false)
            stmt[1] = step
            stmt[2] = status.name
            stmt[3] = Instant.now().toString()
            stmt[4] = sagaId
            stmt.executeUpdate()
        }

    private suspend fun updateSagaRetry(sagaId: String, retryCount: Int, error: String?) =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val sql = "UPDATE ${sagaConfig.tableName} SET retry_count = ?, error = ?, updated_at = ? WHERE saga_id = ?"
            val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
            val stmt = conn.prepareStatement(sql, false)
            stmt[1] = retryCount
            stmt[2] = error
            stmt[3] = Instant.now().toString()
            stmt[4] = sagaId
            stmt.executeUpdate()
        }

    private suspend fun moveToDeadLetter(sagaId: String, error: String?) =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val sql = "UPDATE ${sagaConfig.tableName} SET status = 'DEAD_LETTER', error = ?, updated_at = ? WHERE saga_id = ?"
            val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
            val stmt = conn.prepareStatement(sql, false)
            stmt[1] = error
            stmt[2] = Instant.now().toString()
            stmt[3] = sagaId
            stmt.executeUpdate()
        }

    private suspend fun getRetryCount(sagaId: String): Int = newSuspendedTransaction(Dispatchers.IO, database) {
        val sql = "SELECT retry_count FROM ${sagaConfig.tableName} WHERE saga_id = ?"
        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        stmt[1] = sagaId
        val rs = stmt.executeQuery().resultSet
        if (rs.next()) rs.getInt("retry_count") else 0
    }

    private suspend fun getSagaInstance(sagaId: String): SagaInstance? = newSuspendedTransaction(Dispatchers.IO, database) {
        val sql = """
            SELECT saga_id, saga_name, status, current_step, context, error, retry_count, created_at, updated_at
            FROM ${sagaConfig.tableName}
            WHERE saga_id = ?
        """.trimIndent()

        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        stmt[1] = sagaId
        val rs = stmt.executeQuery().resultSet

        if (rs.next()) {
            SagaInstance(
                sagaId = rs.getString("saga_id"),
                sagaName = rs.getString("saga_name"),
                status = SagaStatus.valueOf(rs.getString("status")),
                currentStep = rs.getInt("current_step"),
                context = rs.getString("context"),
                error = rs.getString("error"),
                retryCount = rs.getInt("retry_count"),
                createdAt = Instant.parse(rs.getString("created_at")),
                updatedAt = Instant.parse(rs.getString("updated_at")),
            )
        } else {
            null
        }
    }
}
