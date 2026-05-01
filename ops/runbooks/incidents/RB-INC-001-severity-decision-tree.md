# Severity 判定フロー Runbook

> **severity**: SEV1〜SEV3（判定対象）
> **owner**: tier1-platform-team
> **estimated_mttr**: 15min（判定完了まで）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

以下のいずれかが起動トリガーとなる。

- Loki アラート（`alert_severity_*` ルール）が Slack `#incident-alert` チャンネルに着信
- 外部ユーザー・採用組織からの障害報告（Backstage サポートチケット / メール）
- 監視当番による手動検知（死活監視ダッシュボード Grafana `/d/k1s0-health`）
- CI/CD パイプライン失敗による本番デプロイ停止
- セキュリティツール（Falco / gitleaks）からの自動通報

## 2. 初動 (Immediate Action)

判定者はインシデント認知後 **15 分以内** に Severity を確定し、対応 Runbook を起動する。

### 判定フロー

```
┌─────────────────────────────────────────────────────┐
│ STEP 1: 影響サービスを特定                            │
│   kubectl get pods -A | grep -v Running              │
│   → 複数 tier または全テナントに影響？                │
└──────────────────┬──────────────────────────────────┘
                   │ Yes → SEV1 候補
                   │ No  → STEP 2 へ
┌──────────────────▼──────────────────────────────────┐
│ STEP 2: 継続時間とエラーレートを確認                  │
│   Grafana SLO ダッシュボード:                         │
│     - エラーバジェット消費率 > 20%/h  → SEV1         │
│     - エラーレート > 5% かつ > 10 分 → SEV2          │
│     - エラーレート < 5% または < 10 分 → SEV3        │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│ STEP 3: セキュリティ・法的影響の確認                  │
│   - PII / 個人情報の漏えい疑い？ → SEV1              │
│   - テナント越境アクセス検知？  → SEV1              │
│   - 法的開示要求（令状等）あり？ → SEV1              │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│ STEP 4: 経営報告閾値の確認                            │
│   SEV1: 即時 CTO へ電話報告、30 分以内に役員 Slack   │
│   SEV2: 1 時間以内に Engineering Manager へ Slack    │
│   SEV3: 当番が対応、翌朝の定例で報告                 │
└─────────────────────────────────────────────────────┘
```

### Severity 判定基準早見表

| 指標 | SEV1 | SEV2 | SEV3 |
|------|------|------|------|
| サービス全停止 | Yes | - | - |
| 複数テナント影響 | Yes | 部分的 | 単一テナント |
| エラーバジェット消費 | >20%/h | 5〜20%/h | <5%/h |
| 継続時間 | >15min | 10〜60min | <10min |
| PII 漏えい疑い | Yes | - | - |
| テナント越境 | Yes | - | - |
| MTTR 目標 | 2h | 8h | 24h |
| 経営報告 | CTO 即時 | EM 1h以内 | 定例報告 |

## 3. 復旧 (Recovery)

1. Severity 確定後、該当 Runbook を **5 分以内** に起動する。
   - SEV1 セキュリティ系: `pii-leak-detection.md` / `tenant-boundary-breach.md`
   - SEV1 可用性系: 可用性 Runbook（別ファイル）
   - SEV2: `auth-abuse-detection.md` / `error-code-alert-policy.md`
   - SEV3: `error-code-alert-policy.md`
2. インシデント指揮者（Incident Commander）を指名する（SEV1/2 は必須）。
3. 対応進捗を Slack `#incident-<YYYYMMDD>-<slug>` チャンネルで共有する（SEV1 は 15 分ごと）。
4. Severity ダウングレード条件: SLO 回復 + 根本原因が封じ込め済みであること。

## 4. 原因調査 (Root Cause Analysis)

- Loki ログで影響開始時刻を特定する。

  ```bash
  logcli query '{namespace="k1s0-tier1"}' --since=2h --limit=1000 \
    | grep -E "(ERROR|CRITICAL)" | head -50
  ```

- Grafana でエラーレート急上昇タイミングと直前のデプロイを照合する。
- Argo CD で直近 2 時間のデプロイ履歴を確認する。

  ```bash
  argocd app history k1s0-tier1 --last 10
  ```

- SEV1 は 24 時間以内にポストモーテムのドラフトを起こす。

## 5. 事後処理 (Post-incident)

- ポストモーテム作成（SEV1: 24h / SEV2: 72h / SEV3: 1 週間）
- Severity 判定精度のレビュー（過剰判定・過少判定を記録）
- Runbook へのフィードバック反映 PR
- 四半期ごとに判定基準を見直し、アラートルールと整合させる

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-SIR-001)
- 関連設計書: docs/03_要件定義/30_非機能要件/I_SLI_SLO_エラーバジェット.md
- 関連 ADR: ADR-SEC-001 (Keycloak), ADR-SEC-002 (OpenBao)
- 関連 Runbook: ../../oncall/escalation.md, error-code-alert-policy.md
