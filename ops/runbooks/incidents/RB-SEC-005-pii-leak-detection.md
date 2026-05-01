---
runbook_id: RB-SEC-005
title: 個人情報漏えい検知・初動封じ込め
category: SEC
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: k1s0_pii_exfil_detected (Falco) / PiiLeakSuspect (Loki)
fmea_id: 間接対応
estimated_recovery: 封じ込め 24h / 個人情報保護委員会速報 72h / 確報 30 日
last_updated: 2026-05-02
---

# RB-SEC-005: 個人情報漏えい検知・初動封じ込め

本 Runbook は PII（個人情報）の漏えい検知時の初動・封じ込め手順を定める。
封じ込めは 24 時間以内、個人情報保護委員会への速報は 72 時間以内が法定期限（個人情報保護法 26 条）。
NFR-E-SIR-002 / NFR-E-ENC-003 / NFR-G-CLS-002 に対応する。
規制報告は [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) を後段で起動する。

## 1. 前提条件

- 実行者は `security-sre` ClusterRole + `kcadm.sh`（Keycloak admin）+ MinIO Object Lock 操作権限を保持。
- 必要ツール: `kubectl` / `mc`（MinIO CLI）/ `kcadm.sh` / `logcli` / `gpg`（証跡暗号化用）。
- kubectl context が `k1s0-prod`。
- CTO・法務・CPO 連絡先（[`oncall/contacts.md`](../../oncall/contacts.md)）が手元にあること。
- MinIO `k1s0-forensics` バケットが Object Lock 有効状態で存在すること。

## 2. 対象事象

以下のいずれかが漏えい疑いのトリガー:

- Falco アラート: `k1s0_pii_exfil_detected`（PII パターンを含む異常なデータ転送）。
- Loki アラート: PII 含有レスポンスの外部 IP への送信検知。
- tier1 PII API での `NFR-G-CLS-002` スキャン結果アラート。
- 外部からの漏えい報告（ユーザー、セキュリティリサーチャー、CERT）。
- gitleaks による Git リポジトリへの PII コミット検知。
- `K1s0Error.InternalError` 急増 + ログに個人情報パターン。

検知シグナル:

```bash
# Falco アラートの確認
kubectl logs -n falco -l app.kubernetes.io/name=falco --since=1h \
  | grep -i "pii\|personal\|exfil" | tail -30
# PII スキャン最新結果
kubectl get job -n k1s0-security -l app=pii-scanner -o wide
```

通知経路: PagerDuty `security-sre` → Slack `#incident-sev1` → CPO 直通電話。

## 3. 初動手順（5 分以内）

最初の 5 分で漏えいの可能性を裏付け、SEV1 確定 + CPO 連絡を完了する。

```bash
# Falco アラートの直近 30 分を確認
kubectl logs -n falco -l app.kubernetes.io/name=falco --since=30m \
  | grep -i "pii\|personal\|exfil" | head -10
```

```bash
# 漏えい疑い API の Loki ログから tenant_id / user_id を抽出
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "LEAK_SUSPECT" | json | line_format "{{.tenant_id}} {{.user_id}} {{.path}}"' \
  --since=1h | head -10
```

ステークホルダー通知（必須）:

- SEV1 を即時宣言、Slack `#incident-sev1` に「PII 漏えい疑い、調査開始」を投稿。
- CPO へ電話、CTO へ電話、法務へメール（[`oncall/escalation.md`](../../oncall/escalation.md) §Step 3）。
- Status Page を「調査中」に更新（外部漏えいが裏付けられるまで詳細は伏せる）。

## 4. 原因特定手順

- Loki + Falco の相関分析で漏えい開始時刻を特定する。
- 漏えい経路を特定する（API バグ / 権限設定ミス / 内部 actor / 外部侵害）。
- 影響 API エンドポイントのコードレビューを実施する。
- Keycloak の認証・認可ログを調査し、不審なアクセスを確認する:

```bash
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | line_format "{{.userId}} {{.ipAddress}} {{.type}}"' \
  --since=72h | grep -v "LOGIN_SUCCESS" | head -50
```

よくある原因:

1. **API レスポンスでの過剰露出**: tenant_id 検証バグで他テナント PII を返却。[`RB-SEC-006`](RB-SEC-006-tenant-boundary-breach.md) と連鎖。
2. **ログへの平文出力**: アプリ コードでマスク漏れ。NFR-G-CLS-002 スキャナで検知される。
3. **内部 actor による漏えい**: Secret 大量読取と連鎖。[`RB-AUTH-002`](RB-AUTH-002-auth-abuse-detection.md) と並行。
4. **Git コミット流出**: gitleaks 検知。即時 git filter-repo で履歴削除 + 該当 secret rotate。
5. **外部攻撃による侵害**: SQLi / SSRF 等。Envoy WAF ログを確認。

エスカレーション: 原因が確定しなくても封じ込めを優先。CPO の判断で個人情報保護委員会速報の準備を開始（72h 期限）。

## 5. 復旧手順

### Step 1: 初期トリアージ（〜30 分）

