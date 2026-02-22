# K1s0.System.Config

YAML ベースの設定管理ライブラリ。基本設定ファイルの読み込み、環境別設定のディープマージ、バリデーション、Vault シークレットの上書きを提供する。

## インストール

```xml
<ProjectReference Include="path/to/K1s0.System.Config.csproj" />
```

## 使用例

### 基本的な読み込み

```csharp
using K1s0.System.Config;

// 基本設定のみ
var config = ConfigLoader.Load("config.yaml");

// 環境別設定をマージ
var config = ConfigLoader.Load("config.yaml", "config.prod.yaml");
```

### DI 登録

```csharp
using K1s0.System.Config;

services.AddK1s0Config("config.yaml", "config.prod.yaml");

// コンストラクタインジェクション
public class MyService(AppConfig config)
{
    // config.App.Name, config.Server.Port etc.
}
```

### Vault シークレットのマージ

```csharp
var secrets = new Dictionary<string, string>
{
    ["database.password"] = "secret-from-vault",
};
var updated = VaultHelper.MergeVaultSecrets(config, secrets);
```

## 設定ファイル例

```yaml
app:
  name: my-service
  version: "1.0.0"
  tier: system          # system | business | service
  environment: dev      # dev | staging | prod
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: info
    format: json
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
```

## バリデーション項目

- `app.name` は必須
- `app.version` は必須
- `app.tier` は `system`, `business`, `service` のいずれか
- `app.environment` は `dev`, `staging`, `prod` のいずれか
- `server.host` は必須
- `server.port` は 1 以上
- `auth.jwt.issuer` は必須（auth セクションがある場合）
- `auth.jwt.audience` は必須（auth セクションがある場合）
