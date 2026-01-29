using FluentAssertions;
using K1s0.Error;
using Microsoft.EntityFrameworkCore;

namespace K1s0.Db.Tests;

public class TestEntity
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
}

public class TestDbContext : DbContext
{
    public DbSet<TestEntity> TestEntities => Set<TestEntity>();

    public TestDbContext(DbContextOptions<TestDbContext> options) : base(options) { }
}

public class TestRepository : RepositoryBase<TestEntity, int>
{
    public TestRepository(TestDbContext context) : base(context) { }
}

public class RepositoryBaseTests : IDisposable
{
    private readonly TestDbContext _context;
    private readonly TestRepository _repository;

    public RepositoryBaseTests()
    {
        var options = new DbContextOptionsBuilder<TestDbContext>()
            .UseInMemoryDatabase($"TestDb_{Guid.NewGuid()}")
            .Options;
        _context = new TestDbContext(options);
        _repository = new TestRepository(_context);
    }

    public void Dispose()
    {
        _context.Dispose();
        GC.SuppressFinalize(this);
    }

    [Fact]
    public async Task AddAsync_And_FindById_ReturnsEntity()
    {
        var entity = new TestEntity { Id = 1, Name = "Alice" };
        await _repository.AddAsync(entity);
        await _context.SaveChangesAsync();

        var found = await _repository.FindByIdAsync(1);

        found.Should().NotBeNull();
        found!.Name.Should().Be("Alice");
    }

    [Fact]
    public async Task FindByIdAsync_NotFound_ReturnsNull()
    {
        var found = await _repository.FindByIdAsync(999);
        found.Should().BeNull();
    }

    [Fact]
    public async Task GetByIdAsync_NotFound_ThrowsNotFoundException()
    {
        var act = () => _repository.GetByIdAsync(999, "test.entity.not_found");

        await act.Should().ThrowAsync<NotFoundException>()
            .Where(ex => ex.ErrorCode == "test.entity.not_found");
    }

    [Fact]
    public async Task GetAll_ReturnsAllEntities()
    {
        _context.TestEntities.Add(new TestEntity { Id = 1, Name = "A" });
        _context.TestEntities.Add(new TestEntity { Id = 2, Name = "B" });
        await _context.SaveChangesAsync();

        var all = _repository.GetAll().ToList();

        all.Should().HaveCount(2);
    }

    [Fact]
    public async Task Update_ModifiesEntity()
    {
        var entity = new TestEntity { Id = 1, Name = "Before" };
        _context.TestEntities.Add(entity);
        await _context.SaveChangesAsync();

        entity.Name = "After";
        _repository.Update(entity);
        await _context.SaveChangesAsync();

        var found = await _repository.FindByIdAsync(1);
        found!.Name.Should().Be("After");
    }

    [Fact]
    public async Task Remove_DeletesEntity()
    {
        var entity = new TestEntity { Id = 1, Name = "ToDelete" };
        _context.TestEntities.Add(entity);
        await _context.SaveChangesAsync();

        _repository.Remove(entity);
        await _context.SaveChangesAsync();

        var found = await _repository.FindByIdAsync(1);
        found.Should().BeNull();
    }
}

public class UnitOfWorkTests : IDisposable
{
    private readonly TestDbContext _context;

    public UnitOfWorkTests()
    {
        var options = new DbContextOptionsBuilder<TestDbContext>()
            .UseInMemoryDatabase($"UowDb_{Guid.NewGuid()}")
            .Options;
        _context = new TestDbContext(options);
    }

    public void Dispose()
    {
        _context.Dispose();
        GC.SuppressFinalize(this);
    }

    [Fact]
    public async Task SaveChangesAsync_PersistsChanges()
    {
        var uow = new UnitOfWork<TestDbContext>(_context);
        _context.TestEntities.Add(new TestEntity { Id = 1, Name = "Test" });

        int count = await uow.SaveChangesAsync();

        count.Should().Be(1);
    }
}
