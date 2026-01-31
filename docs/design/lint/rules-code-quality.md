# K020-K022: コード品質検査

← [Lint 設計書](./)

---

## K020: 環境変数参照の禁止

### 目的

環境変数の直接参照を禁止し、設定ファイル経由での設定読み込みを強制する。

### 検査対象パターン

**Rust** (対象拡張子: `.rs`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "std::env::var",
    "std::env::var_os",
    "std::env::vars",
    "std::env::vars_os",
    "std::env::set_var",
    "std::env::remove_var",
    "env::var(",
    "env::var_os(",
    "env::vars(",
    "env::set_var(",
    "env::remove_var(",
    "dotenv",
    "dotenvy",
];
```

**Go** (対象拡張子: `.go`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "os.Getenv",
    "os.LookupEnv",
    "os.Setenv",
    "os.Unsetenv",
    "os.Environ",
    "godotenv",
];
```

**TypeScript** (対象拡張子: `.ts`, `.tsx`, `.js`, `.jsx`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "process.env",
    "import.meta.env",
    "dotenv",
];
```

**C#** (対象拡張子: `.cs`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "Environment.GetEnvironmentVariable",
    "Environment.GetEnvironmentVariables",
    "Environment.ExpandEnvironmentVariables",
    ".AddEnvironmentVariables(",
];
```

**Python** (対象拡張子: `.py`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "os.environ",
    "os.getenv",
    "os.putenv",
    "os.unsetenv",
    "load_dotenv",
    "from dotenv",
    "import dotenv",
];
```

**Dart** (対象拡張子: `.dart`):
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "Platform.environment",
    "fromEnvironment",
    "flutter_dotenv",
];
```

### 除外パターン

- テストファイル（`*_test.rs`, `*_test.go`, `*.test.ts`, `*_test.dart`）
- `--env-var-allowlist` で指定されたファイル

### 違反例

```rust
// K020 違反: 環境変数の直接参照
let db_url = std::env::var("DATABASE_URL").unwrap();
```

### 正しい実装

```rust
// k1s0-config を使用
let config = ConfigLoader::new(options)?;
let db_config: DbConfig = config.load()?;
```

---

## K021: config YAML への機密直書き禁止

### 目的

機密情報を YAML ファイルに直接記述することを禁止する。

### 検査対象キー

```rust
const SECRET_KEY_PATTERNS: &[&str] = &[
    "password",
    "secret",
    "api_key",
    "apikey",
    "api-key",
    "token",
    "credential",
    "private_key",
    "privatekey",
    "private-key",
];
```

### 許可されるパターン

```rust
// OK: _file サフィックスで外部ファイルを参照
password_file: /var/run/secrets/db-password

// OK: 空値
password:

// OK: プレースホルダ
password: ${DB_PASSWORD}  # 環境変数展開は別の仕組みで
```

### 違反例

```yaml
# K021 違反: 機密情報の直接記述
database:
  password: my-secret-password
```

### 正しい実装

```yaml
# _file サフィックスで外部ファイルを参照
database:
  password_file: /var/run/secrets/k1s0/db-password
```

---

## K022: Clean Architecture 依存方向違反

### 目的

Clean Architecture の依存方向ルールを強制する。

### 依存ルール

```
外側 → 内側 のみ許可

presentation → application → domain ← infrastructure

禁止される依存:
- domain → application
- domain → presentation
- domain → infrastructure
- application → presentation
```

### 検査方法

ファイルパスとインポート文を解析して依存方向を検査する。

```rust
// domain 層のファイルで application をインポート
// K022 違反
mod domain {
    use crate::application::services::UserService;  // NG
}
```

### 層の判定

```rust
fn get_layer(path: &str) -> Option<Layer> {
    if path.contains("/domain/") { Some(Layer::Domain) }
    else if path.contains("/application/") { Some(Layer::Application) }
    else if path.contains("/presentation/") { Some(Layer::Presentation) }
    else if path.contains("/infrastructure/") { Some(Layer::Infrastructure) }
    else { None }
}
```

### C# の依存方向検査

C# プロジェクトでは `.csproj` ファイルの `<ProjectReference>` を解析して依存方向を検証します。

```xml
<!-- Domain プロジェクトが Application を参照 → K022 違反 -->
<ProjectReference Include="..\MyService.Application\MyService.Application.csproj" />
```

Include パスからプロジェクト名（層名）を抽出し、禁止依存パターンを検証します。

### Python の依存方向検査

Python ファイル（`.py`）の import 文を解析して依存方向を検証します。

```python
# domain 層のファイルで application をインポート → K022 違反
from application.services import UserService  # NG
import application.usecases  # NG
from .application import something  # NG
from ..application import something  # NG
```

検出パターン:
- `from {layer}` / `import {layer}`
- `from .{layer}` / `from ..{layer}`
