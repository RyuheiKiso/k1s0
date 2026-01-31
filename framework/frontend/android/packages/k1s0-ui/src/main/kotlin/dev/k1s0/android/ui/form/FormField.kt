package dev.k1s0.android.ui.form

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp

/**
 * Schema-driven form field types.
 */
enum class FieldType {
    TEXT,
    EMAIL,
    PASSWORD,
    NUMBER,
    PHONE,
}

/**
 * Schema definition for a single form field.
 *
 * @property name The field identifier used as a key in the form data map.
 * @property label The human-readable label displayed above the input.
 * @property type The input field type controlling keyboard and validation behavior.
 * @property required Whether the field must have a non-empty value.
 * @property placeholder Optional placeholder text shown when the field is empty.
 */
data class FormFieldSchema(
    val name: String,
    val label: String,
    val type: FieldType = FieldType.TEXT,
    val required: Boolean = false,
    val placeholder: String? = null,
)

/**
 * Renders a single form field based on a [FormFieldSchema].
 *
 * @param schema The schema defining this field's behavior and appearance.
 * @param value The current field value.
 * @param onValueChange Callback invoked when the field value changes.
 * @param error Optional error message to display below the field.
 * @param modifier Optional [Modifier] for the field container.
 */
@Composable
fun FormField(
    schema: FormFieldSchema,
    value: String,
    onValueChange: (String) -> Unit,
    error: String? = null,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(vertical = 4.dp)) {
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            label = { Text(schema.label) },
            placeholder = schema.placeholder?.let { { Text(it) } },
            isError = error != null,
            modifier = Modifier.fillMaxWidth(),
            keyboardOptions = KeyboardOptions(
                keyboardType = when (schema.type) {
                    FieldType.TEXT -> KeyboardType.Text
                    FieldType.EMAIL -> KeyboardType.Email
                    FieldType.PASSWORD -> KeyboardType.Password
                    FieldType.NUMBER -> KeyboardType.Number
                    FieldType.PHONE -> KeyboardType.Phone
                },
            ),
            visualTransformation = if (schema.type == FieldType.PASSWORD) {
                PasswordVisualTransformation()
            } else {
                VisualTransformation.None
            },
            singleLine = true,
        )

        if (error != null) {
            Text(
                text = error,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.padding(start = 16.dp, top = 4.dp),
            )
        }
    }
}
