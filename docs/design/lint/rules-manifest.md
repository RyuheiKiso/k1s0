# K001-K011: manifest 検査・必須ファイル検査

← [Lint 設計書](./)

---

## K001-K003: manifest 検査

### K001: manifest.json の存在確認

```
対象: .k1s0/manifest.json
重要度: Error
ヒント: k1s0 new-feature で生成したプロジェクトか確認してください
```

### K002: 必須キーの検査

**必須キー（Error）:**
- `k1s0_version`
- `template.name`
- `template.version`
- `template.fingerprint`
- `service.service_name`
- `service.language`

**必須キー（Warning）:**
- `managed_paths`
- `protected_paths`

### K003: 値の妥当性検査

**service.language:**
```rust
const VALID_LANGUAGES: &[&str] = &["rust", "go", "csharp", "python", "typescript", "dart"];
```

**service.service_type:**
```rust
const VALID_TYPES: &[&str] = &["backend", "frontend", "bff"];
```

**template.name:**
```rust
const VALID_TEMPLATES: &[&str] = &[
    "backend-rust",
    "backend-go",
    "backend-csharp",
    "backend-python",
    "frontend-react",
    "frontend-flutter",
];
```

---

## K010-K011: 必須ファイル検査

### RequiredFiles

```rust
pub struct RequiredFiles {
    /// 必須ディレクトリ
    pub directories: Vec<&'static str>,
    /// 必須ファイル
    pub files: Vec<&'static str>,
}

impl RequiredFiles {
    /// テンプレート名から必須ファイルを取得
    pub fn from_template_name(name: &str) -> Option<Self>;
}
```

### backend-rust（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "config",
        "deploy/base",
        "deploy/overlays/dev",
        "deploy/overlays/stg",
        "deploy/overlays/prod",
    ],
    files: vec![
        "Cargo.toml",
        "README.md",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
    ],
}
```

### backend-rust（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src/domain",
        "src/application",
        "src/infrastructure",
    ],
    files: vec![
        "Cargo.toml",
        "README.md",
        "src/lib.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
    ],
}
```

### backend-go（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "internal/domain",
        "internal/application",
        "internal/infrastructure",
        "internal/presentation",
        "config",
    ],
    files: vec![
        "go.mod",
        "README.md",
        "cmd/main.go",
        "config/default.yaml",
    ],
}
```

### backend-go（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "internal/domain",
        "internal/application",
        "internal/infrastructure",
    ],
    files: vec![
        "go.mod",
        "README.md",
    ],
}
```

### backend-csharp（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src",
        "config",
        "deploy/base",
    ],
    files: vec![
        "README.md",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
    ],
}
```

### backend-csharp（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![],
    files: vec![
        "README.md",
    ],
}
```

### backend-python（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src",
        "config",
        "deploy/base",
    ],
    files: vec![
        "pyproject.toml",
        "README.md",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
    ],
}
```

### backend-python（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![],
    files: vec![
        "pyproject.toml",
        "README.md",
    ],
}
```

### frontend-react（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "src/pages",
        "src/components/layout",
        "config",
    ],
    files: vec![
        "package.json",
        "README.md",
        "src/main.tsx",
        "src/App.tsx",
        "config/default.yaml",
    ],
}
```

### frontend-react（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src/domain",
        "src/application",
    ],
    files: vec![
        "package.json",
        "README.md",
        "tsconfig.json",
    ],
}
```

### frontend-flutter（feature 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "lib/src/domain",
        "lib/src/application",
        "lib/src/infrastructure",
        "lib/src/presentation",
        "config",
    ],
    files: vec![
        "pubspec.yaml",
        "README.md",
        "lib/main.dart",
        "config/default.yaml",
    ],
}
```

### frontend-flutter（domain 層）の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "lib/src/domain",
        "lib/src/application",
    ],
    files: vec![
        "pubspec.yaml",
        "README.md",
    ],
}
```
