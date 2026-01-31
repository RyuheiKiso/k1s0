package dev.k1s0.android.ui.components

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

/**
 * k1s0 top app bar with consistent styling.
 *
 * Wraps Material 3 [TopAppBar] with the k1s0 color scheme.
 *
 * @param title The title text to display.
 * @param modifier Optional [Modifier] for the app bar.
 * @param navigationIcon Optional composable for the navigation icon (e.g. back arrow).
 * @param actions Optional composable for action icons on the trailing side.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun K1s0TopAppBar(
    title: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable () -> Unit = {},
) {
    TopAppBar(
        title = { Text(text = title) },
        modifier = modifier,
        navigationIcon = navigationIcon,
        actions = { actions() },
        colors = TopAppBarDefaults.topAppBarColors(
            containerColor = MaterialTheme.colorScheme.surface,
            titleContentColor = MaterialTheme.colorScheme.onSurface,
        ),
    )
}
