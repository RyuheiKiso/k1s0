// 真偽値を反転する MAUI IValueConverter（IsBusy / IsNotBusy のバインディング用）。

using System.Globalization;

namespace K1s0.Native.Shared.Converters;

public sealed class InvertedBoolConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is bool b ? !b : value;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is bool b ? !b : value;
    }
}
