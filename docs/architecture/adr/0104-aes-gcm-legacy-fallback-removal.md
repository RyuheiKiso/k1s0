# ADR-0104: AES-GCM レガシーフォールバック削除

## ステータス

承認済み

## コンテキスト

ADR-0090 において AES-256-GCM 暗号化に AAD（Additional Authenticated Data）を導入した。
この際、既存データ（AAD なしで暗号化された旧形式）との後方互換性を保つために
`aes_decrypt_with_legacy_fallback` 関数を Phase A（一時的移行措置）として実装した。

この関数は次のように動作していた:

1. まず AAD あり（新形式）で復号を試みる
2. 失敗した場合は AAD なし（旧形式 `b""`）でフォールバックする

Phase A の実装は `config` サービスと `notification` サービスの2か所で使用されており、
具体的な対象カラムは以下の通り:

- `config_entries.encrypted_value`（config サービス）
- `notification.channels.encrypted_config`（notification サービス）

Phase B（本番データの完全再暗号化）の完了をもって、このフォールバック機構は不要となる。
フォールバック関数が残存していると、旧形式データへの復号が誤って許容され続け、
NIST SP 800-38D 準拠の AAD 検証を迂回するリスクが残る。

## 決定

`aes_decrypt_with_legacy_fallback` 関数を削除し、`aes_decrypt`（AAD 必須）のみを使用する。

本番環境への適用前に、Phase B バッチ再暗号化スクリプトを実行してすべての旧形式データを
新形式（AAD あり）に変換することを必須とする。

### 削除対象

- `regions/system/library/rust/encryption/src/aes.rs` — 関数本体を削除
- `regions/system/library/rust/encryption/src/lib.rs` — `pub use` の re-export を削除
- `regions/system/library/rust/encryption/tests/encryption_test.rs` — フォールバック関連テスト2件を削除

### 呼び出し元の更新

- `regions/system/server/rust/config/src/adapter/repository/config_postgres.rs`
  — `aes_decrypt_with_legacy_fallback` を `aes_decrypt` に置換（2箇所）
- `regions/system/server/rust/notification/src/adapter/repository/channel_postgres.rs`
  — `aes_decrypt_with_legacy_fallback` を `aes_decrypt` に置換（1箇所）、インポートも更新

### 本番用 Phase B スクリプト仕様

本番デプロイ前に以下のバッチ再暗号化を実施すること:

**対象テーブル・カラム**:

| テーブル | カラム | AAD |
|---------|-------|-----|
| `config_entries` | `encrypted_value` | `namespace` 値をバイト列として使用 |
| `notification.channels` | `encrypted_config` | `id` 値（チャンネル ID）をバイト列として使用 |

**手順**:

1. `is_encrypted = true` かつ `encrypted_value IS NOT NULL` のレコードを全件取得する
2. 現行の `aes_decrypt_with_legacy_fallback` で復号する（スクリプト用に一時保持）
3. 同一データを `aes_encrypt(key, plaintext, aad)` で再暗号化する
4. `UPDATE` で `encrypted_value` を新形式の暗号文に置換する
5. Phase B 完了後にこの ADR のデプロイを適用する

## 理由

- フォールバック関数が存在する限り、旧形式（AAD なし）の ciphertext が受け入れられ続ける
- ADR-0090 の目的である ciphertext swap attack 防止が不完全なままとなる
- Phase A は一時的措置と明記されており、本番データ移行後の削除が設計上の前提だった
- 削除後は `aes_decrypt` のみを公開することで、AAD なし復号への誤用を型レベルで排除できる

## 影響

**ポジティブな影響**:

- NIST SP 800-38D 準拠の AAD 検証が完全に強制される
- 旧形式データへの復号経路が消滅し、ciphertext swap attack リスクがゼロになる
- ライブラリの公開 API が簡潔になり、誤用リスクが低下する

**ネガティブな影響・トレードオフ**:

- 本番デプロイ前に Phase B バッチ再暗号化スクリプトの実行が必須となる
- Phase B 未実施の状態でデプロイすると、旧形式データの復号に失敗し サービスエラーが発生する
- 本番環境ではメンテナンスウィンドウ内での再暗号化作業が必要となる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: フォールバック関数を永続的に維持する | Phase B を実施せず、フォールバックを常設する | ADR-0090 の設計意図に反する。旧形式復号経路を永続化すると AAD の保護効果が失われる |
| 案 B: フォールバック関数に警告ログを追加して監視する | 旧形式検出時に warn! を出力する | ログ監視に依存しており根本解決にならない。フォールバック経路を削除する方が確実 |
| 案 C: フィーチャーフラグで切り替える | 環境変数でフォールバック有無を制御する | 設定ミスで本番にフォールバックが残るリスクがある。コード削除の方が確実性が高い |

## 参考

- [ADR-0090: AES-GCM AAD 導入](0090-aes-gcm-aad-introduction.md)
- NIST SP 800-38D: Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-05 | 初版作成 | @k1s0-team |
