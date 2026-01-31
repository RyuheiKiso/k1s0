package dev.k1s0.android.ui.form

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * Schema definition for an entire form.
 *
 * @property fields The ordered list of field schemas comprising the form.
 * @property submitLabel The label for the submit button. Defaults to "Submit".
 */
data class FormSchema(
    val fields: List<FormFieldSchema>,
    val submitLabel: String = "Submit",
)

/**
 * Generates a complete form from a [FormSchema].
 *
 * Renders all fields defined in the schema, manages form state internally,
 * performs basic required-field validation, and invokes [onSubmit] with
 * the collected form data when the user submits.
 *
 * @param schema The form schema defining the fields and submit label.
 * @param onSubmit Callback invoked with a map of field name to value on submission.
 * @param modifier Optional [Modifier] for the form container.
 */
@Composable
fun FormGenerator(
    schema: FormSchema,
    onSubmit: (Map<String, String>) -> Unit,
    modifier: Modifier = Modifier,
) {
    val formValues = remember { mutableStateMapOf<String, String>() }
    val formErrors = remember { mutableStateMapOf<String, String>() }

    Column(modifier = modifier.padding(16.dp)) {
        schema.fields.forEach { fieldSchema ->
            FormField(
                schema = fieldSchema,
                value = formValues[fieldSchema.name] ?: "",
                onValueChange = { value ->
                    formValues[fieldSchema.name] = value
                    formErrors.remove(fieldSchema.name)
                },
                error = formErrors[fieldSchema.name],
                modifier = Modifier.fillMaxWidth(),
            )
        }

        Spacer(modifier = Modifier.height(16.dp))

        Button(
            onClick = {
                formErrors.clear()
                var hasError = false

                schema.fields.forEach { fieldSchema ->
                    val value = formValues[fieldSchema.name] ?: ""
                    if (fieldSchema.required && value.isBlank()) {
                        formErrors[fieldSchema.name] = "${fieldSchema.label} is required"
                        hasError = true
                    }
                }

                if (!hasError) {
                    onSubmit(formValues.toMap())
                }
            },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(schema.submitLabel)
        }
    }
}
