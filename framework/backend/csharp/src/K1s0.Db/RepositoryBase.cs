using K1s0.Error;
using Microsoft.EntityFrameworkCore;

namespace K1s0.Db;

/// <summary>
/// Generic repository base providing standard CRUD operations.
/// </summary>
/// <typeparam name="TEntity">The entity type.</typeparam>
/// <typeparam name="TId">The entity's primary key type.</typeparam>
public abstract class RepositoryBase<TEntity, TId>
    where TEntity : class
    where TId : notnull
{
    /// <summary>
    /// The underlying <see cref="DbContext"/>.
    /// </summary>
    protected DbContext Context { get; }

    /// <summary>
    /// The <see cref="DbSet{TEntity}"/> for the entity.
    /// </summary>
    protected DbSet<TEntity> DbSet { get; }

    /// <summary>
    /// Creates a new repository instance.
    /// </summary>
    /// <param name="context">The database context.</param>
    protected RepositoryBase(DbContext context)
    {
        Context = context;
        DbSet = context.Set<TEntity>();
    }

    /// <summary>
    /// Finds an entity by its primary key.
    /// </summary>
    /// <param name="id">The primary key value.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The entity, or null if not found.</returns>
    public virtual async Task<TEntity?> FindByIdAsync(TId id, CancellationToken cancellationToken = default)
    {
        return await DbSet.FindAsync([id], cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Finds an entity by its primary key, throwing <see cref="NotFoundException"/> if not found.
    /// </summary>
    /// <param name="id">The primary key value.</param>
    /// <param name="errorCode">The error code to use if the entity is not found.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The entity.</returns>
    public virtual async Task<TEntity> GetByIdAsync(
        TId id,
        string errorCode,
        CancellationToken cancellationToken = default)
    {
        var entity = await FindByIdAsync(id, cancellationToken).ConfigureAwait(false);
        return entity ?? throw new NotFoundException(errorCode, $"{typeof(TEntity).Name} with ID '{id}' was not found.");
    }

    /// <summary>
    /// Returns all entities.
    /// </summary>
    /// <returns>A queryable of all entities.</returns>
    public virtual IQueryable<TEntity> GetAll()
    {
        return DbSet.AsNoTracking();
    }

    /// <summary>
    /// Adds a new entity to the repository.
    /// </summary>
    /// <param name="entity">The entity to add.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    public virtual async Task AddAsync(TEntity entity, CancellationToken cancellationToken = default)
    {
        await DbSet.AddAsync(entity, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Updates an existing entity.
    /// </summary>
    /// <param name="entity">The entity to update.</param>
    public virtual void Update(TEntity entity)
    {
        DbSet.Update(entity);
    }

    /// <summary>
    /// Removes an entity from the repository.
    /// </summary>
    /// <param name="entity">The entity to remove.</param>
    public virtual void Remove(TEntity entity)
    {
        DbSet.Remove(entity);
    }
}
