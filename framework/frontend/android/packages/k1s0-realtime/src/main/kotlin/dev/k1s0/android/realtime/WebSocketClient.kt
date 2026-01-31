package dev.k1s0.android.realtime

import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Connection state for WebSocket and SSE clients.
 */
enum class ConnectionState {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    RECONNECTING,
}

/**
 * WebSocket client with automatic reconnection, heartbeat, and offline queue support.
 *
 * @param url The WebSocket server URL (ws:// or wss://).
 * @param reconnectionStrategy The strategy for reconnection attempts.
 * @param offlineQueue The queue for buffering messages while disconnected.
 * @param heartbeatIntervalMs The interval for sending heartbeat (ping) frames, in milliseconds.
 *   Set to 0 to disable heartbeats.
 * @param scope The coroutine scope for managing the connection lifecycle.
 */
class WebSocketClient(
    private val url: String,
    private val reconnectionStrategy: ReconnectionStrategy = ReconnectionStrategy(),
    private val offlineQueue: OfflineQueue = OfflineQueue(),
    private val heartbeatIntervalMs: Long = 30_000L,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) {

    private val client = HttpClient(OkHttp) {
        install(WebSockets)
    }

    private var session: DefaultClientWebSocketSession? = null
    private var connectionJob: Job? = null
    private var heartbeatJob: Job? = null

    private val _state = MutableStateFlow(ConnectionState.DISCONNECTED)
    /** Observable connection state. */
    val state: StateFlow<ConnectionState> = _state.asStateFlow()

    private val _messages = MutableSharedFlow<String>(extraBufferCapacity = 64)
    /** Flow of incoming text messages from the server. */
    val messages: SharedFlow<String> = _messages.asSharedFlow()

    /**
     * Connects to the WebSocket server.
     *
     * If already connected, this is a no-op. On connection failure,
     * automatic reconnection is attempted according to the [reconnectionStrategy].
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
            client.webSocket(url) {
                session = this
                _state.value = ConnectionState.CONNECTED

                // Drain offline queue
                val queued = offlineQueue.drain()
                queued.forEach { send(Frame.Text(it)) }

                // Start heartbeat
                startHeartbeat()

                // Read incoming messages
                for (frame in incoming) {
                    if (frame is Frame.Text) {
                        _messages.emit(frame.readText())
                    }
                }
            }
        } catch (_: Exception) {
            // Connection lost or failed
        } finally {
            session = null
            heartbeatJob?.cancel()
            _state.value = ConnectionState.DISCONNECTED
        }

        // Attempt reconnection
        val delay = reconnectionStrategy.delayForAttempt(attempt)
        if (delay != null) {
            delay(delay)
            connectInternal(attempt + 1)
        }
    }

    private fun startHeartbeat() {
        if (heartbeatIntervalMs <= 0) return

        heartbeatJob?.cancel()
        heartbeatJob = scope.launch {
            while (isActive) {
                delay(heartbeatIntervalMs)
                try {
                    session?.send(Frame.Ping(byteArrayOf()))
                } catch (_: Exception) {
                    break
                }
            }
        }
    }

    /**
     * Sends a text message through the WebSocket connection.
     *
     * If not currently connected, the message is added to the [offlineQueue]
     * and will be sent when the connection is re-established.
     *
     * @param message The text message to send.
     */
    suspend fun send(message: String) {
        val currentSession = session
        if (currentSession != null && _state.value == ConnectionState.CONNECTED) {
            try {
                currentSession.send(Frame.Text(message))
            } catch (_: Exception) {
                offlineQueue.enqueue(message)
            }
        } else {
            offlineQueue.enqueue(message)
        }
    }

    /**
     * Disconnects from the WebSocket server and cancels reconnection.
     */
    fun disconnect() {
        connectionJob?.cancel()
        heartbeatJob?.cancel()
        scope.launch {
            try {
                session?.close(CloseReason(CloseReason.Codes.NORMAL, "Client disconnect"))
            } catch (_: Exception) {
                // Ignore close errors
            }
            session = null
            _state.value = ConnectionState.DISCONNECTED
        }
    }

    /**
     * Closes the client and releases all resources.
     */
    fun close() {
        disconnect()
        client.close()
    }
}
