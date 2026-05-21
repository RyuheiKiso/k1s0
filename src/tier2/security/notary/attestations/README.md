# attestations/

# attestations ディレクトリの目的
このディレクトリは notary attestation の出力先です。
SLSA provenance や cosign attestation（`cosign attest`）が生成する JSON-L 形式の attestation バンドルを格納します。

# 生成タイミング
CI/CD パイプラインの build・sign フェーズで以下のコマンドが実行されるとファイルが生成されます:
- `cosign attest --predicate slsa_provenance.json <image>`
- `notary sign <artifact>` (Notation CLI)

# 手動編集禁止
このディレクトリの中身は CI が自動生成する build artifact です。手動での編集・コミットは禁止です。
生成物は `.gitignore` により git 管理外とすることを推奨します（CI artifact store / OCI registry に保存します）。

# 参照
- `docs/04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md`
- `src/security/cosign/` — cosign signing policy
