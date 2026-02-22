using YamlDotNet.RepresentationModel;

namespace K1s0.System.Config;

public static class MergeHelper
{
    public static YamlMappingNode DeepMerge(
        YamlMappingNode baseNode,
        YamlMappingNode overlayNode)
    {
        var result = new YamlMappingNode();

        foreach (var entry in baseNode.Children)
        {
            result.Add(entry.Key, entry.Value);
        }

        foreach (var entry in overlayNode.Children)
        {
            if (result.Children.ContainsKey(entry.Key)
                && result.Children[entry.Key] is YamlMappingNode baseMapping
                && entry.Value is YamlMappingNode overlayMapping)
            {
                result.Children[entry.Key] = DeepMerge(baseMapping, overlayMapping);
            }
            else
            {
                result.Children[entry.Key] = entry.Value;
            }
        }

        return result;
    }
}
