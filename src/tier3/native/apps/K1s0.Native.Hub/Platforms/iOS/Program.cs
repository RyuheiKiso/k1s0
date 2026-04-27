// iOS の UIApplication 起動エントリ。

using ObjCRuntime;
using UIKit;

namespace K1s0.Native.Hub;

public class Program
{
    static void Main(string[] args)
    {
        UIApplication.Main(args, null, typeof(AppDelegate));
    }
}