1. 漏えいデータの種別を確認する（氏名 / メール / 電話 / マイナンバー / 要配慮個人情報）。
2. 漏えい経路の仮説を立てる。
3. 影響テナントを特定する:

```bash
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "pii" | json | line_format "{{.tenant_id}} {{.user_id}} {{.path}}"' \
  --since=24h | sort | uniq -c | sort -rn | head -20
```

4. SEV1 を宣言し、[`oncall/escalation.md`](../../oncall/escalation.md) に従って CTO・法務・CPO に連絡する。

### Step 2: 通信封じ込め（〜1 時間）

漏えい経路と疑われるエンドポイントを NetworkPolicy でブロック:

```bash
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: quarantine-pii-leak
  namespace: k1s0-tier1
spec:
  podSelector:
    matchLabels:
      app: <suspicious-service>
  policyTypes:
  - Egress
  egress: []
EOF
```

該当 Pod の外部通信を即時遮断:

```bash
kubectl cordon <node>  # 必要に応じてノードを隔離
kubectl scale deployment/<suspicious-service> --replicas=0 -n k1s0-tier1
```

### Step 3: 証跡保全（〜2 時間）

影響期間のログを MinIO Object Lock バケットにコピーして改ざん防止:

```bash
mc cp --recursive \
  minio/k1s0-logs/tier1/$(date +%Y-%m-%d) \
  minio/k1s0-forensics/incident-$(date +%Y%m%d-%H%M)/
mc legalhold set minio/k1s0-forensics/incident-$(date +%Y%m%d-%H%M)/ ON
```

Pod のメモリダンプを取得する（可能な場合）:

```bash
kubectl debug -it <pod-name> -n k1s0-tier1 \
  --image=ghcr.io/nicolaka/netshoot -- sh
```

### Step 4: 影響範囲の確定（〜8 時間）

影響ユーザー数・テナント数をクエリで特定:

```bash
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "LEAK_SUSPECT" | json | line_format "{{.user_id}}"' \
  --since=48h | sort -u > /tmp/affected_users.txt
wc -l /tmp/affected_users.txt
```

漏えいデータの種別・件数を暗号化済スプレッドシートに記録（GPG または SOPS で暗号化）。

### Step 5: 段階的復旧（〜24 時間）

1. 封じ込め完了後、遮断した NetworkPolicy / Deployment を段階的に復旧。
2. 影響テナントのアクセストークンを無効化、再認証を促す:

```bash
kcadm.sh delete sessions/<session-id> -r k1s0
# または realm 全体のセッション無効化（重大な場合のみ）
kcadm.sh delete clients/<client-id>/user-sessions -r k1s0
```

3. 影響ユーザーへの個別通知を準備する（[`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) で実施）。
4. PII スキャンを再実行して漏えいが終息していることを確認する。

## 6. 検証手順

復旧完了の判定基準:

- Falco アラート `k1s0_pii_exfil_detected` が 24h 間継続して未発火。
- PII スキャナ Job 最新実行が `Succeeded` で漏えい指標 0 件。
- 直近 24h の Loki クエリ `{namespace="k1s0-tier1"} |= "LEAK_SUSPECT"` が 0 件。
- 漏えい経路の API エンドポイントが修正版 / 隔離済み（コード fix の場合は PR マージ済み + デプロイ完了）。
- 影響テナントのセッションが全て無効化済み（再認証完了率を Keycloak audit log で確認）。
- 証跡が MinIO `k1s0-forensics/incident-<id>/` に Legal Hold 設定で保全済み。
- 影響ユーザー数・件数が確定し暗号化済スプレッドシートに記録済み。

## 7. 予防策

- ポストモーテム作成（24h 以内、`postmortems/<YYYY-MM-DD>-RB-SEC-005.md`）。
- 個人情報保護委員会への速報（漏えい確定後 **72 時間以内**）→ [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) で実施。
- 個人情報保護委員会への確報（漏えい確定後 **30 日以内**）。
- 影響を受けた本人への通知（速やかに、遅くとも確報提出前）。
- PII スキャン・検知ルールの強化 PR（`infra/security/falco/rules/`）。
- アプリ コード レビューに「PII マスク」観点を追加（PR テンプレ更新）。
- 月次 Chaos Drill 対象に「擬似 PII 含有レスポンス」シナリオを追加。

## 8. 関連 Runbook

- 関連設計書: [`docs/03_要件定義/30_非機能要件/E_セキュリティ.md`](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md) (NFR-E-SIR-002, NFR-E-ENC-003)、[`G_データ保護とプライバシー.md`](../../../docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md) (NFR-G-CLS-002)
- 関連 ADR: [ADR-SEC-001 (Keycloak)](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md), [ADR-SEC-002 (OpenBao)](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md), [ADR-SEC-003 (SPIRE)](../../../docs/02_構想設計/adr/ADR-SEC-003-spire.md)
- 連鎖 Runbook:
  - [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) — 規制報告（72h 以内必須）
  - [`RB-SEC-006-tenant-boundary-breach.md`](RB-SEC-006-tenant-boundary-breach.md) — テナント越境が原因の場合
  - [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) — 内部 actor / Secret 大量読取が原因の場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
