package dev.k1s0.android.ui

import dev.k1s0.android.ui.theme.K1s0Colors
import dev.k1s0.android.ui.theme.K1s0DarkColorScheme
import dev.k1s0.android.ui.theme.K1s0LightColorScheme
import dev.k1s0.android.ui.theme.K1s0Typography
import dev.k1s0.android.ui.form.FieldType
import dev.k1s0.android.ui.form.FormFieldSchema
import dev.k1s0.android.ui.form.FormSchema
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class K1s0ThemeTest {

    @Test
    fun `light color scheme uses correct primary color`() {
        assertEquals(K1s0Colors.Primary, K1s0LightColorScheme.primary)
    }

    @Test
    fun `dark color scheme differs from light`() {
        assertNotEquals(K1s0LightColorScheme.background, K1s0DarkColorScheme.background)
    }

    @Test
    fun `typography body large is defined`() {
        assertNotNull(K1s0Typography.bodyLarge)
        assertTrue(K1s0Typography.bodyLarge.fontSize.value > 0)
    }

    @Test
    fun `FormFieldSchema holds field configuration`() {
        val schema = FormFieldSchema(
            name = "email",
            label = "Email Address",
            type = FieldType.EMAIL,
            required = true,
            placeholder = "user@example.com",
        )

        assertEquals("email", schema.name)
        assertEquals(FieldType.EMAIL, schema.type)
        assertTrue(schema.required)
    }

    @Test
    fun `FormSchema holds fields and submit label`() {
        val schema = FormSchema(
            fields = listOf(
                FormFieldSchema(name = "name", label = "Name", required = true),
                FormFieldSchema(name = "email", label = "Email", type = FieldType.EMAIL),
            ),
            submitLabel = "Register",
        )

        assertEquals(2, schema.fields.size)
        assertEquals("Register", schema.submitLabel)
    }
}
