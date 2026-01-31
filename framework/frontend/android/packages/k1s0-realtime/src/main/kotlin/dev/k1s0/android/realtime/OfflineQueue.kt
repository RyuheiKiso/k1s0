package dev.k1s0.android.realtime

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Thread-safe queue for messages that could not be sent while offline.
 *
 * Messages are enqueued when the connection is unavailable and
 * drained (sent) when the connection is re-established.
 *
 * @property maxSize The maximum number of messages to buffer. Oldest messages
 *   are dropped when the queue exceeds this size. Defaults to 100.
 */
class OfflineQueue(private val maxSize: Int = 100) {

    private val mutex = Mutex()
    private val queue = ArrayDeque<String>()

    /**
     * Adds a message to the offline queue.
     *
     * If the queue is at capacity, the oldest message is removed.
     *
     * @param message The message to enqueue.
     */
    suspend fun enqueue(message: String) {
        mutex.withLock {
            if (queue.size >= maxSize) {
                queue.removeFirst()
            }
            queue.addLast(message)
        }
    }

    /**
     * Drains all queued messages in FIFO order.
     *
     * @return A list of all queued messages. The queue is cleared after draining.
     */
    suspend fun drain(): List<String> {
        return mutex.withLock {
            val messages = queue.toList()
            queue.clear()
            messages
        }
    }

    /**
     * Returns the current number of queued messages.
     */
    suspend fun size(): Int = mutex.withLock { queue.size }

    /**
     * Returns true if the queue has no messages.
     */
    suspend fun isEmpty(): Boolean = mutex.withLock { queue.isEmpty() }

    /**
     * Clears all queued messages without returning them.
     */
    suspend fun clear() {
        mutex.withLock { queue.clear() }
    }
}
