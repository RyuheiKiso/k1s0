package dev.k1s0.android.state

/**
 * Base sealed interface for UI state representation.
 *
 * Provides a consistent pattern for representing loading, success,
 * error, and empty states across all screens in k1s0 applications.
 *
 * @param T The type of data held in the success state.
 */
sealed interface UiState<out T> {

    /** Initial idle state before any data loading. */
    data object Idle : UiState<Nothing>

    /** Data is currently being loaded. */
    data object Loading : UiState<Nothing>

    /**
     * Data loaded successfully.
     *
     * @property data The loaded data.
     */
    data class Success<T>(val data: T) : UiState<T>

    /**
     * An error occurred during data loading.
     *
     * @property message A human-readable error message.
     * @property cause The underlying throwable, if available.
     */
    data class Error(
        val message: String,
        val cause: Throwable? = null,
    ) : UiState<Nothing>

    /** Data loaded but the result set is empty. */
    data object Empty : UiState<Nothing>
}

/**
 * Returns the data if this state is [UiState.Success], or null otherwise.
 */
fun <T> UiState<T>.dataOrNull(): T? = when (this) {
    is UiState.Success -> data
    else -> null
}

/**
 * Returns true if this state is [UiState.Loading].
 */
fun <T> UiState<T>.isLoading(): Boolean = this is UiState.Loading

/**
 * Returns true if this state is [UiState.Error].
 */
fun <T> UiState<T>.isError(): Boolean = this is UiState.Error
