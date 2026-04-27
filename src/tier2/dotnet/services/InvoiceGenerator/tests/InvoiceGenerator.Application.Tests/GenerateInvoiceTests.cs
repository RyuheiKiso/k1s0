// GenerateInvoiceUseCase の単体テスト。

using K1s0.Tier2.InvoiceGenerator.Application.UseCases;
using K1s0.Tier2.InvoiceGenerator.Infrastructure.Persistence;
using Xunit;

namespace K1s0.Tier2.InvoiceGenerator.Application.Tests;

public class GenerateInvoiceTests
{
    private static DateTimeOffset Now() => new(2026, 4, 27, 12, 0, 0, TimeSpan.Zero);

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_PersistsAndComputesTotal()
    {
        var repo = new InMemoryInvoiceRepository();
        var useCase = new GenerateInvoiceUseCase(repo, Now);
        var input = new GenerateInvoiceUseCase.Input("Acme", new List<GenerateInvoiceUseCase.LineInput>
        {
            new("apple", 3, "JPY", 100),
            new("banana", 2, "JPY", 200),
        });
        var output = await useCase.ExecuteAsync(input, CancellationToken.None);
        Assert.Equal(700, output.TotalMinorAmount);
        Assert.Equal("JPY", output.Currency);
        Assert.Equal("Acme", output.Customer);
        // repo に永続化されていること。
        var persisted = await repo.FindByIdAsync(output.Id, CancellationToken.None);
        Assert.NotNull(persisted);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_NoLines_Throws()
    {
        var repo = new InMemoryInvoiceRepository();
        var useCase = new GenerateInvoiceUseCase(repo, Now);
        var input = new GenerateInvoiceUseCase.Input("Acme", new List<GenerateInvoiceUseCase.LineInput>());
        await Assert.ThrowsAsync<ArgumentException>(() => useCase.ExecuteAsync(input, CancellationToken.None));
    }
}
