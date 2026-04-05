# ADR-0063: セッション暗号化における AAD（認証付加データ）の導入

## ステータス

承認済み

## コンテキスト

bff-proxy の `EncryptedStore`（`internal/session/encrypted_store.go`）は AES-256-GCM でセッションデータを暗号化している。
外部技術審査報告書（POLY-002）において、以下のセキュリティリスクが指摘された。

**セッションスワップ攻撃（Session Swap Attack）**:

AES-GCM は暗号文の完全性を保証するが、暗号化時に AAD（Additional Authenticated Data）を使用しない場合、
攻撃者が Redis 等のストレージ上で暗号文を別のキーにコピー・移動させても GCM タグ検証が通過してしまう。

具体的なシナリオ:
1. 攻撃者がユーザー A のセッション暗号文を取得する
2. その暗号文をユーザー B のセッション ID に対応するキーに書き込む
3. ユーザー B のセッション ID で復号するとユーザー A のセッションデータが取得される
4. 結果としてセッション固定・なりすましが可能になる

また、ワンタイム交換コード（mobile OAuth flow の `/auth/exchange`）も同様のリスクを持っていた。

実装対象: `regions/system/server/go/bff-proxy/internal/session/encrypted_store.go`

## 決定

AES-GCM の暗号化・復号関数に `aad []byte`（Additional Authenticated Data）パラメータを追加し、
セッション ID または交換コードを AAD として渡すことでバインディングを実現する。

```go
// 暗号化: AAD にセッション ID をバインド
ciphertext := gcm.Seal(nonce, nonce, plaintext, aad) // aad = []byte(sessionID)

// 復号: 同一の AAD で検証
plaintext, err := gcm.Open(nil, nonce, ciphertext, aad) // aad = []byte(sessionID)
```

呼び出し元ごとの AAD:
- `Create`, `Get`, `Update`: `[]byte(sessionID)`
- `CreateExchangeCode`, `GetExchangeCode`: `[]byte(code)`（ワンタイム交換コード）

## 理由

1. **最小変更**: `encrypt`/`decrypt` のシグネチャに `aad []byte` を追加するだけで対応可能。暗号アルゴリズムの変更は不要
2. **GCM の仕様活用**: AES-GCM の AAD 機能は暗号文をコンテキスト（セッション ID）に結びつけるために設計されている
3. **セッションスワップ防止**: AAD が異なれば GCM タグ検証が必ず失敗するため、暗号文の移動・コピーが無効化される
4. **後方互換性なし（意図的）**: AAD なしで暗号化された既存セッションは復号できなくなる。これは既存セッションの強制無効化を意味し、デプロイ時のユーザー再ログインを要求する

## 影響

**ポジティブな影響**:

- セッションスワップ攻撃が原理的に不可能になる
- 暗号文がそのセッション ID に完全にバインドされ、他の文脈で再利用できなくなる
- AES-GCM の設計意図に沿った正しい実装になる
- モバイル OAuth フローの交換コードも同様に保護される

**ネガティブな影響・トレードオフ**:

- デプロイ時に既存の全セッションが無効化される（AAD なし暗号文は復号できないため）
- ユーザーはデプロイ後に再ログインが必要になる
- Redis 上の既存暗号文を移行するマイグレーションは不要（セッションは短命のため）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 暗号化アルゴリズムを変更 | ChaCha20-Poly1305 等に変更 | 変更コストが大きく、AAD サポートは AES-GCM も同等に持つ |
| Redis キープレフィックスで分離 | セッション種別ごとに異なるキープレフィックスを使用 | 暗号レイヤーではなくストレージレイヤーでの対処であり、根本的解決にならない |
| セッションスワップを運用で防ぐ | Redis ACL でキーへの書き込みを制限 | 攻撃者がストレージにアクセスできる前提では不十分 |

## 参考

- [POLY-002] 外部技術審査報告書（2026-03-31）AES-GCM AAD 空の指摘
- [RFC 5116](https://datatracker.ietf.org/doc/html/rfc5116): An Interface and Algorithms for Authenticated Encryption — AAD の定義
- 実装ファイル: `regions/system/server/go/bff-proxy/internal/session/encrypted_store.go`
- 関連 ADR: [ADR-0059: BFF-Proxy セッション暗号化方式](./0059-bff-proxy-session-encryption.md)（存在する場合）
