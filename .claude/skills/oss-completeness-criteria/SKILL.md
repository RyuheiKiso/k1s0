---
name: oss-completeness-criteria
description: k1s0 の OSS 完成度を採用検討者視点で自己採点する時に参照する skill。OSSF Scorecard 18 項目 / CNCF Sandbox 採用基準 / OpenSSF Best Practices Badge (Passing 17 / Silver / Gold) の 3 つの外部基準を統合した自己採点プロセスと、`/audit oss` での自動化範囲を定める。
---

# OSS 完成度 自己採点プロセス

本 Skill は k1s0 が「OSS として採点に耐えるか」を **第三者基準で測る**手順を定める。`audit_criteria.md` §D 軸の正典を補完し、`tools/audit/lib/oss.sh` の機械検証範囲外の項目を人間が採点する手順を提供する。

## 大前提: 「完璧」は到達不能

`audit_criteria.md` で既に明記したとおり、本プロジェクトは「OSS として完璧」という到達不能な目標を採らない。代わりに **「採用検討者が信頼できる水準」** を以下の合算で判定する：

- **OSSF Scorecard**: 7/10 以上
- **CNCF Sandbox 採用基準**: 最低要件 PASS
- **OpenSSF Best Practices Badge**: Passing 達成、Silver を視野

「完璧」と書く代わりに、各基準への充足度を **相対値**で記録する。

## 3 つの外部基準

### 基準 A: OSSF Scorecard（18 項目）

