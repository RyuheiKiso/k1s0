// Architecture テスト用の Assembly メタ情報。
// Trait 漏れを防ぐため、本 Assembly のすべてのテストに Category=Architecture を付与する保険。

using Xunit;

[assembly: TestFramework("Xunit.Sdk.TestFrameworkProxy", "xunit.execution.dotnet")]
