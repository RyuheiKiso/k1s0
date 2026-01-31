package dev.k1s0.android.state

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Base ViewModel with built-in [UiState] management.
 *
 * Provides a convenient pattern for managing a single UI state flow,
 * executing async operations with automatic loading/error handling.
 *
 * @param T The type of data for the [UiState].
 */
abstract class K1s0ViewModel<T> : ViewModel() {

    private val _uiState = MutableStateFlow<UiState<T>>(UiState.Idle)

    /** The current UI state as a read-only [StateFlow]. */
    val uiState: StateFlow<UiState<T>> = _uiState.asStateFlow()

    /**
     * Updates the UI state.
     *
     * @param state The new UI state.
     */
    protected fun setState(state: UiState<T>) {
        _uiState.value = state
    }

    /**
     * Executes a suspending block with automatic loading and error state management.
     *
     * Sets the state to [UiState.Loading] before execution, then to [UiState.Success]
     * or [UiState.Error] based on the result.
     *
     * @param block The suspending block that produces the data.
     */
    protected fun loadData(block: suspend CoroutineScope.() -> T) {
        viewModelScope.launch {
            setState(UiState.Loading)
            try {
                val data = block()
                setState(UiState.Success(data))
            } catch (e: Exception) {
                setState(UiState.Error(
                    message = e.message ?: "An unexpected error occurred",
                    cause = e,
                ))
            }
        }
    }

    /**
     * Executes a suspending block within [viewModelScope] without state management.
     *
     * @param block The suspending block to execute.
     */
    protected fun launch(block: suspend CoroutineScope.() -> Unit) {
        viewModelScope.launch(block = block)
    }
}
