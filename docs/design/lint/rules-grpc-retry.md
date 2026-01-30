# K030-K032: gRPC リトライ設定検査

← [Lint 設計書](./)

---

## K030: リトライ設定の検出（可視化）

```
重要度: Warning
目的: リトライ設定が存在することを開発者に認識させる
```

## K031: ADR 参照がない

```
重要度: Warning
目的: リトライ設定に関する設計決定が文書化されていることを確認
検査: コメントに ADR-XXXX への参照があるか
```

## K032: 設定が不完全

```
重要度: Warning
目的: リトライ設定の必須項目が揃っているか確認
```

**必須項目:**
- `max_attempts`: 最大リトライ回数
- `initial_backoff`: 初期バックオフ
- `max_backoff`: 最大バックオフ
- `backoff_multiplier`: バックオフ乗数
- `retryable_status_codes`: リトライ対象ステータスコード

## 検査例

```yaml
# gRPC リトライ設定
grpc:
  client:
    retry:
      # ADR-0005 参照  ← K031 OK
      max_attempts: 3
      initial_backoff: 100ms
      max_backoff: 1s
      backoff_multiplier: 2.0
      retryable_status_codes:
        - UNAVAILABLE
        - DEADLINE_EXCEEDED
```
