using K1s0.System.Pagination;

namespace K1s0.System.Pagination.Tests;

public class PaginationTests
{
    [Fact]
    public void PageResponse_Create_CalculatesTotalPages()
    {
        var items = new List<string> { "a", "b" };
        var req = new PageRequest(1, 2);
        var resp = PageResponse<string>.Create(items, 5, req);

        Assert.Equal(5, resp.Total);
        Assert.Equal(1u, resp.Page);
        Assert.Equal(2u, resp.PerPage);
        Assert.Equal(3u, resp.TotalPages);
        Assert.Equal(2, resp.Items.Count);
    }

    [Fact]
    public void PageResponse_Create_ExactDivision()
    {
        var items = new List<int> { 1, 2 };
        var req = new PageRequest(1, 2);
        var resp = PageResponse<int>.Create(items, 4, req);

        Assert.Equal(2u, resp.TotalPages);
    }

    [Fact]
    public void PageResponse_Create_ZeroPerPage_ReturnsZeroTotalPages()
    {
        var items = new List<int>();
        var req = new PageRequest(1, 0);
        var resp = PageResponse<int>.Create(items, 10, req);

        Assert.Equal(0u, resp.TotalPages);
    }

    [Fact]
    public void CursorPagination_EncodeAndDecode_Roundtrip()
    {
        var original = "item-12345";
        var cursor = CursorPagination.Encode(original);
        var decoded = CursorPagination.Decode(cursor);

        Assert.Equal(original, decoded);
    }

    [Fact]
    public void CursorPagination_Encode_ReturnsBase64()
    {
        var cursor = CursorPagination.Encode("test");
        Assert.Equal("dGVzdA==", cursor);
    }

    [Fact]
    public void PageRequest_RecordEquality()
    {
        var a = new PageRequest(1, 10);
        var b = new PageRequest(1, 10);
        Assert.Equal(a, b);
    }
}
