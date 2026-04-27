# ApprovalFlow

tier2 の承認フローサービス（.NET 8 + ASP.NET Core minimal API）。Onion Architecture 4 層構成。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_solution配置.md`
- 内部構造: `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md`

## エンドポイント

| メソッド | パス | 役割 |
|---|---|---|
| `POST` | `/api/approvals` | 新規承認申請 |
| `POST` | `/api/approvals/{id}/decide` | 承認 / 却下 |
| `GET` | `/healthz` | liveness |
| `GET` | `/readyz` | readiness |

### POST /api/approvals

リクエスト:

```json
{
  "requester": "alice",
  "currency": "JPY",
  "minorAmount": 50000,
  "reason": "出張費"
}
```

正常レスポンス（201 Created）:

```json
{
  "id": "...",
  "status": "PENDING",
  "submittedAt": "2026-04-27T12:00:00Z"
}
```

### POST /api/approvals/{id}/decide

リクエスト:

```json
{
  "approver": "bob",
  "decision": "APPROVE"
}
```

`decision` は `APPROVE` または `REJECT`。

## レイアウト

```text
ApprovalFlow/
├── ApprovalFlow.sln                # 単独 .sln（CI path-filter 用）
├── Dockerfile                      # multi-stage（aspnet:8.0-bookworm-slim runtime）
├── catalog-info.yaml
├── README.md
├── src/
│   ├── ApprovalFlow.Api/           # ASP.NET Core minimal API
│   ├── ApprovalFlow.Application/   # UseCases / DTOs（→ Domain）
│   ├── ApprovalFlow.Domain/        # Entity / ValueObject / Event / Interface（依存なし）
│   └── ApprovalFlow.Infrastructure/# InMemoryApprovalRepository（→ Domain）
└── tests/
    ├── ApprovalFlow.Domain.Tests/
    ├── ApprovalFlow.Application.Tests/
    └── ApprovalFlow.ArchitectureTests/   # NetArchTest による層間依存方向強制
```

## 状態遷移

`Pending` から開始し、以下のいずれかへ遷移する。

```text
Pending ─Approve─→ Approved
Pending ─Reject──→ Rejected
Pending ─Cancel──→ Cancelled  (申請者本人のみ)
```

`Approved / Rejected / Cancelled` からの再遷移は禁止（InvalidOperationException）。

## エラーコード

| コード | カテゴリ | 説明 |
|---|---|---|
| `E-T2-APPR-001` | VALIDATION | submit 入力不正（金額 0 / requester 空 等） |
| `E-T2-APPR-002` | VALIDATION | id 形式不正（Guid parse 失敗） |
| `E-T2-APPR-003` | VALIDATION | decide 入力不正 |
| `E-T2-APPR-004` | NOT_FOUND | 集約が存在しない |
| `E-T2-APPR-005` | CONFLICT | 状態遷移違反（既決を再決定 等） |

## ビルド

```bash
# tier2/dotnet ルートから。
dotnet build services/ApprovalFlow/ApprovalFlow.sln -c Release

# 単体テスト。
dotnet test services/ApprovalFlow/ApprovalFlow.sln

# Architecture テストのみ。
dotnet test services/ApprovalFlow/ApprovalFlow.sln --filter Category=Architecture
```

## 永続化の置換ポイント

リリース時点 では `InMemoryApprovalRepository` を `IApprovalRepository` に DI している。リリース時点 で k1s0 State backed / Postgres EF Core 実装に置換する場合、以下の手順:

1. `src/ApprovalFlow.Infrastructure/Persistence/` に新実装（例: `K1s0StateApprovalRepository`）を追加
2. `Program.cs` の `AddSingleton<IApprovalRepository, ...>` を切替
3. Application 層は変更不要（interface 越しのため）
