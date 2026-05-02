# oncall/sops-key — SOPS AGE 鍵運用

本ディレクトリは SOPS（[Mozilla SOPS](https://github.com/getsops/sops)）の AGE 暗号鍵を管理する。
オンコール担当者だけが読取可能な機密ファイル（`contacts-adopters.yaml` / 採用組織 API 鍵 / PagerDuty token 等）の暗号化に使用する。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md) と
[ADR-SEC-002（OpenBao）](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md) に対応する。

## 配置（予定）

```text
sops-key/
├── README.md          # 本ファイル
├── age.txt.enc        # AGE 秘密鍵（OpenBao Transit で暗号化、git に commit 可）
└── recipients.txt     # 公開鍵リスト（オンコール担当の AGE 公開鍵）
```

## 鍵の生成と運用

リリース時点では起案者単独運用のため、AGE 鍵を 1 つ生成する。採用後の運用拡大時 で複数オンコール担当者に鍵を分散する。

### 初回鍵生成（起案者）

```bash
# AGE キーペア生成
age-keygen -o /tmp/age-key.txt
# /tmp/age-key.txt の中身:
#   # public key: age1xxxx...
#   AGE-SECRET-KEY-1xxxx...

# 公開鍵を recipients.txt に追記
grep "public key:" /tmp/age-key.txt | awk '{print $4}' >> ops/oncall/sops-key/recipients.txt

# 秘密鍵を OpenBao Transit で暗号化して age.txt.enc として commit
bao write transit/encrypt/k1s0-sops plaintext=$(base64 < /tmp/age-key.txt) \
  | jq -r '.data.ciphertext' > ops/oncall/sops-key/age.txt.enc

# /tmp/age-key.txt は使い切ったら shred で消去
shred -u /tmp/age-key.txt
```

### 復号して使う（オンコール担当）

```bash
# OpenBao Transit で復号（要 k1s0-oncall ポリシー）
ENC=$(cat ops/oncall/sops-key/age.txt.enc)
PLAIN=$(bao write -field=plaintext transit/decrypt/k1s0-sops ciphertext="${ENC}")
echo "${PLAIN}" | base64 -d > ~/.config/sops/age/keys.txt
chmod 600 ~/.config/sops/age/keys.txt

# SOPS で暗号化されたファイルを編集
sops ops/oncall/contacts-adopters.yaml
```

### ローテーション

- **頻度**: 12 か月ごと、または鍵保有者の体制変更時に随時。
- **手順**:
  1. 新規 AGE キーペア生成（上記「初回鍵生成」と同手順）。
  2. 既存 SOPS 暗号化ファイルを `sops updatekeys <file>` で新公開鍵を追加。
  3. 旧鍵の保有者（退職・体制変更）を `recipients.txt` から削除し、`sops updatekeys` で旧鍵を除外。
  4. 旧 `age.txt.enc` を git 履歴から `git filter-repo` で完全削除（コンプライアンス要件）。
  5. ローテ完了を四半期 SRE レビューで記録。

## 暗号化対象（リリース時点で SOPS 化が確定しているもの）

- `ops/oncall/contacts-adopters.yaml` — 採用組織別の連絡先（個人情報含む）
- `ops/oncall/rotation/*.yaml` の secret セクション（PagerDuty API token 等）
- `infra/security/openbao/` の sealed config（`bao operator init -key-shares=5 -key-threshold=3` の出力）

## 緊急時のキー紛失対応

- 単一鍵紛失時: バックアップ（OpenBao Transit に常駐）から復号可能。`age.txt.enc` 自体が使えなくなった場合は OpenBao の transit 鍵から再生成。
- OpenBao Transit 鍵紛失時: SEV1 として [`../../runbooks/secret-rotation.md`](../../runbooks/secret-rotation.md) §「OpenBao unseal share」の手順に従い、Shamir 5/3 で再構成する。

## 関連

- 関連 Runbook: [`../../runbooks/secret-rotation.md`](../../runbooks/secret-rotation.md)（特に §「OpenBao unseal share」）
- 関連 ADR: [ADR-SEC-002（OpenBao）](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md)
- 関連設計書: [`docs/05_実装/85_Identity設計/secrets-matrix.md`](../../../docs/05_実装/85_Identity設計/secrets-matrix.md)
