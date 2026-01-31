using FluentAssertions;
using K1s0.Auth.Policy;
using Action = K1s0.Auth.Policy.Action;

namespace K1s0.Auth.Tests;

public class PolicyEvaluatorTests
{
    private static PolicySubject CreateSubject(
        IReadOnlyList<string>? roles = null,
        IReadOnlyList<string>? permissions = null) =>
        new(
            Sub: "user-1",
            Roles: roles ?? Array.Empty<string>(),
            Permissions: permissions ?? Array.Empty<string>(),
            Groups: Array.Empty<string>(),
            TenantId: null);

    [Fact]
    public async Task Admin_WithAdminRole_IsAllowed()
    {
        var repo = new PolicyBuilder()
            .AllowAdmin("orders", "admin")
            .Build();
        var evaluator = new RepositoryPolicyEvaluator(repo);
        var subject = CreateSubject(roles: new[] { "admin" });

        var result = await evaluator.EvaluateAsync(new PolicyRequest(subject, Action.Admin, "orders"));

        result.Should().BeTrue();
    }

    [Fact]
    public async Task Read_WithViewerRole_IsAllowed()
    {
        var repo = new PolicyBuilder()
            .AllowRead("orders", "viewer")
            .Build();
        var evaluator = new RepositoryPolicyEvaluator(repo);
        var subject = CreateSubject(roles: new[] { "viewer" });

        var result = await evaluator.EvaluateAsync(new PolicyRequest(subject, Action.Read, "orders"));

        result.Should().BeTrue();
    }

    [Fact]
    public async Task UndefinedAction_IsDenied()
    {
        var repo = new PolicyBuilder()
            .AllowRead("orders", "viewer")
            .Build();
        var evaluator = new RepositoryPolicyEvaluator(repo);
        var subject = CreateSubject(roles: new[] { "viewer" });

        var result = await evaluator.EvaluateAsync(new PolicyRequest(subject, Action.Delete, "orders"));

        result.Should().BeFalse();
    }

    [Fact]
    public async Task Write_WithoutRequiredRole_IsDenied()
    {
        var repo = new PolicyBuilder()
            .AllowWrite("orders", "editor")
            .Build();
        var evaluator = new RepositoryPolicyEvaluator(repo);
        var subject = CreateSubject(roles: new[] { "viewer" });

        var result = await evaluator.EvaluateAsync(new PolicyRequest(subject, Action.Write, "orders"));

        result.Should().BeFalse();
    }

    [Fact]
    public async Task WildcardPattern_MatchesAllResources()
    {
        var repo = new PolicyBuilder()
            .AllowRead("*", "viewer")
            .Build();
        var evaluator = new RepositoryPolicyEvaluator(repo);
        var subject = CreateSubject(roles: new[] { "viewer" });

        var result = await evaluator.EvaluateAsync(new PolicyRequest(subject, Action.Read, "anything"));

        result.Should().BeTrue();
    }
}
