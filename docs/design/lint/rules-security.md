# K050-K053: セキュリティ検査

← [Lint 設計書](./)

---

## K050: SQL インジェクションリスク検出

### 目的

SQL 文の文字列補間・文字列結合によるクエリ構築を検出し、パラメータバインドの使用を強制する。

### 検出パターン

**Rust** (`.rs`):
```
format!("SELECT ...
format!("INSERT ...
format!("UPDATE ...
format!("DELETE ...
```
※ 大文字・小文字の両方を検出

**Go** (`.go`):
```
fmt.Sprintf("SELECT ...
fmt.Sprintf("select ...
"SELECT " +
"INSERT " +
"UPDATE " +
"DELETE " +
```
※ 大文字・小文字の両方を検出

**TypeScript** (`.ts`, `.tsx`, `.js`, `.jsx`):
```
`SELECT ${...
`INSERT ${...
`UPDATE ${...
`DELETE ${...
"SELECT " +
"INSERT " +
"UPDATE " +
"DELETE " +
```
※ テンプレートリテラルと文字列結合の両方を検出

**C#** (`.cs`):
```
$"SELECT ...
$"INSERT ...
$"UPDATE ...
$"DELETE ...
"SELECT " +
"INSERT " +
"UPDATE " +
"DELETE " +
```
※ 文字列補間と文字列結合の両方を検出

**Python** (`.py`):
```
f"SELECT ...    f'SELECT ...
f"INSERT ...    f'INSERT ...
f"UPDATE ...    f'UPDATE ...
f"DELETE ...    f'DELETE ...
"SELECT " +
"INSERT " +
"UPDATE " +
"DELETE " +
"SELECT ".format(
"INSERT ".format(
"UPDATE ".format(
"DELETE ".format(
```
※ f-string、文字列結合、str.format() の3パターンを検出

**Dart** (`.dart`):
```
'SELECT $...    "SELECT $...
'INSERT $...    "INSERT $...
'UPDATE $...    "UPDATE $...
'DELETE $...    "DELETE $...
```

### 除外

- コメント行はスキップされる
- 同一行で複数パターンがマッチしても1つのみ報告

### 違反例

```python
# K050 違反: f-string による SQL 構築
query = f"SELECT * FROM users WHERE id = {user_id}"
```

### 正しい実装

```python
# パラメータバインドを使用
query = "SELECT * FROM users WHERE id = $1"
result = await conn.fetch(query, user_id)
```

---

## K053: ログへの機密情報出力検出

### 目的

ログ出力に機密情報（パスワード、トークン、シークレット等）が含まれることを検出し、情報漏洩を防止する。

### 機密キーワード

```
password, token, secret, api_key, apikey, credential, private_key
```

### 安全とみなされるサフィックス

以下のサフィックスが付いた変数名は安全とみなされ、検出対象外となる:

```
_hash, _hashed
```

### 検出対象のログ関数

**Rust** (`.rs`):
```
tracing::info!(, tracing::warn!(, tracing::error!(, tracing::debug!(, tracing::trace!(
info!(, warn!(, error!(, debug!(, trace!(
log::info!(, log::warn!(, log::error!(, log::debug!(, log::trace!(
```

**Go** (`.go`):
```
log.Print(, log.Printf(, log.Println(
slog.Info(, slog.Warn(, slog.Error(, slog.Debug(
zap.Info(, zap.Warn(, zap.Error(, zap.Debug(
logger.Info(, logger.Warn(, logger.Error(, logger.Debug(
```

**TypeScript** (`.ts`, `.tsx`, `.js`, `.jsx`):
```
console.log(, console.warn(, console.error(, console.info(, console.debug(
logger.info(, logger.warn(, logger.error(, logger.debug(
```

**C#** (`.cs`):
```
logger.LogInformation(, logger.LogWarning(, logger.LogError(, logger.LogDebug(, logger.LogTrace(
_logger.LogInformation(, _logger.LogWarning(, _logger.LogError(, _logger.LogDebug(
Log.Information(, Log.Warning(, Log.Error(, Log.Debug(
```

**Python** (`.py`):
```
logging.info(, logging.warning(, logging.error(, logging.debug(
logger.info(, logger.warning(, logger.error(, logger.debug(
print(
```

**Dart** (`.dart`):
```
log(, print(, debugPrint(
logger.i(, logger.w(, logger.e(, logger.d(
```

### 検出ロジック

1. 行がログ関数呼び出しを含むか判定
2. 同じ行に機密キーワードが含まれるか判定（大文字小文字区別なし）
3. キーワードの直後に安全サフィックス（`_hash`, `_hashed`）がある場合は除外

### 除外

- コメント行はスキップされる

### 違反例

```go
// K053 違反: ログにパスワードを出力
slog.Info("user login", "password", user.Password)
```

### 正しい実装

```go
// 機密情報を除外してログ出力
slog.Info("user login", "user_id", user.ID)
```