[OpenSSF Scorecard](https://scorecard.dev/) は 18 項目を機械的にスコアリングする。public repo + scorecard-cli が前提。

#### Scorecard 項目（k1s0 自己採点用チェックリスト）

| 項目 | 説明 | k1s0 における自己採点 |
|---|---|---|
| Code-Review | PR が承認 + 履歴管理されているか | GitHub Branch Protection 設定（public 化後に確認） |
| Maintained | 90 日以内の commit / Issue 活動 | 直近 30 日 commits 数を `tools/audit/lib/oss.sh` で機械化 |
| CII-Best-Practices | Best Practices Badge 取得状況 | 基準 C で別途採点 |
| License | OSI 承認 license の有無 | `LICENSE` 内の SPDX 識別子で機械判定可能 |
| Signed-Releases | リリースバイナリの cosign 署名 | `ops/supply-chain/signatures/` の存在 + `.github/workflows/_reusable-push.yml` の cosign sign-blob 経路 |
| Branch-Protection | main / リリースブランチの保護 | GitHub Settings の API（public 化後）、現状は `repo-settings.md` で文書化 |
| Token-Permissions | GitHub Actions の `permissions:` 制限 | `.github/workflows/*.yml` の grep で確認 |
| Pinned-Dependencies | 依存を SHA で pinning しているか | `renovate.json` + go.sum / Cargo.lock / pnpm-lock.yaml の存在 |
| Vulnerabilities | 既知脆弱性の有無 | dependabot / renovate alert（public 化後） |
| Binary-Artifacts | repo 内のビルド済バイナリ | `git ls-files | grep -E '\.exe$|\.dll$'` 等（k1s0 はバイナリを commit しない方針） |
| SAST | 静的解析の有無 | `.github/workflows/_reusable-lint.yml` で各言語 linter を実行 |
| Security-Policy | SECURITY.md の存在 + 報告経路明示 | `SECURITY.md` 内の `mailto:` / `vulnerability-disclosure` 経路を文字列で確認 |
| Fuzzing | fuzz target の存在 | `tests/fuzz/` 配下に Go std fuzzing / Rust standalone harness（SHIP_STATUS E3 / G7）|
| Dependency-Update-Tool | Renovate / Dependabot の設定 | `renovate.json` 存在で OK |
| Webhooks | webhook 認証 | k1s0 は public 化前なので N/A |
| Dangerous-Workflow | GHA 内の `pull_request_target` 等の危険な構成 | `.github/workflows/` 内 grep |
| CI-Tests | PR ごとの CI 実行 | `.github/workflows/pr.yml` の path-filter 構成 |
| Contributors | 異なる組織からの contributor | `git log --format=%ae | sort -u` で email domain を確認 |

#### 自己採点手順

1. public repo 化前は **手動チェック**: 各項目を上記表に従って Met / Unmet / N/A で判定
2. public repo 化後は `scorecard-cli --repo=https://github.com/<owner>/<repo>` で機械採点
3. スコア 7/10 を「採用検討者が信頼できる水準」の閾値とする
4. 不足項目は ADR / Issue 化して採用初期 / 採用後の運用拡大時に解消

### 基準 B: CNCF Sandbox 最低要件

[CNCF Sandbox 採用基準](https://github.com/cncf/sandbox)。k1s0 が CNCF Sandbox に申請する場合（または採用組織が CNCF 互換性を要件にする場合）の必須項目。

#### CNCF Sandbox 必須項目

| 項目 | k1s0 における対応 |
|---|---|
| OSI 承認 license | `LICENSE`（Apache-2.0、OSI 承認、`tools/audit/lib/oss.sh` で機械判定） |
| Code of Conduct | `CODE_OF_CONDUCT.md`（Contributor Covenant 準拠が望ましい） |
| Contributing Guide | `CONTRIBUTING.md`（PR / Issue の手順） |
| Governance | `GOVERNANCE.md`（meritocratic / 意思決定プロセス） |
| Security Policy | `SECURITY.md`（vulnerability 報告経路） |
| 公開 issue tracker | GitHub Issues（public 化後） |
| 公開 release tag | `git tag` + GitHub Releases（v0 リリース時） |
| 公開 roadmap | `docs/01_企画/` または `ROADMAP.md` |
| Vendor-neutral | 単一 vendor / 単一組織に依存しない | 採用組織複数の見込み |
| Trademark policy | OSS 商標の利用方針 | リリース時点で明文化 |

#### 自己採点手順

1. ファイル存在の機械チェック → `tools/audit/lib/oss.sh`（自動化済み）
2. 中身の品質確認（governance の意思決定プロセス記述、SECURITY の報告期限等）→ 人間レビュー
3. Vendor-neutral / Trademark policy → 採用組織複数化後に再評価

### 基準 C: OpenSSF Best Practices Badge

[OpenSSF Best Practices Badge Program](https://www.bestpractices.dev/)。Passing（17 項目）→ Silver → Gold の 3 段階。

#### Passing 17 項目の領域

1. **Basics**（公開 repo / OSS license / 説明文 / VCS 利用）
2. **Change Control**（公開 VCS / 半年以上の history / 一意なバージョン番号）
3. **Reporting**（vulnerability 報告経路 / 半年以内の対応実績）
4. **Quality**（build / 自動テスト / 新機能テスト / warning フリービルド）
5. **Security**（cryptography 適切利用 / hardening / TLS / static analysis）
6. **Analysis**（static analyzer / warnings 対応）

#### 自己採点手順

1. 外部サイト https://www.bestpractices.dev/ で repo URL を入力
2. 各項目を Met / Unmet / Unknown / N/A で自己申告
3. 全項目 Met で Passing badge 取得
4. Silver / Gold を目指す場合、追加項目（Two-person review / Documentation / Crypto requirements 等）を順次満たす

## 自動化範囲と手動範囲

`tools/audit/lib/oss.sh` で機械化する範囲：

- ✅ ファイル存在（CNCF Sandbox 6 ファイル）
- ✅ ファイルサイズ（簡易的な「中身の量」指標）
- ✅ git 統計（直近 30 日 commits / 全 contributors 数）
- ✅ scorecard-cli 不在の検出（保留扱い）
- ✅ LICENSE 種別判定（OSI 承認 license の SPDX 識別子確認）
- ✅ SECURITY.md 内の vulnerability 報告経路（`mailto:` / URL）の存在確認
- ✅ `.github/` 系（CODEOWNERS / labels.yml / workflows / repo-settings.md）の存在確認
- ✅ SBOM / cosign 署名（`ops/supply-chain/{sbom,signatures}/` の存在確認）
- ✅ Pinned-Dependencies（renovate.json + lock files の存在）
- ✅ Fuzzing（`tests/fuzz/` の存在確認）
- ✅ SAST（`.github/workflows/_reusable-lint.yml` の存在確認）
- ✅ Binary-Artifacts（commit 済バイナリの検出）

人間が採点する範囲（自動化不能 or public repo 必須）：

- ❌ Branch-Protection（GitHub API、public 化後）
- ❌ Code-Review（PR 履歴の分析、public 化後）
- ❌ Vulnerabilities（dependabot alert、public 化後）
- ❌ Token-Permissions の妥当性判定（YAML 構文 OK と意味論 OK は別）
- ❌ Dangerous-Workflow の安全性判定
- ❌ Best Practices Badge の自己申告（外部サイト操作）

## 採点運用ルール

### Claude が守るべき原則

1. **「完璧」「全部 OK」を書かない**: `audit-protocol` skill の原則に整合。各項目は Met / Unmet / Unknown / N/A の 4 値で記録
2. **public repo 化前の判定は「Unknown」優先**: scorecard-cli / dependabot 等が動かない項目は、根拠なく Met と書かず Unknown と明記
3. **証跡の保存**: 各項目の判定根拠（grep 結果 / find 結果 / ファイル参照）を `.claude/audit-evidence/<date>/oss-checklist.txt` に保存
4. **数値で語る**: 「Passing 17 項目中 N 項目 Met / M 項目 Unmet / K 項目 Unknown」の分数で示す

### 人間が守るべき原則

1. **public 化の前と化後の両方で再採点**: public repo 化したら scorecard-cli を即実行、判定を更新
2. **Best Practices Badge は外部サイトで自己採点**: https://www.bestpractices.dev/ で repo URL を入力、各項目を申告
3. **不足項目を ADR / Issue に昇格**: Unmet 項目は採用初期 / 採用後の運用拡大時に解消する計画を持つ
4. **採用検討者向けに公開**: 採点結果を `docs/AUDIT.md` D 軸に commit して公開

## 退路と段階運用

OSS 完成度は **段階的に上げる前提**で設計する。リリース時点で全 18 項目 Met を目指さない。

| 段階 | 目標 |
|---|---|
| リリース時点 (v0) | CNCF Sandbox 最低要件 Met / OSSF Scorecard 5/10 / Best Practices Passing 9/17 |
| 採用初期 | OSSF Scorecard 7/10 / Best Practices Passing 17/17 |
| 採用後の運用拡大時 | OSSF Scorecard 9/10 / Best Practices Silver |
| 長期目標（任意） | OSSF Scorecard 10/10 / Best Practices Gold / CNCF Sandbox 申請 |

各段階で何を達成すべきかを `docs/SHIP_STATUS.md` の段階表に紐付けて運用する。

## 関連

- 判定基準: [`docs/00_format/audit_criteria.md`](../../../docs/00_format/audit_criteria.md) §D 軸
- 監査スナップショット: [`docs/AUDIT.md`](../../../docs/AUDIT.md) D 軸
- 監査ロジック: `tools/audit/lib/oss.sh`
- 連携 skill: `audit-protocol`（PASS を勝手に書かない原則）/ `principal-architect-mindset`（Layer 3 SoT 整合 + 業界標準への適合）

## 参考文献

- OSSF Scorecard: scorecard.dev
- OpenSSF Best Practices Badge: bestpractices.dev
- CNCF Sandbox 採用基準: github.com/cncf/sandbox
- CNCF Project Maturity Levels: cncf.io/project-maturity
- Linux Foundation Security Considerations: linuxfoundation.org/security
