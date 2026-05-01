# 不正ログイン / Secret 大量読取検知・封じ込め Runbook

> **severity**: SEV2（不正ログイン試行）/ SEV1（認証突破 / Secret 大量読取）
> **owner**: security-sre
> **estimated_mttr**: 4h（SEV2）/ 2h（SEV1 昇格時）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

以下のシグナルが起動トリガーとなる（NFR-E-MON-002, NFR-E-AC-005 関連）。

- Loki アラート `k1s0_auth_brute_force`: 同一 IP から 5 分間に 20 回以上の
  `INVALID_PASSWORD` または `INVALID_OTP` イベント（Keycloak 監査ログ）
- Loki アラート `k1s0_secret_bulk_read`: 単一 SPIFFE ID から 5 分間に 100 件以上の
  OpenBao シークレット読取
- Falco アラート `k1s0_suspicious_token_usage`: 異なる地理的 IP から同一 JWT の使用
- Keycloak リアルタイムイベント: `LOGIN_ERROR` が急増

```bash
# Keycloak 認証失敗ログの確認
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | type="LOGIN_ERROR"
  | line_format "{{.ipAddress}} {{.userId}} {{.error}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -20

# OpenBao の異常な Secret 読取確認
logcli query '{namespace="k1s0-security", job="openbao"}
  |= "secret/read" | json
  | line_format "{{.remote_address}} {{.request_path}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -20
```

## 2. 初動 (Immediate Action)

### パターン A: ブルートフォース / クレデンシャルスタッフィング攻撃

#### Step 1: 攻撃元 IP のブロック（〜15 分）

1. 攻撃元 IP を特定する。

   ```bash
   logcli query '{namespace="k1s0-auth", job="keycloak"}
     | json | type="LOGIN_ERROR"
     | line_format "{{.ipAddress}}"' \
     --since=1h | sort | uniq -c | sort -rn | head -10
   ```

2. Envoy Gateway の IPDenyPolicy でブロックする。

   ```bash
   kubectl apply -f - <<EOF
   apiVersion: gateway.envoyproxy.io/v1alpha1
   kind: SecurityPolicy
   metadata:
     name: block-brute-force-<incident-date>
     namespace: k1s0-ingress
   spec:
     targetRef:
       group: gateway.networking.k8s.io
       kind: HTTPRoute
       name: k1s0-public
     authorization:
       defaultAction: Allow
       rules:
       - action: Deny
         principal:
           clientCIDRs:
           - <attacker-ip>/32
   EOF
   ```

3. Keycloak でアカウントロック閾値を一時的に強化する（Realm Settings → Brute Force）。

#### Step 2: 侵害されたアカウントの特定（〜1 時間）

4. `LOGIN_SUCCESS` に先行する `LOGIN_ERROR` が多い user_id を確認する。

   ```bash
   logcli query '{namespace="k1s0-auth", job="keycloak"}
     | json | type="LOGIN_SUCCESS"
     | line_format "{{.userId}} {{.ipAddress}}"' \
     --since=6h > /tmp/login_success.txt
   # 攻撃元 IP と一致する userId を特定
   grep "<attacker-ip>" /tmp/login_success.txt
   ```

5. 侵害が疑われるアカウントのセッションを無効化し、パスワードリセットを強制する。

   ```bash
   kcadm.sh update users/<user-id>/logout -r k1s0
   kcadm.sh update users/<user-id> -r k1s0 \
     -s requiredActions='["UPDATE_PASSWORD"]'
   ```

### パターン B: Secret 大量読取（内部侵害疑い）

#### Step 3: OpenBao の該当 token を即時 revoke（〜30 分）

6. 異常な読取を行っている SPIFFE ID / AppRole を特定する。

   ```bash
   bao audit list
   # 直近の audit log から bulk-read を確認
   kubectl exec -it openbao-0 -n k1s0-security -- \
     bao read sys/audit-hash/file
   ```

7. 該当 token を revoke する。

   ```bash
   # AppRole / Service Account token の revoke
   bao token revoke -accessor <token-accessor>
   # またはポリシーを即時更新して読取権限を剥奪
   bao policy write <service>-deny infra/security/openbao/policies/<service>-deny.hcl
   ```

8. 影響 Pod を再起動して新 token を取得させる。

   ```bash
   kubectl rollout restart deployment/<service> -n k1s0-tier1
   ```

### Step 4: SEV1 昇格の判断（〜30 分）

9. 以下の条件に 1 つでも該当する場合は SEV1 に昇格し、`../../oncall/escalation.md` を起動する。

   - 管理者アカウント（`realm-admin` ロール）の侵害が疑われる
   - OpenBao の root token または unseal share の漏えいが疑われる
   - 複数テナントの Secret が読取られた
   - PII を含む Secret が読取られた

## 3. 復旧 (Recovery)

1. 攻撃が終息したことを Loki アラートが消灯で確認する。
2. ブロックした IP の NetworkPolicy を定期見直し（1 週間後）で解除可否を判定する。
3. 侵害されたアカウントの MFA 再登録を促す。
4. OpenBao のポリシーを見直し、最小権限の原則（NFR-E-AC-004）に沿って再設計する。

## 4. 原因調査 (Root Cause Analysis)

- 認証ログの全タイムラインを作成し、最初の疑わしいアクセスを特定する。
- パスワードリスト攻撃の場合は `haveibeenpwned` 等での流出確認を検討する。
- OpenBao の audit log から読取られた Secret パスを全件確認する。

  ```bash
  kubectl exec -it openbao-0 -n k1s0-security -- \
    bao read sys/audit-hash/file -format=json \
    | jq '.data | select(.type=="request") | .path' | sort | uniq -c | sort -rn
  ```

## 5. 事後処理 (Post-incident)

- ポストモーテム作成（SEV1: 24h / SEV2: 72h 以内）
- Keycloak のブルートフォース保護設定を恒久的に強化する PR
- MFA の適用範囲拡大を検討する（NFR-E-AC-005）
- OpenBao の Secret 読取アラート閾値を調整する
- 侵害アカウントが管理者の場合: `secret-rotation.md` で全 Secret ローテーション

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-MON-002, NFR-E-AC-004, NFR-E-AC-005)
- 関連 ADR: ADR-SEC-001 (Keycloak), ADR-SEC-002 (OpenBao), ADR-SEC-003 (SPIRE)
- 関連 Runbook: secret-rotation.md, ../../oncall/escalation.md, tenant-boundary-breach.md
