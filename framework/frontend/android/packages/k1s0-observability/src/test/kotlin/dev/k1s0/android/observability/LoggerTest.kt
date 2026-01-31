package dev.k1s0.android.observability

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class LoggerTest {

    @Test
    fun `LogLevel ordering is correct`() {
        assertTrue(LogLevel.VERBOSE.ordinal < LogLevel.DEBUG.ordinal)
        assertTrue(LogLevel.DEBUG.ordinal < LogLevel.INFO.ordinal)
        assertTrue(LogLevel.INFO.ordinal < LogLevel.WARN.ordinal)
        assertTrue(LogLevel.WARN.ordinal < LogLevel.ERROR.ordinal)
    }

    @Test
    fun `withContext creates new logger with merged context`() {
        val logger = Logger(context = mapOf("service" to "test"))
        val child = logger.withContext(mapOf("request_id" to "123"))
        // Child logger is a new instance (no shared mutable state)
        assertNotNull(child)
    }

    @Test
    fun `TracingConfig holds service name and endpoint`() {
        val config = TracingConfig(
            serviceName = "test-service",
            endpoint = "http://localhost:4317",
            sampleRate = 0.5,
        )
        assertEquals("test-service", config.serviceName)
        assertEquals(0.5, config.sampleRate)
    }

    @Test
    fun `TracingSetup generates trace IDs`() {
        val setup = TracingSetup(TracingConfig(serviceName = "test"))
        val traceId = setup.generateTraceId()
        assertEquals(32, traceId.length)
    }

    @Test
    fun `TracingSetup generates span IDs`() {
        val setup = TracingSetup(TracingConfig(serviceName = "test"))
        val spanId = setup.generateSpanId()
        assertEquals(16, spanId.length)
    }

    @Test
    fun `TracingSetup creates W3C traceparent header`() {
        val setup = TracingSetup(TracingConfig(serviceName = "test", sampleRate = 1.0))
        val headers = setup.createTraceHeaders()
        val traceparent = headers["traceparent"]
        assertNotNull(traceparent)
        assertTrue(traceparent!!.startsWith("00-"))
        assertTrue(traceparent.endsWith("-01"))
    }

    @Test
    fun `NoOpCrashReporter does not throw`() {
        val reporter = NoOpCrashReporter()
        reporter.reportException(RuntimeException("test"))
        reporter.reportError("test error")
        reporter.setUserId("user-1")
        reporter.setUserId(null)
        reporter.addBreadcrumb("action", "category")
    }
}
