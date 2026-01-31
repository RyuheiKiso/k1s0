package dev.k1s0.android.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn

/**
 * Converts a [Flow] of data into a [StateFlow] of [UiState],
 * automatically handling loading, success, and error states.
 *
 * @param T The data type emitted by the source flow.
 * @param scope The [CoroutineScope] for the resulting [StateFlow].
 * @param started The sharing strategy. Defaults to [SharingStarted.WhileSubscribed] with 5s stop timeout.
 * @return A [StateFlow] of [UiState] wrapping the data.
 */
fun <T> Flow<T>.asUiState(
    scope: CoroutineScope,
    started: SharingStarted = SharingStarted.WhileSubscribed(5_000),
): StateFlow<UiState<T>> {
    return this
        .map<T, UiState<T>> { UiState.Success(it) }
        .onStart { emit(UiState.Loading) }
        .catch { emit(UiState.Error(it.message ?: "Unknown error", it)) }
        .stateIn(scope, started, UiState.Idle)
}

/**
 * Updates a [MutableStateFlow] value using a transform function.
 *
 * @param T The state type.
 * @param transform A function that receives the current value and returns the new value.
 */
inline fun <T> MutableStateFlow<T>.update(transform: (T) -> T) {
    value = transform(value)
}

/**
 * Maps a [StateFlow] of [UiState] to a [StateFlow] of [UiState] with a different data type.
 *
 * @param T The source data type.
 * @param R The target data type.
 * @param scope The [CoroutineScope] for the resulting [StateFlow].
 * @param transform The mapping function applied to the success data.
 * @return A new [StateFlow] of [UiState] with the transformed data type.
 */
fun <T, R> StateFlow<UiState<T>>.mapSuccess(
    scope: CoroutineScope,
    transform: (T) -> R,
): StateFlow<UiState<R>> {
    return this.map { state ->
        when (state) {
            is UiState.Idle -> UiState.Idle
            is UiState.Loading -> UiState.Loading
            is UiState.Success -> UiState.Success(transform(state.data))
            is UiState.Error -> UiState.Error(state.message, state.cause)
            is UiState.Empty -> UiState.Empty
        }
    }.stateIn(scope, SharingStarted.Eagerly, UiState.Idle)
}
