# attestations/samples — Bootstrap fixtures

本番 attestation の出力先（`attestations/` 直下）とは分離した samples ディレクトリ。
CI pipeline が cosign attest / SLSA provenance / Notation CLI で生成した本番 artifact は
`attestations/` 直下に配置され、`.gitignore` で Git 管理外とする。

## 各 sample の用途と検証コマンド

### slsa_provenance_v1.0.sample.intoto.jsonl
SLSA Provenance v1.0 の JSON-L 形式サンプル。
検証: `cat slsa_provenance_v1.0.sample.intoto.jsonl | jq .`

### cosign_attest.sample.bundle
cosign attest --predicate で生成される bundle のサンプル。
検証: `cosign verify-blob --bundle cosign_attest.sample.bundle <artifact_path>`

### notation_sign.sample.signature
Notation CLI で生成されるシグネチャのサンプル。
検証: `notation verify --signature notation_sign.sample.signature <artifact_path>`
