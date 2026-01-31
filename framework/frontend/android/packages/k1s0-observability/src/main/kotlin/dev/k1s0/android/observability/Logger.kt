package dev.k1s0.android.observability

import android.util.Log
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

/**
 * Log severity levels.
 */
enum class LogLevel {
    VERBOSE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

/**
 * Structured logger for k1s0 Android applications.
 *
 * Produces structured log entries with consistent formatting,
 * contextual metadata, and severity levels. Logs are output
 * to Android Logcat with optional structured JSON formatting.
 *
 * @param tag The default Logcat tag. Defaults to "k1s0".
 * @param minLevel The minimum log level to output. Messages below this level are discarded.
 * @param context Persistent contextual fields added to every log entry.
 */
class Logger(
    private val tag: String = "k1s0",
    private val minLevel: LogLevel = LogLevel.DEBUG,
    private val context: Map<String, String> = emptyMap(),
) {

    private val json = Json { encodeDefaults = true }

    /**
     * Creates a child logger with additional context fields.
     *
     * @param additionalContext The extra context fields to merge.
     * @return A new [Logger] instance with the merged context.
     */
    fun withContext(additionalContext: Map<String, String>): Logger {
        return Logger(
            tag = tag,
            minLevel = minLevel,
            context = context + additionalContext,
        )
    }

    /** Logs a message at VERBOSE level. */
    fun verbose(message: String, fields: Map<String, String> = emptyMap()) {
        log(LogLevel.VERBOSE, message, fields)
    }

    /** Logs a message at DEBUG level. */
    fun debug(message: String, fields: Map<String, String> = emptyMap()) {
        log(LogLevel.DEBUG, message, fields)
    }

    /** Logs a message at INFO level. */
    fun info(message: String, fields: Map<String, String> = emptyMap()) {
        log(LogLevel.INFO, message, fields)
    }

    /** Logs a message at WARN level. */
    fun warn(message: String, fields: Map<String, String> = emptyMap()) {
        log(LogLevel.WARN, message, fields)
    }

    /** Logs a message at ERROR level. */
    fun error(message: String, throwable: Throwable? = null, fields: Map<String, String> = emptyMap()) {
        log(LogLevel.ERROR, message, fields, throwable)
    }

    private fun log(
        level: LogLevel,
        message: String,
        fields: Map<String, String>,
        throwable: Throwable? = null,
    ) {
        if (level.ordinal < minLevel.ordinal) return

        val allFields = context + fields
        val structured = if (allFields.isNotEmpty()) {
            val entry = LogEntry(
                level = level.name,
                message = message,
                fields = allFields,
            )
            json.encodeToString(entry)
        } else {
            message
        }

        when (level) {
            LogLevel.VERBOSE -> Log.v(tag, structured, throwable)
            LogLevel.DEBUG -> Log.d(tag, structured, throwable)
            LogLevel.INFO -> Log.i(tag, structured, throwable)
            LogLevel.WARN -> Log.w(tag, structured, throwable)
            LogLevel.ERROR -> Log.e(tag, structured, throwable)
        }
    }
}

/**
 * Internal structured log entry for JSON serialization.
 */
@Serializable
internal data class LogEntry(
    val level: String,
    val message: String,
    val fields: Map<String, String> = emptyMap(),
)
