# Secret Rotation Runbook（骨格）

本 runbook は k1s0 で扱う各 secret 種別の **定期ローテーション手順** と **漏洩発生時の即時対応** を集約する。`docs-design-spec` Skill の Runbook 規約（タイプ C: 検出 / 初動 / 復旧 / 原因調査 / 事後処理）に従う。

> **本ファイルはリリース時点 骨格版**。実機検証と実 incident からの学びは plan 14-01 の Runbook 整備フェーズで本実装する。

## 関連設計

- [docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md](../../docs/05_実装/85_Identity設計/secrets-matrix.md)
- [ADR-SEC-002（OpenBao）](../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md)
- [`infra/security/openbao/policies/`](../../infra/security/openbao/policies/)

## 共通原則

1. **rotation は OpenBao 中心**。tier1 facade は新値を Pod の rolling restart で読み直す。
2. **二段階移行**: 新値書込 → アプリが新旧両方読める期間 → 旧値削除。
3. **観測性**: rotation 前後で SLO 違反が発生していないか Grafana で確認。
4. **不可逆な操作は 4-eyes**: prod の OpenBao での `bao secret delete` は admin 2 名承認。

## 定期 rotation 周期一覧

| secret 種別 | 周期 | 自動化可否 |
|---|---|---|
| DB password | 90 日 | ✅ OpenBao Database secrets engine で自動払出 |
| Kafka SASL | 30 日 | ✅ Strimzi KafkaUser CR で再生成 |
| OIDC client secret | 180 日 | △ Keycloak Operator + 手動切替 |
| TLS 証明書 | 90 日 | ✅ cert-manager 自動更新 |
| OIDC keyless（cosign） | 不要 | ✅ Sigstore が短命証明書を毎回発行 |
| GHCR PAT | 90 日 | △ 採用側で手動更新 |
| OpenBao unseal share | 365 日 | ❌ Shamir share の手動再分散 |

## 種別別の rotation 手順（骨格）

### 1. DB password（CloudNativePG）

**検出**: 漏洩 incident 報告 / 90 日 SLA 超過の Mimir alert

**初動（〜30 分）**:
1. OpenBao で新パスワードを生成 (`bao write -force database/rotate-role/k1s0-tier1`)
2. CNPG cluster にも反映 (`kubectl edit cluster k1s0-pg -n k1s0-data`)

**復旧（〜2 時間）**:
1. tier1 facade を rolling restart で新値を取得 (`kubectl rollout restart deployment/tier1-facade`)
2. `tools/local-stack/status.sh` 相当で接続確認

**原因調査**:
- 漏洩経路の特定（gitleaks ログ / アクセス log）
- 横展開の可能性検討

**事後処理**:
- postmortem を `ops/runbooks/postmortems/` に
- secret-matrix への追記（影響範囲拡張時）

### 2. Kafka SASL credentials

**初動**: KafkaUser の Secret 削除 → User Operator が再払出
```bash
kubectl delete secret <user>-secret -n k1s0-data
# Strimzi User Operator が自動再生成
```

### 3. OIDC client secret（Keycloak）

**初動**: Keycloak admin で client secret regenerate → OpenBao に書込 → tier1 rolling restart

### 4. TLS 証明書

cert-manager の自動更新（`renewBefore: 720h`）に任せる。手動更新が必要な緊急時:
```bash
kubectl delete certificate <name> -n <ns>
# cert-manager が即時再発行
```

### 5. OIDC keyless（cosign）

鍵 rotation 不要（Sigstore Fulcio が毎回新証明書発行）。万一 Sigstore Rekor に偽 sign が記録された場合の対応:
1. 該当 image を GHCR から削除
2. 採用側に advisory 通知
3. `cosign verify` で真正な sign のみ通すよう Kyverno policy で署名 issuer を限定

### 6. GHCR PAT

採用側組織の手順で更新。GitHub UI で PAT 発行 → OpenBao 書込 → image-pull-secret 更新。

### 7. OpenBao unseal share

**最重要 / 最頻度低**:
- Shamir 5/3 構成、過半数で unseal 可能
- 紛失時はその share の保有者を変更 → 全 share を再分散
- 詳細: `ops/runbooks/forensics/04_key-compromise.md`（plan 13-09）で詳述

## 漏洩発生時の即時対応

漏洩を検知した secret 種別ごとに、以下の SLA で対応する。

| 種別 | 即時対応 | 推奨 SLA |
|---|---|---|
| DB password | OpenBao で rotate → tier1 rolling restart | 1 時間 |
| Kafka SASL | KafkaUser Secret 削除 → 再払出 | 1 時間 |
| OIDC client secret | Keycloak で regenerate | 4 時間 |
| TLS 証明書 | cert-manager Certificate を delete → 再発行 | 即時 |
| cosign 鍵（dev fixture） | dev fixture を新世代に置換 | 24 時間 |
| GHCR PAT | revoke → 更新 | 即時 |
| OpenBao unseal share | 該当 share の owner 変更 + 再分散 | 24 時間 |

## 関連

- [docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md](../../docs/05_実装/85_Identity設計/secrets-matrix.md)
- [`.gitleaks.toml`](../../.gitleaks.toml) — 漏洩検出 rule
- [`infra/security/openbao/policies/`](../../infra/security/openbao/policies/) — OpenBao policy
- [plan 13-05 secret scan / 13-09 Forensics Runbook](../../plan/13_セキュリティとサプライチェーン/)
