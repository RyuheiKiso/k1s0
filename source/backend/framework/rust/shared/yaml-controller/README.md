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
  name: service1
    url: http://service1.example.com
    port: 8080
  name: service2
    url: http://service2.example.com
    port: 8081
```

## メソッド

### read_yaml

yamlファイルを読み込み、内容を返します。
- 引数:
  - file_path: 読み込むyamlファイルのパス
  - key: 取得したいデータのキー（省略可能）
- 戻り値:
  - keyが指定された場合、そのキーに対応するデータ
  - keyが指定されなかった場合、yamlファイル全体の内容
- 例外:
    - FileNotFoundError: 指定されたファイルが存在しない場合
    - yaml.YAMLError: yamlのパースに失敗した場合
- 使用例:
```rust

// keyを指定してyamlデータを取得
let data = yaml_controller::read_yaml("config.yaml", Some("database"))?;
println!("{:?}", data);

// keyを指定せずにyaml全体を取得
let all_data = yaml_controller::read_yaml("config.yaml", None)?;
println!("{:?}", all_data);

```

### add_key_value

yamlファイルに新しいキーとその値を追加します。
- 引数:
  - file_path: 書き込むyamlファイルのパス
  - key: 追加するキー
  - value: 追加する値
- 戻り値:
  - 成功した場合はtrue、失敗した場合はfalse
- 例外:
    - IOError: ファイルの書き込みに失敗した場合
    - yaml.YAMLError: yamlのシリアライズに失敗した場合
- 使用例:
```rust

// yamlファイルに新しいキーと値を追加
let success = yaml_controller::add_key_value("config.yaml", "new_setting", "value")?;
if success {
    println!("Key-value pair added successfully.");
} else {
    println!("Failed to add key-value pair.");
}

``` 

## update_key_value

yamlファイル内の既存のキーの値を更新します。
- 引数:
  - file_path: 書き込むyamlファイルのパス
  - key: 更新するキー
  - new_value: 新しい値
- 戻り値:
  - 成功した場合はtrue、失敗した場合はfalse
- 例外:
    - IOError: ファイルの書き込みに失敗した場合
    - yaml.YAMLError: yamlのシリアライズに失敗した場合
- 使用例:
```rust

// yamlファイル内の既存のキーの値を更新
let success = yaml_controller::update_key_value("config.yaml", "database.host", "127.0.0.1")?;
if success {
    println!("Key-value pair updated successfully.");
} else {
    println!("Failed to update key-value pair.");
}

```

### delete_key

yamlファイルから指定されたキーを削除します。
- 引数:
  - file_path: 書き込むyamlファイルのパス
  - key: 削除するキー
- 戻り値:
  - 成功した場合はtrue、失敗した場合はfalse
- 例外:
    - IOError: ファイルの書き込みに失敗した場合
    - yaml.YAMLError: yamlのシリアライズに失敗した場合
- 使用例:
```rust

// yamlファイルから指定されたキーを削除
let success = yaml_controller::delete_key("config.yaml", "database.host")?;

if success {
    println!("Key deleted successfully.");
} else {
    println!("Failed to delete key.");
}

```