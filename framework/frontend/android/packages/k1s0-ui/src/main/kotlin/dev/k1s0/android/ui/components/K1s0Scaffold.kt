package dev.k1s0.android.ui.components

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable

/**
 * k1s0 app scaffold wrapping Material 3 [Scaffold].
 *
 * Provides a consistent layout structure with optional top bar,
 * bottom bar, floating action button, and snackbar host.
 *
 * @param topBar Optional composable for the top app bar.
 * @param bottomBar Optional composable for the bottom navigation bar.
 * @param floatingActionButton Optional composable for the FAB.
 * @param content The main body content, receiving [PaddingValues] for proper insets.
 */
@Composable
fun K1s0Scaffold(
    topBar: @Composable () -> Unit = {},
    bottomBar: @Composable () -> Unit = {},
    floatingActionButton: @Composable () -> Unit = {},
    content: @Composable (PaddingValues) -> Unit,
) {
    Scaffold(
        topBar = topBar,
        bottomBar = bottomBar,
        floatingActionButton = floatingActionButton,
        content = content,
    )
}
