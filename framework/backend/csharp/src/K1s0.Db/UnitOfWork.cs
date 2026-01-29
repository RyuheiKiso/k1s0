using Microsoft.EntityFrameworkCore;

namespace K1s0.Db;

/// <summary>
/// Abstraction for committing a batch of changes atomically.
/// </summary>
public interface IUnitOfWork
{
    /// <summary>
    /// Saves all pending changes to the database.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The number of state entries written to the database.</returns>
    Task<int> SaveChangesAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// Unit of work implementation wrapping an EF Core <see cref="DbContext"/>.
/// </summary>
/// <typeparam name="TContext">The DbContext type.</typeparam>
public class UnitOfWork<TContext> : IUnitOfWork where TContext : DbContext
{
    private readonly TContext _context;

    /// <summary>
    /// Creates a new <see cref="UnitOfWork{TContext}"/>.
    /// </summary>
    /// <param name="context">The database context.</param>
    public UnitOfWork(TContext context)
    {
        _context = context;
    }

    /// <inheritdoc />
    public Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return _context.SaveChangesAsync(cancellationToken);
    }
}
