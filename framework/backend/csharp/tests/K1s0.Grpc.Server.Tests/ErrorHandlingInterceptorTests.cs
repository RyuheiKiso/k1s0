using System.Net;
using FluentAssertions;
using Grpc.Core;
using K1s0.Grpc.Server.Interceptors;

namespace K1s0.Grpc.Server.Tests;

public class ErrorHandlingInterceptorTests
{
    [Theory]
    [InlineData(HttpStatusCode.BadRequest, StatusCode.InvalidArgument)]
    [InlineData(HttpStatusCode.Unauthorized, StatusCode.Unauthenticated)]
    [InlineData(HttpStatusCode.Forbidden, StatusCode.PermissionDenied)]
    [InlineData(HttpStatusCode.NotFound, StatusCode.NotFound)]
    [InlineData(HttpStatusCode.Conflict, StatusCode.AlreadyExists)]
    [InlineData(HttpStatusCode.ServiceUnavailable, StatusCode.Unavailable)]
    [InlineData(HttpStatusCode.RequestTimeout, StatusCode.DeadlineExceeded)]
    [InlineData(HttpStatusCode.GatewayTimeout, StatusCode.DeadlineExceeded)]
    [InlineData(HttpStatusCode.InternalServerError, StatusCode.Internal)]
    public void MapToGrpcStatusCode_MapsCorrectly(HttpStatusCode http, StatusCode expected)
    {
        var result = ErrorHandlingInterceptor.MapToGrpcStatusCode(http);
        result.Should().Be(expected);
    }
}
