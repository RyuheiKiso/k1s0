namespace K1s0.System.Idempotency;

public interface IIdempotencyStore
{
    Task<IdempotencyRecord?> GetAsync(string key);

    Task InsertAsync(IdempotencyRecord record);

    Task UpdateAsync(string key, IdempotencyStatus status, string? body = null, int? code = null);

    Task<bool> DeleteAsync(string key);
}
