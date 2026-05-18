// ISecretStore.cs — k1s0 tier1 Library C# 実装: Secret Management L3 facade interface
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および §SecretStore 抽象 に準拠する。
// Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
// OpenBao / AWS Secrets Manager 等のバックエンドを実装で切り替えられる interface を宣言する。
// 公開 API シグネチャに生 key bytes / 生シークレット値を露出しない。

// System: ArgumentNullException / ObjectDisposedException 等の例外型に使用する
using System;
// System.Threading: CancellationToken（非同期キャンセルに使用する）
using System.Threading;
// System.Threading.Tasks: Task / ValueTask の非同期操作に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// SecretMetadata はシークレットのメタデータを宣言する record。
/// 生のシークレット値は含まない（SecretValue 型で別管理する）。
/// Rust core/secret.rs の SecretMetadata struct と 1:1 対応する。
/// </summary>
// SecretMetadata レコード定義: シークレットのメタデータのみを保持する
public sealed record SecretMetadata(
    // SecretId: シークレットの識別子（UUID v7 形式）
    string SecretId,
    // Version: シークレットのバージョン番号（ローテーション追跡用）
    uint Version,
    // KeyClass: このシークレットを保護している鍵のクラス（05_鍵管理適合仕様 §v1 5 class）
    KeyClass KeyClass,
    // TenantId: シークレットが属するテナントの識別子
    string TenantId,
    // IsActive: シークレットが有効かどうか（revoke / rotate 後 false になる）
    bool IsActive
);

/// <summary>
/// SecretRotationPolicy はシークレットローテーションポリシーを宣言する enum。
/// Rust core/secret.rs の SecretRotationPolicy と 4 言語等価強度を保つ。
/// </summary>
// SecretRotationPolicy 列挙型: ローテーション方式を宣言する
public enum SecretRotationPolicy
{
    /// <summary>Manual: 手動ローテーション（自動ローテーションなし）</summary>
    // Manual: ローテーションは呼び出し元が明示的に rotate を呼ぶ
    Manual,

    /// <summary>OnDemand: 要求時ローテーション（呼び出し元が RotateAsync を呼ぶ）</summary>
    // OnDemand: rotate() 呼び出し時のみローテーションする
    OnDemand,

    /// <summary>Scheduled: スケジュールローテーション（間隔秒数で指定）</summary>
    // Scheduled: インフラが自動的にローテーションをスケジュールする
    Scheduled,
}

/// <summary>
/// ISecretStore は Secret Management の L3 抽象 interface。
/// OpenBao / AWS Secrets Manager 等の OSS / サービスを実装で切り替えられる。
/// Rust core/secret.rs の SecretStore trait と 4 言語等価強度を保つ。
/// 公開 API に生シークレット値を返さない（WithSecretAsync の callback パターンを使う）。
/// </summary>
// ISecretStore インターフェース定義
public interface ISecretStore
{
    /// <summary>
    /// GetMetadataAsync はシークレットのメタデータのみを返す（値は返さない）。
    /// Rust の get_metadata(&self, secret_id, tenant_id) -> Result&lt;Option&lt;SecretMetadata&gt;&gt; に対応する。
    /// </summary>
    // GetMetadataAsync メソッド: シークレットのメタデータのみを返す（値は返さない）
    Task<SecretMetadata?> GetMetadataAsync(
        // secretId: シークレットの識別子（UUID v7 形式）
        string secretId,
        // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
        string tenantId,
        // ct: キャンセルトークン（必ず渡す: I/O ブロック防止のため）
        CancellationToken ct = default);

    /// <summary>
    /// WithSecretAsync はシークレット値を callback に渡して処理させる。
    /// callback 外にシークレット値が漏れない設計（値は返さない）。
    /// Rust の with_secret(&self, secret_id, tenant_id, f: FnOnce(&SecretValue) -> T) に対応する。
    /// </summary>
    // WithSecretAsync メソッド: シークレット値を callback に渡す（値は返さない）
    Task<TResult> WithSecretAsync<TResult>(
        // secretId: シークレットの識別子（UUID v7 形式）
        string secretId,
        // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
        string tenantId,
        // callback: シークレットのバイト列を受け取り TResult を返す処理（バイト列を漏洩させない）
        Func<ReadOnlyMemory<byte>, TResult> callback,
        // ct: キャンセルトークン
        CancellationToken ct = default);

    /// <summary>
    /// RotateAsync はシークレットを新しいバージョンにローテーションする。
    /// 返した SecretMetadata に新しい version が含まれる。
    /// Rust の rotate(&self, secret_id, tenant_id) -> Result&lt;SecretMetadata&gt; に対応する。
    /// </summary>
    // RotateAsync メソッド: シークレットを新しいバージョンにローテーションする
    Task<SecretMetadata> RotateAsync(
        // secretId: シークレットの識別子（UUID v7 形式）
        string secretId,
        // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
        string tenantId,
        // ct: キャンセルトークン
        CancellationToken ct = default);

    /// <summary>
    /// RevokeAsync はシークレットを無効化する（IsActive=false にする）。
    /// revoke 後の WithSecretAsync 呼び出しは例外を投げる。
    /// Rust の revoke(&self, secret_id, tenant_id) -> Result&lt;()&gt; に対応する。
    /// </summary>
    // RevokeAsync メソッド: シークレットを無効化する
    Task RevokeAsync(
        // secretId: シークレットの識別子（UUID v7 形式）
        string secretId,
        // tenantId: テナント識別子（RLS 境界を越えないことを保証する）
        string tenantId,
        // ct: キャンセルトークン
        CancellationToken ct = default);
}
