namespace K1s0.Cache;

/// <summary>
/// Base exception for all cache-related errors.
/// </summary>
public class CacheException : Exception
{
    /// <summary>
    /// Initializes a new instance of the <see cref="CacheException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public CacheException(string message)
        : base(message)
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public CacheException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}

/// <summary>
/// Exception thrown when a connection to the Redis server cannot be established.
/// </summary>
public class CacheConnectionException : CacheException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="CacheConnectionException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public CacheConnectionException(string message)
        : base(message)
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheConnectionException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public CacheConnectionException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}

/// <summary>
/// Exception thrown when serialization or deserialization of a cache value fails.
/// </summary>
public class CacheSerializationException : CacheException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="CacheSerializationException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public CacheSerializationException(string message)
        : base(message)
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheSerializationException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public CacheSerializationException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}

/// <summary>
/// Exception thrown when a cache operation fails at the server level.
/// </summary>
public class CacheOperationException : CacheException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="CacheOperationException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public CacheOperationException(string message)
        : base(message)
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="CacheOperationException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public CacheOperationException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}
