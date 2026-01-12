# yaml-controller

---

## 概要

本機能はyamlの読み込みと書き込みを行うコントローラーを提供します。

## yaml例
```yaml
settings:
  theme: dark

database:
  host: localhost
  port: 5432

services:
  - name: service1
    url: http://service1.example.com
    port: 8080
  - name: service2
    url: http://service2.example.com
    port: 8081
```

## yamlの配置場所

- 開発時: `../../config/*.yaml`
- 本番環境: `config/*.yaml`

優先度: 本番環境 > 開発時

本コントローラーは、配置場所が `*.yaml` を含むパターンで表現されるため、実際の操作時は対象となる**ファイル名（例: `config.yaml`）を明示的に指定する**ことを想定しています。`file_name` 引数には具体的なファイル名を必須で渡してください。省略した場合はエラーとなります。

## メソッド

### read_yaml

YAML ファイルを読み込み、内容を返します。
- 引数:
  - `file_name: &str`: 読み込む YAML ファイル名（必須）。
  - `key: Option<&str>`: 取得したいデータのキー（ドット区切り、省略可）。例: `database.host`
- 戻り値:
  - `Result<serde_yaml::Value, Box<dyn std::error::Error>>`: 成功時は `serde_yaml::Value` を返します。
    - `key` 指定時に該当キーが存在しない場合はエラーを返します（`Err(Box::new(KeyNotFoundError))` のような扱い）。
  - `Err(e)`: IO エラーやパースエラー（`std::io::Error`, `serde_yaml::Error` など）。
- 使用例:
```rust
// -> Result<serde_yaml::Value, Box<dyn std::error::Error>>
let data = yaml_controller::read_yaml("config.yaml", Some("database"))?; // 存在しない key は Err
println!("{:?}", data);

let all_data = yaml_controller::read_yaml("config.yaml", None)?; // YAML 全体
println!("{:?}", all_data);
```

### add_key_value

YAML ファイルに新しいキーと値を追加します。
- 引数:
  - `file_name: &str`: 書き込む YAML ファイル名（必須）。ファイルが存在しない場合は新規作成します。
  - `key: &str`: 追加するキー（ドット区切りでネスト可能）。
  - `value: impl serde::Serialize`: 追加する値（`serde::Serialize` を想定）。
- 戻り値:
  - `Result<bool, Box<dyn std::error::Error>>`: 成功時 `Ok(true)`（追加が行われた）、失敗時は `Err(...)`。
    - 指定キーが既に存在する場合はエラーを返します（`Err(Box::new(KeyAlreadyExistsError))` のような扱い）。
- 使用例:
```rust
// -> Result<bool, Box<dyn std::error::Error>>
let success = yaml_controller::add_key_value("config.yaml", "new_setting", "value")?; // 存在する key は Err
if success { println!("Key-value pair added successfully."); }
```

### update_key_value

YAML ファイル内の既存キーの値を更新します。
- 引数:
  - `file_name: &str`: 書き込む YAML ファイル名（必須）。
  - `key: &str`: 更新するキー（ドット区切り）。
  - `new_value: impl serde::Serialize`: 新しい値。
- 戻り値:
  - `Result<bool, Box<dyn std::error::Error>>`: 成功時は `Ok(true)`（更新が行われた）。
    - 指定キーが存在しない場合はエラーを返します（`Err(Box::new(KeyNotFoundError))` のような扱い）。
- 使用例:
```rust
let success = yaml_controller::update_key_value("config.yaml", "database.host", "127.0.0.1")?; // 存在しない key は Err
```

### delete_key

YAML ファイルから指定されたキーを削除します。
- 引数:
  - `file_name: &str`: 書き込む YAML ファイル名（必須）。
  - `key: &str`: 削除するキー（ドット区切り）。
- 戻り値:
  - `Result<bool, Box<dyn std::error::Error>>`: 成功時に `Ok(true)`（削除が行われた）。
    - 指定キーが存在しない場合はエラーを返します（`Err(Box::new(KeyNotFoundError))` のような扱い）。
- 使用例:
```rust
let success = yaml_controller::delete_key("config.yaml", "database.host")?; // 存在しない key は Err
if success { println!("Key deleted successfully."); }
```

---

## 実装に関する注意（README 注記）

- 書き込み操作は可能なら一時ファイルを使った原子書き込み（書き込み→リネーム）を行うことを推奨します。
- 同時書き込みの保護（ファイルロック等）が必要な場合は実装側で対策してください。
- 上記の戻り値の具体的型やキーが見つからなかった時の挙動は実装依存です。本 README は推奨仕様を示しています。実装に合わせて README を更新してください。