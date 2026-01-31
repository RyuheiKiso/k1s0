package dev.k1s0.android.http.throttle

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.atomic.AtomicInteger

/**
 * リクエストスロットル設定
 *
 * @property maxRequestsPerSecond 1秒あたりの最大リクエスト数
 * @property maxConcurrent 同時接続数の上限
 * @property maxQueueSize キュー上限（超過時は即座にリジェクト）
 */
data class ThrottleConfig(
    val maxRequestsPerSecond: Int = 10,
    val maxConcurrent: Int = 5,
    val maxQueueSize: Int = 50,
)

/**
 * リクエストスロットル
 *
 * トークンバケット + 同時接続制限 + キュー上限によるリクエストレート制御。
 */
class RequestThrottle(
    private val config: ThrottleConfig = ThrottleConfig(),
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.Default + SupervisorJob()),
) {
    private val semaphore = Semaphore(config.maxConcurrent)
    private val tokenBucket = AtomicInteger(config.maxRequestsPerSecond)
    private val queue = LinkedBlockingQueue<CompletableDeferred<Unit>>(config.maxQueueSize)
    private val allowed = AtomicInteger(0)
    private val rejected = AtomicInteger(0)
    private val refillJob: Job

    init {
        refillJob = scope.launch {
            while (isActive) {
                delay(1000)
                tokenBucket.set(config.maxRequestsPerSecond)
                processQueue()
            }
        }
    }

    /**
     * Acquires a slot. Suspends if tokens or concurrency slots are unavailable.
     * Throws [IllegalStateException] if the queue is full.
     */
    suspend fun acquire() {
        if (tokenBucket.decrementAndGet() >= 0) {
            semaphore.acquire()
            allowed.incrementAndGet()
            return
        }
        tokenBucket.incrementAndGet()
        val deferred = CompletableDeferred<Unit>()
        if (!queue.offer(deferred)) {
            rejected.incrementAndGet()
            throw IllegalStateException("Request throttle queue full")
        }
        deferred.await()
    }

    /** Releases a slot after the request completes. */
    fun release() {
        semaphore.release()
        processQueue()
    }

    /** Throttle statistics snapshot. */
    data class Stats(
        val allowed: Int,
        val rejected: Int,
        val queued: Int,
        val active: Int,
    )

    /** Returns current throttle statistics. */
    fun stats(): Stats = Stats(
        allowed = allowed.get(),
        rejected = rejected.get(),
        queued = queue.size,
        active = config.maxConcurrent - semaphore.availablePermits,
    )

    /** Disposes of the throttle, cancelling the refill job and rejecting queued requests. */
    fun dispose() {
        refillJob.cancel()
        queue.forEach { it.completeExceptionally(IllegalStateException("Throttle disposed")) }
        queue.clear()
    }

    private fun processQueue() {
        while (tokenBucket.get() > 0 && semaphore.availablePermits > 0) {
            val deferred = queue.poll() ?: break
            if (tokenBucket.decrementAndGet() >= 0) {
                semaphore.tryAcquire()
                allowed.incrementAndGet()
                deferred.complete(Unit)
            } else {
                tokenBucket.incrementAndGet()
                queue.offer(deferred)
                break
            }
        }
    }
}
