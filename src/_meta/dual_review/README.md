# dual sign-off 体系

k1s0 v1.0.0 の dual sign-off 規約に従い、全 Phase の実装が以下の二重 sign を満たすことを物理要件とする。

## sign-off 要件

| sign 種別 | 主体 | 方式 |
|---|---|---|
| human_cosign | 実装者本人 | `cosign sign-blob` + 公開鍵 fingerprint 記録 |
| ai_static_analysis | Claude Opus 4.7 (AI agent) | `cargo clippy -D warnings` + `semgrep --strict` + `codeql analyze` の pass 証拠 |

LLM 単独 sign-off 禁止（CLAUDE.md の dual sign-off 規約 / formal 整合 8 の規律を保つ）。

## cosign 公開鍵管理

cosign 署名の private key は OpenBao Transit に保管する（`src/security/openbao/transit_config.yaml` の `cosign_key` エントリ）。  
公開鍵 fingerprint は本 README 末尾に記録する。

## Phase 完了フロー

1. Phase X の実装完了後、実装者が以下を実行する:
   ```sh
   cosign sign-blob \
     --key hashivault://transit/sign/cosign_key \
     --tlog-upload=false \
     release_gate.lock.yaml \
     > release_gate.lock.yaml.sig
   ```
2. AI static analysis evidence (cargo clippy / semgrep / codeql) の CI pass を確認する。
3. `v1_implementation_review.dual_review.lock.yaml` の対応 phase entry を更新する。
4. `git tag -s v1.0.0-phase-X` を発行する（明示承認が必要）。

## sign-off 状態

現在の sign-off 状態は `v1_implementation_review.dual_review.lock.yaml` を参照すること。

## cosign 公開鍵 fingerprint

```
# OpenBao Transit から export した cosign 公開鍵の SHA-256 fingerprint
# (実際の fingerprint は cosign key generate 後に記録する)
fingerprint: PENDING_HUMAN_KEY_GENERATION
algorithm: ECDSA P-256
key_path: transit/keys/cosign_key
```
