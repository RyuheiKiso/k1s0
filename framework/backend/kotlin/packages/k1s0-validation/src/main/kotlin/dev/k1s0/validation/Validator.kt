package dev.k1s0.validation

/**
 * DSL-based validator for composing validation rules.
 *
 * Usage:
 * ```kotlin
 * val result = Validator.validate {
 *     field(name, Rules.notBlank("name"))
 *     field(name, Rules.maxLength("name", 100))
 *     field(age, Rules.positiveInt("age"))
 * }
 * ```
 */
public class Validator {

    private val errors = mutableListOf<String>()

    /**
     * Validates a value against a rule and collects any error.
     *
     * @param value The value to validate.
     * @param rule The validation rule to apply.
     */
    public fun <T> field(value: T, rule: ValidationRule<T>) {
        rule.validate(value)?.let { errors.add(it) }
    }

    /**
     * Validates a value against multiple rules.
     *
     * @param value The value to validate.
     * @param rules The validation rules to apply.
     */
    public fun <T> field(value: T, vararg rules: ValidationRule<T>) {
        for (rule in rules) {
            rule.validate(value)?.let { errors.add(it) }
        }
    }

    /**
     * Adds a custom error if the condition is true.
     *
     * @param condition The condition to check.
     * @param message The error message to add.
     */
    public fun check(condition: Boolean, message: String) {
        if (condition) errors.add(message)
    }

    public companion object {
        /**
         * Executes validation using the DSL builder and returns the result.
         *
         * @param block The validation DSL block.
         * @return The aggregated [ValidationResult].
         */
        public fun validate(block: Validator.() -> Unit): ValidationResult {
            val validator = Validator()
            validator.block()
            return if (validator.errors.isEmpty()) {
                ValidationResult.success()
            } else {
                ValidationResult.failure(validator.errors)
            }
        }
    }
}
