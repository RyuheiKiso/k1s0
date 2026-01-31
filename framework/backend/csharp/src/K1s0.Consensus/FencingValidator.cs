namespace K1s0.Consensus;

/// <summary>
/// Thread-safe validator for fencing tokens. Ensures that only operations
/// with a token greater than or equal to the highest observed token are accepted.
/// </summary>
public sealed class FencingValidator
{
    private long _highestToken;

    /// <summary>
    /// Creates a new <see cref="FencingValidator"/> with an initial token value.
    /// </summary>
    /// <param name="initialToken">The initial highest known token. Default is 0.</param>
    public FencingValidator(ulong initialToken = 0)
    {
        _highestToken = (long)initialToken;
    }

    /// <summary>
    /// Gets the current highest observed fence token.
    /// </summary>
    public ulong CurrentToken => (ulong)Interlocked.Read(ref _highestToken);

    /// <summary>
    /// Validates a fence token. If the token is greater than or equal to the current
    /// highest token, the highest token is updated and the method returns <c>true</c>.
    /// If the token is stale (lower than the current highest), returns <c>false</c>.
    /// </summary>
    /// <param name="token">The fence token to validate.</param>
    /// <returns><c>true</c> if the token is valid; <c>false</c> if it is stale.</returns>
    public bool Validate(ulong token)
    {
        var tokenLong = (long)token;

        while (true)
        {
            var current = Interlocked.Read(ref _highestToken);
            if (tokenLong < current)
            {
                return false;
            }

            if (tokenLong == current)
            {
                return true;
            }

            if (Interlocked.CompareExchange(ref _highestToken, tokenLong, current) == current)
            {
                return true;
            }
        }
    }

    /// <summary>
    /// Validates a fence token and throws <see cref="StaleFenceTokenException"/> if stale.
    /// </summary>
    /// <param name="token">The fence token to validate.</param>
    /// <exception cref="StaleFenceTokenException">Thrown when the token is stale.</exception>
    public void ValidateOrThrow(ulong token)
    {
        if (!Validate(token))
        {
            throw new StaleFenceTokenException(token, CurrentToken);
        }
    }
}
