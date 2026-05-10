---
id: arch.security.pii_protection_policy
axis: security
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.security.security_index
  - arch.security.threat_model_policy
  - detail.security.security_enforcement
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes: []
lock_artifacts:
  - pii_classification.lock.yaml
---

# security PII 保護方針

## 一文方針
- PII / 機微情報の class taxonomy（v1_pii）を本層が宣言し、tier2/22 で分類された各 column が tier1/11 鍵階層 × tier1/12 schema breaking 判定 × tier2/32 RLS FORCE × 09_data preservation_class × 39 client 暗号化 × 09 監査 の 6 軸全てに必ず物理 mitigation を持つことを cross-axis に lock し、いずれかの mitigation が欠落する PII column の merge を CI fail で物理拒否する。

## 至高路線における立ち位置
- PII column の plaintext 運用禁止、application-layer envelope encryption 必須
- PII class taxonomy の opt-out 禁止、全 column が pii_class 宣言を持つ
- business 都合の PII rest 解除禁止、解除には breaking 判定経由の major version bump + tier1/11 鍵階層変更 PR が必要
- third-party SaaS への PII export を v1 では禁止

## PII class taxonomy（5 class）

### v1_pii_basic（氏名 / 住所 / 電話 / 端末 ID）
- rest: envelope encryption（DEK）
- in-transit: TLS 1.3 / mTLS
- retention: preservation_class が `v1_zone_replicated` 以上、archive 7 年まで
- access: tier2 RLS FORCE + 業務 entity scope
- audit: data_access fact 必須
- client: 39 reconcile_class 別、IndexedDB 上は暗号化必須

### v1_pii_special（人種 / 信条 / 病歴 / 犯罪歴）
- rest: envelope encryption + DEK rotation 30 日
- in-transit: TLS 1.3 / mTLS + per-tenant scope
- retention: 必要最小限、業務不要時は即時論理消去経路（tombstone）+ 1 年以内 crypto-erase
- access: RLS FORCE + 二人原則（変更時 reviewer 必須、tier1/10 step-up auth 必須）
- audit: data_access fact + intent_class（参照目的明示）必須
- client: client cache 禁止、毎回 server 取得

### v1_pii_finance（口座 / クレカ番号）
- rest: envelope encryption + tokenization（PAN は tokenize、Format-Preserving Encryption は採用しない）
- in-transit: mTLS
- retention: PCI-DSS 準拠（v2、v1 では PAN を保持しない architecture）
- access: tier2 PII service 経由のみ、application 直接 join 禁止
- audit: 常時 RequestResponse audit
- client: 禁止（tokenize 後の reference のみ）

### v1_pii_credential（password hash / OTP seed / KEK ID / DEK ID）
- rest: bcrypt/argon2id（password）/ HSM-backed（KEK）/ envelope（DEK）
- in-transit: mTLS、application 経路で plaintext 禁止
- retention: incident 時 crypto-erase 即時可能
- access: 認証 daemon のみ、業務 application 不可
- audit: authn fact + secret_lifecycle fact

### v1_pii_health_or_minor（未成年 / 健康 / 位置情報）
- rest: v1_pii_special と同等 + 地域別 residency 規律（v2、v1 では同 cluster 内に集約）
- in-transit: mTLS + per-tenant scope
- retention: v1_pii_special と同等
- access: v1_pii_special と同等 + intent_class 厳格
- audit: data_access + access purpose 必須
- client: v1_pii_special と同等

## 6 軸 cross-axis bind（PII class × 軸 = mitigation 表）

| 軸 | 内容 |
|---|---|
| 軸 1: tier1/11 鍵階層 | 各 PII class に DEK class（KEK 経由 wrap）を bind、rotation cadence は class 別に短縮（special / finance は 30 日、basic は 90 日）|
| 軸 2: tier1/12 schema breaking 判定 | PII class 列の rest 暗号化解除 / class 緩和 / column 削除を breaking として block、Apicurio rule に `pii_class_only_strengthens` rule を追加 |
| 軸 3: tier2/32 RLS FORCE | PII 列を含む table は POLICY + FORCE_RLS 必須、tenant_id 列との joint RLS で物理隔離 |
| 軸 4: 09_data preservation_class | PII class 別 minimum preservation_class を物理 lock（v1_pii_basic ≥ v1_zone_replicated、v1_pii_special ≥ v1_cross_region_replicated 等）|
| 軸 5: 39 client 状態 | reconcile_class 別 client 暗号化要件を bind、IndexedDB rest 暗号化 + 端末ロック解除 step-up を要求 |
| 軸 6: 09 audit subject | PII 列の data_access は audit_event の data_access fact + intent_class 必須、access purpose を ML anomaly detection の入力に |

## taxonomy の宣言経路
- PII class は schema 列レベルで Apicurio Registry の extension（`pii_class`）として宣言、build artifact（`pii_classification.lock.yaml`）に集約
- 全 column の `pii_class` が `unspecified` のままは schema 登録 admission block（必ず `pii_class = none` か `v1_pii_xxx` を明示）
- taxonomy 変更 PR は security 層 reviewer 必須（CODEOWNERS で物理 enforce）

## 採用しない方針
- PII column の plaintext 運用: 禁止
- PII class taxonomy の opt-out: 禁止
- business 都合の PII rest 解除: 禁止
- PII の application log への混入: 禁止、Vector pipeline の masking rule（PII pattern detect → redact）を物理 enforce
- third-party SaaS への PII export: v1 では禁止
- PII の client cache 自由化: 禁止、reconcile_class が要求する暗号化 + lifetime 制限

## 強制機構との bind
- 本方針は [security 強制機構](../../04_詳細設計/02_強制機構/06_security強制機構.md) の以下経路で物理 enforce される:
    - 層 A: jsonschema による pii_classification.lock.yaml schema 検証
    - 層 B: Conftest による全 column pii_class 注釈 check
    - 層 D: Kyverno `require-pii-class-annotation` `require-encryption-on-pii` `require-rls-on-pii-table` `require-step-up-auth-on-special-pii` admission policy
    - 層 E: envelope encryption の HSM-backed DEK 物理層 enforce

## 関連参照
- [security 設計方針 index](README.md)
- [脅威モデル方針](01_脅威モデル方針.md)
- [秘密管理方針](03_秘密管理方針.md)
- [監査方針](05_監査方針.md)
- [tier1 鍵管理適合仕様](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
