---
id: detail.lock_yaml.immutable_archive
axis: meta
phase: detail
kind: policy
status: draft
depends_on:
  - detail.lock_yaml.artifact_lock_naming_convention
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes:
    - v1_temporal_safety_proof
---

# immutable archive 体系

## 一文方針
- 全 build artifact lock ファイル + audit hash chain root + formal proof artifact + cosign signature + SBOM + SLSA provenance attestation を、Ceph RGW Object Lock Compliance mode + WORM retention 10〜15 year + 外部公証 attestation で immutable 保管する。root 権限でも改竄不能、GDPR crypto-shred との互換性を維持する。

## immutable archive 対象
- 全 build artifact lock ファイル（`*.lock.yaml`）
- audit hash chain root（tier2 audit_local の年次 root）
- formal proof artifact（TLA+ trace / Stainless certificate / Dafny .doo / Lean .olean / Kani report / CBMC report）
- cosign signature + SBOM（Syft）+ SLSA provenance attestation（in-toto）
- Witness multi-attestation chain
- Rekor inclusion proof

## 物理 storage（Ceph RGW Object Lock Compliance mode）
- Object Lock Compliance mode: root 権限でも解除不可
- WORM retention: 10〜15 year（規制要件次第）
- bucket 単位の設定: tenant 別 / artifact_class 別

## 外部公証 attestation
- audit hash chain root: RFC 3161 trusted timestamp + Sigstore transparency log
- formal proof artifact: Cosign signed + Sigstore Rekor inclusion proof
- Witness multi-attestation chain: SLSA L4

## archive 階層
- **hot**: ClickHouse（audit signal SoR、近 90 日）
- **warm**: ClickHouse（90 日〜1 年、cold storage 移行前）
- **cold**: Ceph RGW Object Lock Compliance mode（1 年〜10 年）
- **archive_to_offline**: tape / 外部 archive service（10 年〜、規制要件次第）

## GDPR crypto-shred 互換性
- immutable archive は cryptographically shielded（KEK shamir で wrap）
- GDPR 物理削除時、当該 tenant の KEK destroy で wrapped DEK を不可逆失効
- archive 自体は immutable（root 権限でも変更不可）だが、復号不能化により実質削除

## 5 層 defense-in-depth
- 層 A: cosign signed + SBOM 生成（compile 時）
- 層 B: lint（archive 対象の網羅検査）
- 層 C: restore_drill（cold archive からの復元検証、年次）
- 層 D: runtime（Ceph RGW Object Lock Compliance mode + WORM retention）
- 層 E: 外部公証 attestation（RFC 3161 + Sigstore + Witness multi-attestation chain）

## CI 不変条件
- 全 lock ファイルが Cosign signed
- audit hash chain root の外部公証 attestation 完備
- formal proof artifact の reviewer dual sign-off + cosign signature
- restore_drill last_green_at が cadence 内
- crypto-shred 互換 property test green（全 share destroy で wrapped DEK 不可読化）

## 1.0.0 ship blocker
- Ceph RGW Object Lock Compliance mode の物理設定 green
- WORM retention 設定 green
- 外部公証 attestation 経路 green
- crypto-shred 互換 property test green
- restore_drill 7 日連続 verify green

## 採用しない設計
- mutable archive
- root 権限で変更可能な audit log
- WORM retention 設定なしの長期保管
- 外部公証 attestation 不在の audit hash chain root
- Object Lock Compliance mode 以外（Governance mode は root 権限で解除可、不採用）

## 関連参照
- [05_lock_yaml 体系 README](README.md)
- [release_gate 体系](03_release_gate体系.md)
- [artifact_lock 命名規約](04_artifact_lock命名規約.md)
- [鍵管理適合仕様](../01_適合仕様/05_鍵管理適合仕様.md)
- [KEK Shamir 分散](../03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [build_provenance 適合仕様](../01_適合仕様/16_build_provenance適合仕様.md)
