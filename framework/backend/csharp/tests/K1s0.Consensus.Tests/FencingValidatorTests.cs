using FluentAssertions;
using Xunit;

namespace K1s0.Consensus.Tests;

public class FencingValidatorTests
{
    [Fact]
    public void Validate_WithHigherToken_ReturnsTrue()
    {
        var validator = new FencingValidator(0);

        validator.Validate(1).Should().BeTrue();
        validator.CurrentToken.Should().Be(1);
    }

    [Fact]
    public void Validate_WithEqualToken_ReturnsTrue()
    {
        var validator = new FencingValidator(5);

        validator.Validate(5).Should().BeTrue();
        validator.CurrentToken.Should().Be(5);
    }

    [Fact]
    public void Validate_WithLowerToken_ReturnsFalse()
    {
        var validator = new FencingValidator(10);

        validator.Validate(5).Should().BeFalse();
        validator.CurrentToken.Should().Be(10);
    }

    [Fact]
    public void Validate_MonotonicallyIncreasing_AllReturnTrue()
    {
        var validator = new FencingValidator(0);

        for (ulong i = 1; i <= 100; i++)
        {
            validator.Validate(i).Should().BeTrue();
        }

        validator.CurrentToken.Should().Be(100);
    }

    [Fact]
    public void Validate_AfterHigherToken_LowerTokenReturnsFalse()
    {
        var validator = new FencingValidator(0);

        validator.Validate(10).Should().BeTrue();
        validator.Validate(5).Should().BeFalse();
        validator.Validate(10).Should().BeTrue();
        validator.Validate(11).Should().BeTrue();
    }

    [Fact]
    public void ValidateOrThrow_WithValidToken_DoesNotThrow()
    {
        var validator = new FencingValidator(0);

        var act = () => validator.ValidateOrThrow(1);
        act.Should().NotThrow();
    }

    [Fact]
    public void ValidateOrThrow_WithStaleToken_ThrowsStaleFenceTokenException()
    {
        var validator = new FencingValidator(10);

        var act = () => validator.ValidateOrThrow(5);
        act.Should().Throw<StaleFenceTokenException>()
            .Where(e => e.PresentedToken == 5 && e.CurrentToken == 10);
    }

    [Theory]
    [InlineData(0UL, 0UL, true)]
    [InlineData(0UL, 1UL, true)]
    [InlineData(5UL, 5UL, true)]
    [InlineData(5UL, 10UL, true)]
    [InlineData(10UL, 5UL, false)]
    [InlineData(100UL, 1UL, false)]
    public void Validate_ParameterizedCases(ulong initial, ulong token, bool expected)
    {
        var validator = new FencingValidator(initial);
        validator.Validate(token).Should().Be(expected);
    }

    [Fact]
    public void Constructor_WithInitialToken_SetsCurrentToken()
    {
        var validator = new FencingValidator(42);
        validator.CurrentToken.Should().Be(42);
    }

    [Fact]
    public void Constructor_Default_StartsAtZero()
    {
        var validator = new FencingValidator();
        validator.CurrentToken.Should().Be(0);
    }

    [Fact]
    public async Task Validate_ConcurrentAccess_MaintainsMonotonicity()
    {
        var validator = new FencingValidator(0);
        var tasks = new List<Task>();

        for (var i = 0; i < 100; i++)
        {
            var token = (ulong)i;
            tasks.Add(Task.Run(() => validator.Validate(token)));
        }

        await Task.WhenAll(tasks);

        // After all concurrent validations, current token should be at least
        // as high as the highest submitted token
        validator.CurrentToken.Should().BeGreaterThanOrEqualTo(99);
    }
}
