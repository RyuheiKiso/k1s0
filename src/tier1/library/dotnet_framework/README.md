# k1s0 tier1 Library .NET Framework 4.6.2+ Companion

docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §scenario assertion §TLS バージョン注記 に準拠する。

## 対象

- .NET Framework 4.6.2 以降（TLS 1.2 がデフォルトで有効になる最低バージョン）
- tier3 Legacy（`.NET Framework 4.6.2+`）向け互換ラッパー

## TLS 注記

.NET Framework 4.6.2 は TLS 1.2 を既定で有効化する。
ただし TLS 1.3 は .NET 5.0 以降でサポートされる。
Legacy 環境では TLS 1.2 を明示的に設定することを推奨する:

```csharp
// TLS 1.2 を明示的に有効化する（.NET Framework 4.6.2+ 推奨設定）
System.Net.ServicePointManager.SecurityProtocol =
    System.Net.SecurityProtocolType.Tls12;
```

## 提供クラス

- `AuthContextCompat` — `K1s0.Tier1.AuthContext` の .NET Framework 4.6.2+ 互換ラッパー
- `KeyHandleCompat` (`IKeyHandleCompat` / `OpenBaoKeyHandleCompat`) — `K1s0.Tier1.KeyHandle` の .NET Framework 4.6.2+ 互換ラッパー

## 参照

- docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md
- docs/04_詳細設計/01_適合仕様/04_認証適合仕様.md
- docs/04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md
