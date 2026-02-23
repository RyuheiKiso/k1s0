# k1s0-config

k1s0 設定管理 Swift ライブラリ

JSON 設定ファイルの読み込みと検証を提供します。

## 使い方

```swift
import K1s0Config

let url = Bundle.main.url(forResource: "config", withExtension: "json")!
let config = try ConfigLoader.load(from: url)
try ConfigLoader.validate(config)
print(config.app.name)
```

## 開発

```bash
swift build
swift test
```
