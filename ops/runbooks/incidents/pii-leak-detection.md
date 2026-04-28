# 個人情報漏えい検知・初動対応 Runbook

> **severity**: SEV1
> **owner**: security-sre
> **deadline**: 24h（封じ込め完了）/ 72h（個人情報保護委員会速報）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

以下のいずれかが漏えい疑いのトリガーとなる。

- Falco アラート: `k1s0_pii_exfil_detected`（PII パターンを含む異常なデータ転送）
- Loki アラート: PII 含有レスポンスの外部 IP への送信検知
- tier1 PII API での `NFR-G-CLS-002` スキャン結果アラート
- 外部からの漏えい報告（ユーザー、セキュリティリサーチャー、CERT）
- gitleaks による Git リポジトリへの PII コミット検知
- `K1s0Error.InternalError` 急増 + ログに個人情報パターン

```bash
# Falco アラートの確認
kubectl logs -n falco -l app.kubernetes.io/name=falco --since=1h \
  | grep -i "pii\|personal\|exfil" | tail -30
# PII スキャン最新結果
kubectl get job -n k1s0-security -l app=pii-scanner -o wide
```

## 2. 初動 (Immediate Action)

**目標: 24 時間以内に漏えいの封じ込めを完了する。**

### Step 1: 初期トリアージ（〜30 分）

1. 漏えいデータの種別を確認する（氏名 / メール / 電話 / マイナンバー / 要配慮個人情報）。
2. 漏えい経路の仮説を立てる（API 応答 / ログ流出 / 内部 actor / 外部攻撃）。
3. 影響テナントを特定する。

   ```bash
   # 監査ログから対象 tenant_id を特定
   logcli query '{namespace="k1s0-tier1", job="audit"}
     |= "pii" | json | line_format "{{.tenant_id}} {{.user_id}} {{.path}}"' \
     --since=24h | sort | uniq -c | sort -rn | head -20
   ```

4. SEV1 を宣言し、`escalation-contacts.md` に従って CTO・法務・CPO に連絡する。

### Step 2: 通信封じ込め（〜1 時間）

5. 漏えい経路と疑われるエンドポイントを NetworkPolicy でブロックする。

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

6. 該当 Pod の外部通信を即時遮断する。

   ```bash
   kubectl cordon <node>  # 必要に応じてノードを隔離
   kubectl scale deployment/<suspicious-service> --replicas=0 -n k1s0-tier1
   ```

### Step 3: 証跡保全（〜2 時間）

7. 影響期間のログを MinIO Object Lock バケットにコピーして改ざん防止する。

   ```bash
   # MinIO への証跡保全（Object Lock バケット指定）
   mc cp --recursive \
     minio/k1s0-logs/tier1/$(date +%Y-%m-%d) \
     minio/k1s0-forensics/incident-$(date +%Y%m%d-%H%M)/
   # Object Lock 設定確認
   mc legalhold set minio/k1s0-forensics/incident-$(date +%Y%m%d-%H%M)/ ON
   ```

8. Pod のメモリダンプを取得する（可能な場合）。

   ```bash
   kubectl debug -it <pod-name> -n k1s0-tier1 \
     --image=ghcr.io/nicolaka/netshoot -- sh
   ```

### Step 4: 影響範囲の確定（〜8 時間）

9. 影響ユーザー数・テナント数をクエリで特定する。

   ```bash
   # Audit ログから影響 user_id を抽出
   logcli query '{namespace="k1s0-tier1", job="audit"}
     |= "LEAK_SUSPECT" | json | line_format "{{.user_id}}"' \
     --since=48h | sort -u > /tmp/affected_users.txt
   wc -l /tmp/affected_users.txt
   ```

10. 漏えいデータの種別・件数をスプレッドシート（要暗号化）に記録する。

## 3. 復旧 (Recovery)

1. 封じ込め完了後、遮断した NetworkPolicy / Deployment を段階的に復旧する。
2. 影響テナントのアクセストークンを無効化し、再認証を促す。

   ```bash
   # Keycloak で対象ユーザーのセッションを無効化
   kcadm.sh delete sessions/<session-id> -r k1s0
   # または realm 全体のセッション無効化（重大な場合のみ）
   kcadm.sh delete clients/<client-id>/user-sessions -r k1s0
   ```

3. 影響ユーザーへの個別通知を準備する（`pii-regulatory-disclosure.md` 参照）。
4. PII スキャンを再実行して漏えいが終息していることを確認する。

## 4. 原因調査 (Root Cause Analysis)

- Loki + Falco の相関分析で漏えい開始時刻を特定する。
- 漏えい経路を特定する（API バグ / 権限設定ミス / 内部 actor / 外部侵害）。
- 影響 API エンドポイントのコードレビューを実施する。
- Keycloak の認証・認可ログを調査し、不審なアクセスを確認する。

  ```bash
  logcli query '{namespace="k1s0-auth", job="keycloak"}
    | json | line_format "{{.userId}} {{.ipAddress}} {{.type}}"' \
    --since=72h | grep -v "LOGIN_SUCCESS" | head -50
  ```

## 5. 事後処理 (Post-incident)

- ポストモーテム作成（24h 以内）
- 個人情報保護委員会への速報（漏えい確定後 **72 時間以内**）→ `pii-regulatory-disclosure.md`
- 個人情報保護委員会への確報（漏えい確定後 **30 日以内**）
- 影響を受けた本人への通知（速やかに、遅くとも確報提出前）
- PII スキャン・検知ルールの強化 PR

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-SIR-002, NFR-E-ENC-003)
- 関連設計書: docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md (NFR-G-CLS-002)
- 関連 ADR: ADR-SEC-001 (Keycloak), ADR-SEC-002 (OpenBao), ADR-SEC-003 (SPIRE)
- 関連 Runbook: pii-regulatory-disclosure.md, escalation-contacts.md
