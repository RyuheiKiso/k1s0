# ADR-0090: AES-GCM AAD（Additional Authenticated Data）導入

## ステータス

承認済み

## コンテキスト

encryption ライブラリ（`regions/system/library/rust/encryption`）の `aes_encrypt` / `aes_decrypt` は
AES-256-GCM 暗号化を実装しているが、AAD（Additional Authenticated Data）が未使用であった。

AAD なしの AES-GCM 実装には以下のリスクがある。

- **ciphertext swap attack**: 異なるレコードの暗号文を別のレコードに移植しても、鍵が同じであれば復号が成功してしまう
- **NIST SP 800-38D 違反**: GCM モードの標準的な使用では、暗号文のコンテキストを AAD で保護することが推奨されている

この encryption ライブラリは以下の 2 サービスで使用されている。

- `regions/system/server/rust/config` — namespace 単位の設定値暗号化
- `regions/system/server/rust/notification` — チャンネル設定（webhook URL・API キー等）暗号化

## 決定

`aes_encrypt` / `aes_decrypt` 関数に `aad: &[u8]` パラメータを追加し、
暗号化・復号の両操作で `aes_gcm::aead::Payload { msg, aad }` を使用する。

各呼び出し元での AAD 選択方針は以下のとおり。

| サービス | 暗号化対象 | AAD |
|--------|----------|-----|
| config-rust | config_entries テーブルの設定値 | `namespace`（例: `"system.auth"`） |
| notification-rust | channels テーブルのチャンネル設定 | `channel.id`（UUID 文字列） |

## 段階移行計画

### Phase A（本 ADR 対応）

新規暗号化はすべて AAD 付きで実施する。
既存の暗号化データ（AAD なしで暗号化済み）は復号時に認証タグ検証が失敗するため、
DBに既存暗号化データが存在する場合は再暗号化が必要となる。

現在のアーキテクチャでは Vault がシークレットを管理し、サービス起動時に鍵をロードする設計のため、
フォールバック（AAD なしで再試行）ではなく **データの再暗号化** を推奨する。

### Phase B（別途対応）

既存 DB 上の暗号化データを新 AAD 付き暗号文で一括更新するマイグレーションスクリプトを実行する。
対象テーブル:
- `config_entries.encrypted_value`（config-rust DB）
- `notification.channels.encrypted_config`（notification-rust DB）

## 理由

- **NIST SP 800-38D 準拠**: GCM モードでは AAD を使用して暗号文の使用コンテキストを認証することが推奨される
- **ciphertext swap attack の防止**: AAD に暗号化コンテキスト（namespace / channel ID）を含めることで、
  異なるレコードへの暗号文の流用を認証タグ検証が拒否するようになる
- **最小変更**: 関数シグネチャに `aad: &[u8]` を追加するだけで、既存の GCM 実装を変更せずに対応できる

## 影響

**ポジティブな影響**:

- AES-256-GCM の認証機能が暗号文のコンテキストも保護するようになり、セキュリティが向上する
- NIST SP 800-38D に準拠した実装になる
- ciphertext swap attack が不可能になる

**ネガティブな影響・トレードオフ**:

- 既存の AAD なし暗号化データとの後方互換性がない（Phase B の再暗号化対応が必要）
- config-rust・notification-rust の呼び出し元コードの修正が必要
- AAD を誤ったコンテキスト（例: 間違った namespace）で復号しようとすると失敗する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| AAD なし継続 | 現状維持 | NIST SP 800-38D 違反・ciphertext swap attack が可能なため却下 |
| フォールバック実装 | 復号失敗時に AAD なしで再試行 | AAD 導入の安全性向上効果が失われるため却下 |
| AES-GCM-SIV 移行 | nonce の再利用耐性を強化した GCM-SIV へ移行 | 別途 ADR-0092 で検討。本対応は AAD 追加の最小変更を優先する |

## 参考

- [NIST SP 800-38D: Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC](https://csrc.nist.gov/publications/detail/sp/800-38d/final)
- [ADR-0063: セッション暗号化 AAD 導入](./0063-aes-gcm-aad-session-binding.md)
- [ADR-0066: config サービス設定値 AES-256-GCM 暗号化](./0066-config-value-encryption.md)
- [ADR-0052: JSONB カラム暗号化戦略](./0052-jsonb-column-encryption.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（外部技術監査 C-001 対応） | @k1s0 |
