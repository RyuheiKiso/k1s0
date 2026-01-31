package dev.k1s0.validation

import kotlinx.serialization.Serializable

/**
 * Represents the result of a validation operation.
 *
 * @property isValid Whether all validation rules passed.
 * @property errors List of validation error messages, empty when valid.
 */
@Serializable
public data class ValidationResult(
    val isValid: Boolean,
    val errors: List<String> = emptyList(),
) {
    public companion object {
        /** Creates a successful validation result. */
        public fun success(): ValidationResult = ValidationResult(isValid = true)

        /** Creates a failed validation result with the given errors. */
        public fun failure(errors: List<String>): ValidationResult =
            ValidationResult(isValid = false, errors = errors)
    }
}
