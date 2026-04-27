// 本ファイルは iOS プラットフォームのエントリポイント。
// UIApplication.Main で AppDelegate を起動するだけの最小実装。

using ObjCRuntime;
using UIKit;

namespace K1s0.Native.Admin;

public class Program
{
    static void Main(string[] args) => UIApplication.Main(args, null, typeof(AppDelegate));
}
