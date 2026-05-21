// k1s0 tier2 attachment KEK Transit クライアント骨格実装
// IKekTransitClient の in-memory 骨格実装（テスト・骨格ビルド用）
// 本番実装は OpenBao Transit API を呼び出す OpenBaoKekTransitClient を P11 で提供する

// System.Guid などの基本型
using System;

// k1s0 tier2 attachment 名前空間
namespace K1s0.Tier2.Attachment;

/// <summary>
/// InMemoryKekTransitClient: IKekTransitClient の in-memory 骨格実装クラス
/// テナント識別子から決定論的な骨格 DEK ハンドル文字列を返す
/// 骨格実装の制約: 実際の OpenBao Transit API 呼び出しは P11 物理化フェーズで実施する
/// </summary>
public sealed class InMemoryKekTransitClient : IKekTransitClient
{
    // 骨格 DEK ハンドルのプレフィックス（OpenBao Transit key_id 形式に準拠した命名）
    private const string SkeletonKeyPrefix = "kek-skeleton-v1";

    /// <summary>
    /// テナント識別子に紐づく骨格 DEK ハンドル文字列を返す
    /// tenantId: DEK を取得するテナント識別子
    /// 骨格 DEK ハンドル文字列を返す（形式: "kek-skeleton-v1/{tenantId}"）
    /// </summary>
    public string GetDekHandle(Guid tenantId)
    {
        // テナント識別子をキーとした骨格 DEK ハンドルを構築して返す
        return $"{SkeletonKeyPrefix}/{tenantId:D}";
    }
}
