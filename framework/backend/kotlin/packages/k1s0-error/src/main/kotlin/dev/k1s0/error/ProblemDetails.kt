package dev.k1s0.error

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * RFC 7807 Problem Details representation for HTTP error responses.
 *
 * @property status The HTTP status code.
 * @property title A short, human-readable summary of the problem type.
 * @property detail A human-readable explanation specific to this occurrence.
 * @property errorCode The service-specific error code.
 * @property traceId The distributed trace identifier for correlation.
 */
@Serializable
public data class ProblemDetails(
    val status: Int,
    val title: String,
    val detail: String,
    @SerialName("error_code")
    val errorCode: String,
    @SerialName("trace_id")
    val traceId: String? = null,
)
