package dev.k1s0.error

/**
 * Base exception for all k1s0 service errors.
 *
 * Carries a structured [ErrorCode] and a service-specific error code string
 * following the `{service}.{category}.{reason}` format.
 *
 * @property errorCode The canonical error classification.
 * @property serviceErrorCode A service-specific error code string (e.g. "user.not_found").
 * @property detail A human-readable description of the error.
 * @property traceId Optional distributed trace identifier.
 */
public open class K1s0Exception(
    public val errorCode: ErrorCode,
    public val serviceErrorCode: String,
    public val detail: String,
    public val traceId: String? = null,
    cause: Throwable? = null,
) : RuntimeException(detail, cause) {

    /** Converts this exception to an RFC 7807 [ProblemDetails] response. */
    public fun toProblemDetails(): ProblemDetails = ProblemDetails(
        status = errorCode.httpStatus,
        title = errorCode.name,
        detail = detail,
        errorCode = serviceErrorCode,
        traceId = traceId,
    )
}

/** Exception indicating that a requested resource was not found. */
public class NotFoundException(
    serviceErrorCode: String,
    detail: String,
    traceId: String? = null,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.NOT_FOUND, serviceErrorCode, detail, traceId, cause)

/** Exception indicating that the caller is not authenticated. */
public class UnauthorizedException(
    serviceErrorCode: String,
    detail: String,
    traceId: String? = null,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.UNAUTHENTICATED, serviceErrorCode, detail, traceId, cause)

/** Exception indicating that the caller lacks permission. */
public class ForbiddenException(
    serviceErrorCode: String,
    detail: String,
    traceId: String? = null,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.PERMISSION_DENIED, serviceErrorCode, detail, traceId, cause)

/** Exception indicating a conflict with the current resource state. */
public class ConflictException(
    serviceErrorCode: String,
    detail: String,
    traceId: String? = null,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.CONFLICT, serviceErrorCode, detail, traceId, cause)

/** Exception indicating that input validation failed. */
public class ValidationException(
    serviceErrorCode: String,
    detail: String,
    public val violations: List<String> = emptyList(),
    traceId: String? = null,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.VALIDATION_FAILED, serviceErrorCode, detail, traceId, cause)
