# ops/supply-chain — SBOM + cosign signatures (ADR-SUP-001)

本ディレクトリは k1s0 OCI イメージの SBOM（Software Bill of Materials）と
cosign 署名を保管する。**ADR-SUP-001（SLSA v1.1 Build Track Level 2）** の
リリース時点到達状態を実装する。

## 構造

```text
ops/supply-chain/
├── README.md                 # 本ファイル
├── sbom/                     # syft で生成した SBOM（CycloneDX 1.6 + SPDX）
│   ├── k1s0-tier1-state.cyclonedx.json
│   ├── k1s0-tier1-state.spdx.json
│   ├── k1s0-tier1-secret.cyclonedx.json
│   ├── k1s0-tier1-secret.spdx.json
│   ├── k1s0-tier1-workflow.cyclonedx.json
│   ├── k1s0-tier1-workflow.spdx.json
│   ├── k1s0-tier1-audit.cyclonedx.json
│   ├── k1s0-tier1-audit.spdx.json
│   ├── k1s0-tier1-decision.cyclonedx.json
│   ├── k1s0-tier1-decision.spdx.json
│   ├── k1s0-tier1-pii.cyclonedx.json
│   └── k1s0-tier1-pii.spdx.json
├── signatures/               # cosign sign-blob bundle
│   └── *.cyclonedx.json.bundle
└── keys/                     # オフライン検証用鍵 + signing-config
    ├── cosign.pub            # 公開鍵（commit OK）
    ├── cosign.key            # 秘密鍵（COSIGN_PASSWORD="" で生成、本リポジトリ用）
    └── signing-config.json   # offline signing-config（rekor / fulcio 無効）
```

## 生成手順（再現性）

```bash
cd /home/ryuhei_kiso/github/k1s0

# 1) SBOM 生成（CycloneDX 1.6 + SPDX、syft 1.43.0）
for img in tier1-state tier1-secret tier1-workflow tier1-audit tier1-decision tier1-pii; do
  syft "k1s0-${img}:dev" \
    -o "cyclonedx-json=ops/supply-chain/sbom/k1s0-${img}.cyclonedx.json" \
    -o "spdx-json=ops/supply-chain/sbom/k1s0-${img}.spdx.json"
done

# 2) cosign 鍵ペア生成（オフライン署名用、初回のみ）
cd ops/supply-chain/keys
COSIGN_PASSWORD="" cosign generate-key-pair
cosign signing-config create \
  --no-default-rekor --no-default-fulcio --no-default-oidc \
  --out signing-config.json
cd -

# 3) 各 SBOM を sign-blob で署名（bundle 形式 = 署名 + 証明書 + メタデータ一括）
export COSIGN_PASSWORD=""
for img in tier1-state tier1-secret tier1-workflow tier1-audit tier1-decision tier1-pii; do
  cosign sign-blob --yes --key ops/supply-chain/keys/cosign.key \
    --bundle "ops/supply-chain/signatures/k1s0-${img}.cyclonedx.json.bundle" \
    --signing-config ops/supply-chain/keys/signing-config.json \
    "ops/supply-chain/sbom/k1s0-${img}.cyclonedx.json"
done
```

## 検証手順（採用検討者向け）

任意の SBOM ファイルが「k1s0 リポジトリで生成されたものから改ざんされていないこと」を
公開鍵だけで検証できる。

```bash
# 公開鍵 + bundle で SBOM の真正性を検証
cosign verify-blob \
  --key ops/supply-chain/keys/cosign.pub \
  --bundle ops/supply-chain/signatures/k1s0-tier1-state.cyclonedx.json.bundle \
  --insecure-ignore-tlog \
  --insecure-ignore-sct \
  ops/supply-chain/sbom/k1s0-tier1-state.cyclonedx.json
# → "Verified OK"
```

改ざん検出例（1 byte 末尾追加）:

```bash
cp ops/supply-chain/sbom/k1s0-tier1-state.cyclonedx.json /tmp/tampered.json
echo "X" >> /tmp/tampered.json
cosign verify-blob \
  --key ops/supply-chain/keys/cosign.pub \
  --bundle ops/supply-chain/signatures/k1s0-tier1-state.cyclonedx.json.bundle \
  --insecure-ignore-tlog --insecure-ignore-sct \
  /tmp/tampered.json
# → Error: invalid signature when validating ASN.1 encoded signature
```

## 本番運用との関係

本ディレクトリは「リポジトリ同梱のオフライン検証経路」を確立するためのもので、
**本番デプロイは別経路** で署名する:

- 本番イメージは GitHub Actions hosted runner 上で build される
- 署名は cosign の **keyless（GitHub OIDC トークン経由）** で発行され、Rekor 透過ログに記録される
- Kyverno ImageVerify policy（`infra/security/kyverno/image-verify.yaml`、ADR-CICD-003）が
  本番 namespace で署名 + provenance attestation + SBOM attestation の 3 点を必須検証する

本リポジトリの cosign 鍵ペアは「リリース時点 OSS としての真正性証明」用で、長期鍵管理を
前提としない（COSIGN_PASSWORD 空 / 個人開発環境固定）。本番環境ではローテーション可能な
keyless 経路を採用する設計（ADR-SUP-001 §「リリース時点の到達状態」）。

## 直近実走実績（2026-04-30）

- 6 image（tier1-state / -secret / -workflow / -audit / -decision / -pii）について
  - syft 1.43.0 で **CycloneDX 1.6 + SPDX json** を生成（各 ~290–960 KB）
  - cosign v3.0.6 で **sign-blob bundle 形式** に署名（offline signing-config で rekor 無効）
  - 6 件すべて `cosign verify-blob` で **Verified OK** を確認
  - 改ざん検出: SBOM 末尾 1 byte 追加で signature 検証が **invalid ASN.1 encoded signature** で
    rejection することを実証
