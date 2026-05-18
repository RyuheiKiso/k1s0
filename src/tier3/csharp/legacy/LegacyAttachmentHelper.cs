// k1s0 tier3 Legacy .NET Framework 4.6.2+ 添付ファイルヘルパー stub
// MIME タイプ検査 / ファイルサイズ上限チェックを提供する
// tier3/csharp/Attachments.cs の .NET Framework 互換バックポート

// System 名前空間: 基本型 / IO に使用する
using System;
// System.Collections.Generic: HashSet に使用する
using System.Collections.Generic;
// System.IO: File / Stream に使用する
using System.IO;

// k1s0 Legacy tier3 名前空間
namespace K1s0.Tier3.Legacy
{
    // 許可する MIME タイプの定数クラス（FrozenSet 非対応のため HashSet を使用する）
    internal static class LegacyAllowedMimeTypes
    {
        // 許可する MIME タイプを HashSet で管理する（.NET Framework 互換）
        internal static readonly HashSet<string> Value = new HashSet<string>(
            // 大文字小文字を区別しない比較を設定する
            StringComparer.OrdinalIgnoreCase
        )
        {
            // PDF 文書
            "application/pdf",
            // Microsoft Excel（新形式）
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            // Microsoft Word（新形式）
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            // CSV テキスト
            "text/csv",
            // プレーンテキスト
            "text/plain",
            // JPEG 画像
            "image/jpeg",
            // PNG 画像
            "image/png",
        };
    }

    // LegacyAttachmentHelper: .NET Framework 4.6.2+ 互換の添付ファイルヘルパー
    public static class LegacyAttachmentHelper
    {
        // 最大ファイルサイズ（50 MiB）
        private const long MaxFileSizeBytes = 50L * 1024L * 1024L;

        // MIME タイプが許可リストに含まれているか検査する
        public static bool IsAllowedMimeType(string mimeType)
        {
            // null チェックを行う
            if (mimeType == null)
            {
                // null は拒否する
                throw new ArgumentNullException("mimeType");
            }
            // 許可 MIME タイプ一覧に含まれているか確認する
            return LegacyAllowedMimeTypes.Value.Contains(mimeType);
        }

        // ファイルサイズが上限以内かどうかを確認する
        public static bool IsWithinSizeLimit(long sizeBytes)
        {
            // ファイルサイズが最大値以下であることを確認する
            return sizeBytes > 0 && sizeBytes <= MaxFileSizeBytes;
        }

        // ファイルパスからバイトを読み込んでサイズを返す
        public static long GetFileSizeBytes(string filePath)
        {
            // null チェックを行う
            if (filePath == null)
            {
                // null は受け付けない
                throw new ArgumentNullException("filePath");
            }
            // FileInfo を使ってファイルサイズを取得する（.NET Framework 4.6.2+ 互換）
            var info = new FileInfo(filePath);
            // ファイルが存在しない場合は例外を投げる
            if (!info.Exists)
            {
                // ファイルが存在しない場合は FileNotFoundException を投げる
                throw new FileNotFoundException("ファイルが見つかりません", filePath);
            }
            // ファイルサイズを返す
            return info.Length;
        }
    }
}
