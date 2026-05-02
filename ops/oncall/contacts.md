# SEV1 連絡先一覧

本ファイルは SEV1 インシデント対応で必要となる連絡先を集約する。
[`escalation.md`](escalation.md) のフロー実行で本ファイルを参照する。

> **取扱注意**: 本ファイルには個人連絡先・組織内連絡先が含まれる。
> Kyverno ClusterPolicy `restrict-oncall-contacts-read`（[`infra/security/kyverno/`](../../infra/security/kyverno/) で管理）で
> 該当 namespace 外からの読取を制限する。git 上では `<要置換>` プレースホルダを使い、実値はリリース時点の運用開始時に
> SOPS（[`sops-key/`](sops-key/)）で暗号化したうえで注入する。

連絡先の最新化は四半期ごとに実施し、変更は PR + 起案者承認で行う。

## 1. 技術エスカレーション

| 役割 | 連絡先 | 手段 | 期限 |
|------|--------|------|------|
| CTO | `<cto-tel-要置換>` / `<cto@org.example>` | 電話 → Slack DM | 即時 |
| Engineering Manager | `<em-oncall@org.example>` | Slack DM | 即時 |
| Security SRE 当番 | PagerDuty schedule `k1s0-security-oncall` | PagerDuty escalation | 即時 |
| プラットフォーム担当 | PagerDuty schedule `k1s0-platform-oncall` | PagerDuty | 即時 |

PagerDuty Schedule の実体は [`rotation/`](rotation/) で Terraform 管理。

## 2. セキュリティ・法務

| 役割 | 連絡先 | 手段 | 期限 |
|------|--------|------|------|
| 法務担当 | `<legal@org.example>` + `<legal-tel-要置換>` | メール + 電話 | SEV1 確定後 1h |
| 個人情報保護責任者（CPO） | `<privacy@org.example>` | 電話 | PII 漏えい疑い時即時 |
| 採用組織セキュリティ担当 | `<customer-security@<adopter>.example>`（採用先ごと別途台帳） | メール | テナント越境確認後 2h |
| 外部法律顧問 | `<outside-counsel@<law-firm>.example>` | 法務担当判断で起動 | 法務判断 |

採用組織ごとの個別連絡先は採用契約時に締結し、`contacts-adopters.yaml`（SOPS 暗号化、本リポジトリには平文を置かない）で管理する。

## 3. 外部機関

| 機関 | 連絡先 | 条件 |
|------|--------|------|
| 個人情報保護委員会 | https://www.ppc.go.jp/personalinfo/legal/leakAction/ | PII 漏えい確定後 72h 以内（速報）、30 日以内（確報） |
| JPCERT/CC | https://www.jpcert.or.jp/form/ | サイバー攻撃の疑い |
| 警察（サイバー犯罪相談窓口） | 都道府県警察本部 サイバー犯罪相談窓口（管轄に応じて） | 不正アクセス禁止法違反が疑われる場合 |
| GDPR 対象 EU データ漏えい時 | 各国 DPA — Lead supervisor がいない場合は Ireland DPC を窓口に | GDPR Art.33（72h 以内） |

## 4. インフラ・ベンダ

| ベンダ | 連絡先 | 条件 |
|------|--------|------|
| Cloud Provider（GCP / AWS / Azure） | サポートチケット URL（採用契約に応じて） | クラウド側障害疑い |
| Cloudflare | エンタープライズサポート（プラン契約時） | DNS / DDoS 防御 |
| GitHub Enterprise | https://support.github.com/ | GHCR / Actions 障害 |
| 採用 OSS 商用サポート（Strimzi / CNPG / Istio 等） | サポート契約に応じて | OSS バグ起因 |

## 5. 採用組織内連絡経路

| 経路 | 詳細 |
|------|------|
| Slack `#status` | 顧客向けステータス公開チャンネル（読取は全社員） |
| Slack `#incident-<YYYYMMDD>-<slug>` | インシデントごとの作業チャンネル（IC が作成） |
| Slack `#incident-sev1` | SEV1 専用通知チャンネル（PagerDuty 連動） |
| Status Page | https://status.k1s0.example.com/ — IC または広報担当が更新 |

## 連絡先の最新化プロセス

- **頻度**: 四半期ごとに棚卸し（毎年 3 / 6 / 9 / 12 月最終週）。
- **責任者**: 当四半期の SRE オンコール lead。
- **手順**:
  1. 全行を確認し、無効になっているメール / 電話 / URL を特定。
  2. PR を作成し、起案者または EM のレビュー承認を得る。
  3. PagerDuty schedule の実体（[`rotation/`](rotation/)）も同時に確認。
  4. SOPS 暗号化された採用組織連絡先（`contacts-adopters.yaml`）も同期更新。

## 関連

- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md)
- エスカレーションフロー: [`escalation.md`](escalation.md)
- PagerDuty 当番表: [`rotation/`](rotation/)
- SOPS 鍵運用: [`sops-key/`](sops-key/)
