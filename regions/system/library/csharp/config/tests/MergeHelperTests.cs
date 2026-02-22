using Xunit;
using YamlDotNet.RepresentationModel;

namespace K1s0.System.Config.Tests;

public class MergeHelperTests
{
    [Fact]
    public void DeepMerge_OverlayOverridesScalar()
    {
        var baseNode = ParseMapping("key1: value1\nkey2: value2");
        var overlayNode = ParseMapping("key2: overridden");

        var result = MergeHelper.DeepMerge(baseNode, overlayNode);

        Assert.Equal("overridden", GetScalar(result, "key2"));
        Assert.Equal("value1", GetScalar(result, "key1"));
    }

    [Fact]
    public void DeepMerge_OverlayAddsNewKey()
    {
        var baseNode = ParseMapping("key1: value1");
        var overlayNode = ParseMapping("key2: value2");

        var result = MergeHelper.DeepMerge(baseNode, overlayNode);

        Assert.Equal("value1", GetScalar(result, "key1"));
        Assert.Equal("value2", GetScalar(result, "key2"));
    }

    [Fact]
    public void DeepMerge_NestedMappingMergesRecursively()
    {
        var baseNode = ParseMapping("parent:\n  child1: a\n  child2: b");
        var overlayNode = ParseMapping("parent:\n  child2: overridden\n  child3: c");

        var result = MergeHelper.DeepMerge(baseNode, overlayNode);

        var parent = (YamlMappingNode)result.Children[new YamlScalarNode("parent")];
        Assert.Equal("a", GetScalar(parent, "child1"));
        Assert.Equal("overridden", GetScalar(parent, "child2"));
        Assert.Equal("c", GetScalar(parent, "child3"));
    }

    [Fact]
    public void DeepMerge_EmptyOverlay_ReturnsBase()
    {
        var baseNode = ParseMapping("key: value");
        var overlayNode = new YamlMappingNode();

        var result = MergeHelper.DeepMerge(baseNode, overlayNode);

        Assert.Equal("value", GetScalar(result, "key"));
    }

    [Fact]
    public void DeepMerge_EmptyBase_ReturnsOverlay()
    {
        var baseNode = new YamlMappingNode();
        var overlayNode = ParseMapping("key: value");

        var result = MergeHelper.DeepMerge(baseNode, overlayNode);

        Assert.Equal("value", GetScalar(result, "key"));
    }

    private static YamlMappingNode ParseMapping(string yaml)
    {
        var stream = new YamlStream();
        stream.Load(new StringReader(yaml));
        return (YamlMappingNode)stream.Documents[0].RootNode;
    }

    private static string GetScalar(YamlMappingNode node, string key)
    {
        return ((YamlScalarNode)node.Children[new YamlScalarNode(key)]).Value!;
    }
}
