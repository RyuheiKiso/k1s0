// k1s0 tier2 attachment KEK Transit クライアント抽象インターフェース
// tier1 鍵管理適合仕様（05_鍵管理適合仕様.md）の KeyHandle に対応する C# 抽象層
// 本番実装は OpenBao Transit API を呼び出す OpenBaoKekTransitClient を別途 P11 で提供する

// System.Guid などの基本型
using System;

// k1s0 tier2 attachment 名前空間
namespace K1s0.Tier2.Attachment;

/// <summary>
/// IKekTransitClient: DEK（Data Encryption Key）ハンドルを取得するための抽象インターフェース
/// Rust の KeyHandle::sign() / KeyHandle::derive_dek() に対応する C# 抽象層
/// 本番実装は OpenBao Transit API（transit/sign, transit/datakey）を呼び出す
/// </summary>
public interface IKekTransitClient
{
    /// <summary>
    /// テナント識別子に紐づく DEK ハンドル文字列を取得する
    /// tenantId: DEK を取得するテナント識別子（AuthContext からのみ注入する）
    /// DEK ハンドル文字列（OpenBao Transit の key_id / wrapped_key 形式）を返す
    /// </summary>
    string GetDekHandle(Guid tenantId);
}
