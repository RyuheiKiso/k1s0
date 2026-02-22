using NSubstitute;
using Xunit;

namespace K1s0.System.SchemaRegistry.Tests;

public class ConfluentSchemaRegistryClientTests
{
    [Fact]
    public async Task RegisterSchemaAsync_MockClient_ReturnsSchemaId()
    {
        // Arrange
        var mockClient = Substitute.For<ISchemaRegistryClient>();
        mockClient.RegisterSchemaAsync("test-subject", "{}", SchemaType.Avro, Arg.Any<CancellationToken>())
            .Returns(1);

        // Act
        var result = await mockClient.RegisterSchemaAsync("test-subject", "{}", SchemaType.Avro);

        // Assert
        Assert.Equal(1, result);
    }

    [Fact]
    public async Task GetSchemaByIdAsync_MockClient_ReturnsSchema()
    {
        // Arrange
        var expected = new RegisteredSchema
        {
            Id = 1,
            Version = 1,
            SchemaString = """{"type":"record","name":"Test","fields":[]}""",
            SchemaType = SchemaType.Avro,
        };

        var mockClient = Substitute.For<ISchemaRegistryClient>();
        mockClient.GetSchemaByIdAsync(1, Arg.Any<CancellationToken>())
            .Returns(expected);

        // Act
        var result = await mockClient.GetSchemaByIdAsync(1);

        // Assert
        Assert.Equal(expected.Id, result.Id);
        Assert.Equal(expected.SchemaString, result.SchemaString);
        Assert.Equal(expected.SchemaType, result.SchemaType);
    }

    [Fact]
    public async Task CheckCompatibilityAsync_MockClient_ReturnsTrue()
    {
        // Arrange
        var mockClient = Substitute.For<ISchemaRegistryClient>();
        mockClient.CheckCompatibilityAsync("test-subject", "{}", Arg.Any<CancellationToken>())
            .Returns(true);

        // Act
        var result = await mockClient.CheckCompatibilityAsync("test-subject", "{}");

        // Assert
        Assert.True(result);
    }

    [Fact]
    public async Task CheckCompatibilityAsync_MockClient_ReturnsFalse()
    {
        // Arrange
        var mockClient = Substitute.For<ISchemaRegistryClient>();
        mockClient.CheckCompatibilityAsync("test-subject", "{}", Arg.Any<CancellationToken>())
            .Returns(false);

        // Act
        var result = await mockClient.CheckCompatibilityAsync("test-subject", "{}");

        // Assert
        Assert.False(result);
    }

    [Fact]
    public void SchemaRegistryConfig_SubjectName_FormatsCorrectly()
    {
        Assert.Equal("my-topic-value", SchemaRegistryConfig.SubjectName("my-topic"));
        Assert.Equal("my-topic-key", SchemaRegistryConfig.SubjectName("my-topic", "key"));
    }

    [Fact]
    public void RegisteredSchema_Record_HasExpectedProperties()
    {
        var schema = new RegisteredSchema
        {
            Id = 42,
            Version = 3,
            SchemaString = """{"type":"string"}""",
            SchemaType = SchemaType.Json,
        };

        Assert.Equal(42, schema.Id);
        Assert.Equal(3, schema.Version);
        Assert.Equal("""{"type":"string"}""", schema.SchemaString);
        Assert.Equal(SchemaType.Json, schema.SchemaType);
    }
}
