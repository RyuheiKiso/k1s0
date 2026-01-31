package dev.k1s0.android.observability

/**
 * Interface for crash and error reporting.
 *
 * Implementations can integrate with crash reporting services
 * such as Firebase Crashlytics, Sentry, or custom backends.
 */
interface CrashReporter {

    /**
     * Reports a non-fatal exception.
     *
     * @param throwable The exception to report.
     * @param context Optional contextual information for the report.
     */
    fun reportException(throwable: Throwable, context: Map<String, String> = emptyMap())

    /**
     * Reports a custom error message without an exception.
     *
     * @param message The error message to report.
     * @param context Optional contextual information for the report.
     */
    fun reportError(message: String, context: Map<String, String> = emptyMap())

    /**
     * Sets a user identifier for associating crash reports with users.
     *
     * @param userId The user identifier string, or null to clear.
     */
    fun setUserId(userId: String?)

    /**
     * Adds a breadcrumb for crash report context.
     *
     * @param message A description of the event or action.
     * @param category Optional category for grouping breadcrumbs.
     */
    fun addBreadcrumb(message: String, category: String? = null)
}

/**
 * No-op crash reporter implementation.
 *
 * Used as a default when no crash reporting service is configured.
 */
class NoOpCrashReporter : CrashReporter {
    override fun reportException(throwable: Throwable, context: Map<String, String>) {}
    override fun reportError(message: String, context: Map<String, String>) {}
    override fun setUserId(userId: String?) {}
    override fun addBreadcrumb(message: String, category: String?) {}
}
