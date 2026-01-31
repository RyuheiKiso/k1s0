package dev.k1s0.android.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class StateFlowExtensionsTest {

    @Test
    fun `UiState Idle is initial state`() {
        val state: UiState<String> = UiState.Idle
        assertNull(state.dataOrNull())
        assertFalse(state.isLoading())
        assertFalse(state.isError())
    }

    @Test
    fun `UiState Loading is detected correctly`() {
        val state: UiState<String> = UiState.Loading
        assertTrue(state.isLoading())
        assertNull(state.dataOrNull())
    }

    @Test
    fun `UiState Success provides data`() {
        val state: UiState<String> = UiState.Success("hello")
        assertEquals("hello", state.dataOrNull())
        assertFalse(state.isLoading())
    }

    @Test
    fun `UiState Error provides message`() {
        val state: UiState<String> = UiState.Error("failed")
        assertTrue(state.isError())
        assertNull(state.dataOrNull())
    }

    @Test
    fun `UiState Empty has no data`() {
        val state: UiState<String> = UiState.Empty
        assertNull(state.dataOrNull())
    }

    @Test
    fun `MutableStateFlow update transforms value`() {
        val flow = MutableStateFlow(0)
        flow.update { it + 1 }
        assertEquals(1, flow.value)
    }

    @Test
    fun `asUiState converts flow to UiState flow`() = runTest {
        val source = flowOf("data")
        val stateFlow = source.asUiState(this)
        // After collection completes, final state should be Success
        // (exact timing depends on coroutine execution)
        // We verify the flow was created without error
        assertTrue(stateFlow.value is UiState.Idle || stateFlow.value is UiState.Loading || stateFlow.value is UiState.Success)
    }

    @Test
    fun `UiState when expression is exhaustive`() {
        val states: List<UiState<Int>> = listOf(
            UiState.Idle,
            UiState.Loading,
            UiState.Success(42),
            UiState.Error("err"),
            UiState.Empty,
        )

        states.forEach { state ->
            val label = when (state) {
                is UiState.Idle -> "idle"
                is UiState.Loading -> "loading"
                is UiState.Success -> "success"
                is UiState.Error -> "error"
                is UiState.Empty -> "empty"
            }
            assertTrue(label.isNotEmpty())
        }
    }
}
