package dev.k1s0.consensus

import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

/**
 * A single step in a saga workflow.
 */
public interface SagaStep {
    /** The unique name of this step. */
    public val name: String

    /**
     * Executes the forward action of this step.
     *
     * @param context Mutable context shared across saga steps.
     */
    public suspend fun execute(context: MutableMap<String, Any>)

    /**
     * Compensates (rolls back) the effects of this step.
     *
     * @param context The context at the time of compensation.
     */
    public suspend fun compensate(context: MutableMap<String, Any>)
}

/**
 * Definition of a complete saga workflow.
 *
 * @property name The name of the saga.
 * @property steps The ordered list of steps to execute.
 * @property retryPolicy The retry policy for failed steps.
 */
public data class SagaDefinition(
    val name: String,
    val steps: List<SagaStep>,
    val retryPolicy: RetryPolicy = RetryPolicy(),
)

/**
 * Retry policy for saga step execution.
 *
 * @property maxRetries Maximum number of retries before giving up.
 * @property initialDelay Initial delay before the first retry.
 * @property backoffStrategy The backoff strategy between retries.
 * @property maxDelay Maximum delay between retries.
 */
public data class RetryPolicy(
    val maxRetries: Int = 3,
    val initialDelay: Duration = 100.milliseconds,
    val backoffStrategy: BackoffStrategy = BackoffStrategy.EXPONENTIAL,
    val maxDelay: Duration = 10_000.milliseconds,
)

/**
 * Backoff strategy for retry delays.
 */
public enum class BackoffStrategy {
    /** Constant delay between retries. */
    FIXED,

    /** Linearly increasing delay between retries. */
    LINEAR,

    /** Exponentially increasing delay between retries. */
    EXPONENTIAL,
}

/**
 * DSL builder for constructing saga definitions.
 *
 * Usage:
 * ```kotlin
 * val saga = saga("order-saga") {
 *     step("reserve-inventory") {
 *         execute { ctx -> ... }
 *         compensate { ctx -> ... }
 *     }
 *     step("charge-payment") {
 *         execute { ctx -> ... }
 *         compensate { ctx -> ... }
 *     }
 *     retry {
 *         maxRetries = 3
 *         backoff = BackoffStrategy.EXPONENTIAL
 *     }
 * }
 * ```
 */
public class SagaBuilder(private val name: String) {

    private val steps = mutableListOf<SagaStep>()
    private var retryPolicy = RetryPolicy()

    /**
     * Adds a step to the saga.
     */
    public fun step(stepName: String, block: StepBuilder.() -> Unit) {
        val builder = StepBuilder(stepName)
        builder.block()
        steps.add(builder.build())
    }

    /**
     * Configures the retry policy.
     */
    public fun retry(block: RetryPolicyBuilder.() -> Unit) {
        val builder = RetryPolicyBuilder()
        builder.block()
        retryPolicy = builder.build()
    }

    public fun build(): SagaDefinition = SagaDefinition(
        name = name,
        steps = steps.toList(),
        retryPolicy = retryPolicy,
    )
}

/**
 * Builder for a single saga step.
 */
public class StepBuilder(private val name: String) {

    private var executeAction: (suspend (MutableMap<String, Any>) -> Unit)? = null
    private var compensateAction: (suspend (MutableMap<String, Any>) -> Unit)? = null

    /** Sets the forward execution action. */
    public fun execute(action: suspend (MutableMap<String, Any>) -> Unit) {
        executeAction = action
    }

    /** Sets the compensation action. */
    public fun compensate(action: suspend (MutableMap<String, Any>) -> Unit) {
        compensateAction = action
    }

    public fun build(): SagaStep = object : SagaStep {
        override val name: String = this@StepBuilder.name
        override suspend fun execute(context: MutableMap<String, Any>) {
            executeAction?.invoke(context)
        }
        override suspend fun compensate(context: MutableMap<String, Any>) {
            compensateAction?.invoke(context)
        }
    }
}

/**
 * Builder for retry policy configuration.
 */
public class RetryPolicyBuilder {
    public var maxRetries: Int = 3
    public var initialDelay: Duration = 100.milliseconds
    public var backoff: BackoffStrategy = BackoffStrategy.EXPONENTIAL
    public var maxDelay: Duration = 10_000.milliseconds

    public fun build(): RetryPolicy = RetryPolicy(
        maxRetries = maxRetries,
        initialDelay = initialDelay,
        backoffStrategy = backoff,
        maxDelay = maxDelay,
    )
}

/**
 * Top-level DSL function for building a saga definition.
 */
public fun saga(name: String, block: SagaBuilder.() -> Unit): SagaDefinition {
    val builder = SagaBuilder(name)
    builder.block()
    return builder.build()
}
