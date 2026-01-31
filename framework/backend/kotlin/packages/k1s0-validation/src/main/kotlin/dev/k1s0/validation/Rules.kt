package dev.k1s0.validation

/**
 * A single validation rule that checks a value and returns an optional error message.
 */
public fun interface ValidationRule<T> {
    /**
     * Validates the given value.
     *
     * @param value The value to validate.
     * @return An error message if validation fails, or null if it passes.
     */
    public fun validate(value: T): String?
}

/** Built-in validation rules for common patterns. */
public object Rules {

    /** Validates that a string is not blank. */
    public fun notBlank(fieldName: String): ValidationRule<String> = ValidationRule { value ->
        if (value.isBlank()) "$fieldName must not be blank" else null
    }

    /** Validates that a string has a minimum length. */
    public fun minLength(fieldName: String, min: Int): ValidationRule<String> = ValidationRule { value ->
        if (value.length < min) "$fieldName must be at least $min characters" else null
    }

    /** Validates that a string has a maximum length. */
    public fun maxLength(fieldName: String, max: Int): ValidationRule<String> = ValidationRule { value ->
        if (value.length > max) "$fieldName must be at most $max characters" else null
    }

    /** Validates that a string matches a regex pattern. */
    public fun pattern(fieldName: String, regex: Regex, message: String? = null): ValidationRule<String> =
        ValidationRule { value ->
            if (!regex.matches(value)) (message ?: "$fieldName has invalid format") else null
        }

    /** Validates that a comparable value is within a range. */
    public fun <T : Comparable<T>> range(fieldName: String, min: T, max: T): ValidationRule<T> =
        ValidationRule { value ->
            if (value < min || value > max) "$fieldName must be between $min and $max" else null
        }

    /** Validates that a value is positive. */
    public fun positiveInt(fieldName: String): ValidationRule<Int> = ValidationRule { value ->
        if (value <= 0) "$fieldName must be positive" else null
    }

    /** Validates that a string looks like an email address. */
    public fun email(fieldName: String): ValidationRule<String> =
        pattern(fieldName, Regex("^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$"), "$fieldName must be a valid email")
}
