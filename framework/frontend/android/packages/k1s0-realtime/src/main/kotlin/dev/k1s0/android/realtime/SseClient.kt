package dev.k1s0.android.realtime

import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.utils.io.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Represents a Server-Sent Event.
 *
 * @property id The event ID, if provided by the server.
 * @property event The event type name, if provided. Defaults to "message".
 * @property data The event data payload.
 */
data class SseEvent(
    val id: String? = null,
    val event: String? = null,
    val data: String,
)

/**
 * Server-Sent Events (SSE) client with automatic reconnection.
 *
 * @param url The SSE endpoint URL.
 * @param reconnectionStrategy The strategy for reconnection attempts.
 * @param scope The coroutine scope for managing the connection lifecycle.
 */
class SseClient(
    private val url: String,
    private val reconnectionStrategy: ReconnectionStrategy = ReconnectionStrategy(),
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) {

    private val client = HttpClient(OkHttp)
    private var connectionJob: Job? = null

    private val _state = MutableStateFlow(ConnectionState.DISCONNECTED)
    /** Observable connection state. */
    val state: StateFlow<ConnectionState> = _state.asStateFlow()

    private val _events = MutableSharedFlow<SseEvent>(extraBufferCapacity = 64)
    /** Flow of incoming SSE events. */
    val events: SharedFlow<SseEvent> = _events.asSharedFlow()

    private var lastEventId: String? = null

    /**
     * Connects to the SSE endpoint and begins receiving events.
     */
    fun connect() {
        if (_state.value == ConnectionState.CONNECTED || _state.value == ConnectionState.CONNECTING) return

        connectionJob?.cancel()
        connectionJob = scope.launch {
            connectInternal(attempt = 0)
        }
    }

    private suspend fun connectInternal(attempt: Int) {
        _state.value = if (attempt == 0) ConnectionState.CONNECTING else ConnectionState.RECONNECTING

        try {
            client.prepareGet(url) {
                header("Accept", "text/event-stream")
                lastEventId?.let { header("Last-Event-ID", it) }
            }.execute { response ->
                _state.value = ConnectionState.CONNECTED

                val channel = response.bodyAsChannel()
                var currentId: String? = null
                var currentEvent: String? = null
                val dataLines = mutableListOf<String>()

                while (!channel.isClosedForRead) {
                    val line = channel.readUTF8Line() ?: break

                    when {
                        line.startsWith("id:") -> {
                            currentId = line.removePrefix("id:").trim()
                        }
                        line.startsWith("event:") -> {
                            currentEvent = line.removePrefix("event:").trim()
                        }
                        line.startsWith("data:") -> {
                            dataLines.add(line.removePrefix("data:").trim())
                        }
                        line.isEmpty() && dataLines.isNotEmpty() -> {
                            val event = SseEvent(
                                id = currentId,
                                event = currentEvent,
                                data = dataLines.joinToString("\n"),
                            )
                            if (currentId != null) lastEventId = currentId
                            _events.emit(event)

                            currentId = null
                            currentEvent = null
                            dataLines.clear()
                        }
                    }
                }
            }
        } catch (_: Exception) {
            // Connection lost or failed
        } finally {
            _state.value = ConnectionState.DISCONNECTED
        }

        // Attempt reconnection
        val delay = reconnectionStrategy.delayForAttempt(attempt)
        if (delay != null) {
            delay(delay)
            connectInternal(attempt + 1)
        }
    }

    /**
     * Disconnects from the SSE endpoint and cancels reconnection.
     */
    fun disconnect() {
        connectionJob?.cancel()
        _state.value = ConnectionState.DISCONNECTED
    }

    /**
     * Closes the client and releases all resources.
     */
    fun close() {
        disconnect()
        client.close()
    }
}
