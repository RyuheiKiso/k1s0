package dev.k1s0.android.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable

/**
 * k1s0 Material 3 theme composable.
 *
 * Wraps [MaterialTheme] with the k1s0 color scheme and typography.
 * Automatically switches between light and dark color schemes
 * based on the system theme setting.
 *
 * @param darkTheme Whether to use the dark color scheme. Defaults to the system setting.
 * @param content The composable content to render within this theme.
 */
@Composable
fun K1s0Theme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val colorScheme = if (darkTheme) K1s0DarkColorScheme else K1s0LightColorScheme

    MaterialTheme(
        colorScheme = colorScheme,
        typography = K1s0Typography,
        content = content,
    )
}
