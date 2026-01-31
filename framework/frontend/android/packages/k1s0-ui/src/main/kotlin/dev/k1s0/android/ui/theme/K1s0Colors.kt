package dev.k1s0.android.ui.theme

import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

/**
 * k1s0 brand color palette.
 */
object K1s0Colors {
    val Primary = Color(0xFF1565C0)
    val OnPrimary = Color(0xFFFFFFFF)
    val PrimaryContainer = Color(0xFFD1E4FF)
    val OnPrimaryContainer = Color(0xFF001D36)

    val Secondary = Color(0xFF545F71)
    val OnSecondary = Color(0xFFFFFFFF)
    val SecondaryContainer = Color(0xFFD7E3F8)
    val OnSecondaryContainer = Color(0xFF111C2B)

    val Tertiary = Color(0xFF6D5678)
    val OnTertiary = Color(0xFFFFFFFF)
    val TertiaryContainer = Color(0xFFF6D9FF)
    val OnTertiaryContainer = Color(0xFF271431)

    val Error = Color(0xFFBA1A1A)
    val OnError = Color(0xFFFFFFFF)
    val ErrorContainer = Color(0xFFFFDAD6)
    val OnErrorContainer = Color(0xFF410002)

    val Background = Color(0xFFFDFBFF)
    val OnBackground = Color(0xFF1A1C1E)
    val Surface = Color(0xFFFDFBFF)
    val OnSurface = Color(0xFF1A1C1E)

    val DarkBackground = Color(0xFF1A1C1E)
    val DarkOnBackground = Color(0xFFE3E2E6)
    val DarkSurface = Color(0xFF1A1C1E)
    val DarkOnSurface = Color(0xFFE3E2E6)
}

/** Light color scheme for k1s0 applications. */
val K1s0LightColorScheme = lightColorScheme(
    primary = K1s0Colors.Primary,
    onPrimary = K1s0Colors.OnPrimary,
    primaryContainer = K1s0Colors.PrimaryContainer,
    onPrimaryContainer = K1s0Colors.OnPrimaryContainer,
    secondary = K1s0Colors.Secondary,
    onSecondary = K1s0Colors.OnSecondary,
    secondaryContainer = K1s0Colors.SecondaryContainer,
    onSecondaryContainer = K1s0Colors.OnSecondaryContainer,
    tertiary = K1s0Colors.Tertiary,
    onTertiary = K1s0Colors.OnTertiary,
    tertiaryContainer = K1s0Colors.TertiaryContainer,
    onTertiaryContainer = K1s0Colors.OnTertiaryContainer,
    error = K1s0Colors.Error,
    onError = K1s0Colors.OnError,
    errorContainer = K1s0Colors.ErrorContainer,
    onErrorContainer = K1s0Colors.OnErrorContainer,
    background = K1s0Colors.Background,
    onBackground = K1s0Colors.OnBackground,
    surface = K1s0Colors.Surface,
    onSurface = K1s0Colors.OnSurface,
)

/** Dark color scheme for k1s0 applications. */
val K1s0DarkColorScheme = darkColorScheme(
    primary = K1s0Colors.PrimaryContainer,
    onPrimary = K1s0Colors.OnPrimaryContainer,
    primaryContainer = K1s0Colors.Primary,
    onPrimaryContainer = K1s0Colors.OnPrimary,
    secondary = K1s0Colors.SecondaryContainer,
    onSecondary = K1s0Colors.OnSecondaryContainer,
    secondaryContainer = K1s0Colors.Secondary,
    onSecondaryContainer = K1s0Colors.OnSecondary,
    tertiary = K1s0Colors.TertiaryContainer,
    onTertiary = K1s0Colors.OnTertiaryContainer,
    tertiaryContainer = K1s0Colors.Tertiary,
    onTertiaryContainer = K1s0Colors.OnTertiary,
    error = K1s0Colors.ErrorContainer,
    onError = K1s0Colors.OnErrorContainer,
    errorContainer = K1s0Colors.Error,
    onErrorContainer = K1s0Colors.OnError,
    background = K1s0Colors.DarkBackground,
    onBackground = K1s0Colors.DarkOnBackground,
    surface = K1s0Colors.DarkSurface,
    onSurface = K1s0Colors.DarkOnSurface,
)
