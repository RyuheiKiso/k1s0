package dev.k1s0.error

/**
 * Enumeration of standard error codes used across k1s0 services.
 *
 * Each code maps to both an HTTP status code and a gRPC status code string
 * to ensure consistent error representation across protocols.
 *
 * Error codes follow the format: `{service}.{category}.{reason}`
 */
public enum class ErrorCode(
    /** The corresponding HTTP status code. */
    public val httpStatus: Int,
    /** The corresponding gRPC status code name. */
    public val grpcStatus: String,
) {
    /** The request contains invalid arguments. */
    INVALID_ARGUMENT(400, "INVALID_ARGUMENT"),

    /** Authentication is required or has failed. */
    UNAUTHENTICATED(401, "UNAUTHENTICATED"),

    /** The caller does not have permission. */
    PERMISSION_DENIED(403, "PERMISSION_DENIED"),

    /** The requested resource was not found. */
    NOT_FOUND(404, "NOT_FOUND"),

    /** The request conflicts with the current state. */
    CONFLICT(409, "ALREADY_EXISTS"),

    /** Validation of the input failed. */
    VALIDATION_FAILED(422, "INVALID_ARGUMENT"),

    /** An internal server error occurred. */
    INTERNAL(500, "INTERNAL"),

    /** The service is currently unavailable. */
    UNAVAILABLE(503, "UNAVAILABLE"),

    /** The operation timed out. */
    DEADLINE_EXCEEDED(504, "DEADLINE_EXCEEDED"),
}
