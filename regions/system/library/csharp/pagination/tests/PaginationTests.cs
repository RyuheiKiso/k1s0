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
        var sortKey = "2024-01-15";
        var id = "item-12345";
        var cursor = CursorPagination.Encode(sortKey, id);
        var (decodedSortKey, decodedId) = CursorPagination.Decode(cursor);

        Assert.Equal(sortKey, decodedSortKey);
        Assert.Equal(id, decodedId);
    }

    [Fact]
    public void CursorPagination_Decode_MissingSeparator_Throws()
    {
        var cursor = Convert.ToBase64String(global::System.Text.Encoding.UTF8.GetBytes("noseparator"));
        Assert.Throws<FormatException>(() => CursorPagination.Decode(cursor));
    }

    [Fact]
    public void PageRequest_RecordEquality()
    {
        var a = new PageRequest(1, 10);
        var b = new PageRequest(1, 10);
        Assert.Equal(a, b);
    }

    [Fact]
    public void CursorRequest_Fields()
    {
        var req = new CursorRequest("abc", 20);
        Assert.Equal("abc", req.Cursor);
        Assert.Equal(20u, req.Limit);
    }

    [Fact]
    public void CursorMeta_Fields()
    {
        var meta = new CursorMeta("next", true);
        Assert.Equal("next", meta.NextCursor);
        Assert.True(meta.HasMore);
    }

    [Fact]
    public void PaginationMeta_Fields()
    {
        var meta = new PaginationMeta(100, 2, 10, 10);
        Assert.Equal(100, meta.Total);
        Assert.Equal(2u, meta.Page);
        Assert.Equal(10u, meta.PerPage);
        Assert.Equal(10u, meta.TotalPages);
    }

    [Fact]
    public void PerPageValidator_ValidValues()
    {
        Assert.Equal(1u, PerPageValidator.Validate(1));
        Assert.Equal(50u, PerPageValidator.Validate(50));
        Assert.Equal(100u, PerPageValidator.Validate(100));
    }

    [Fact]
    public void PerPageValidator_Zero_Throws()
    {
        Assert.Throws<ArgumentOutOfRangeException>(() => PerPageValidator.Validate(0));
    }

    [Fact]
    public void PerPageValidator_OverMax_Throws()
    {
        Assert.Throws<ArgumentOutOfRangeException>(() => PerPageValidator.Validate(101));
    }
}
