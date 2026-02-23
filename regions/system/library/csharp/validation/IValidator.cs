namespace K1s0.System.Validation;

public interface IValidator
{
    void ValidateEmail(string email);

    void ValidateUuid(string id);

    void ValidateUrl(string url);

    void ValidateTenantId(string tenantId);
}
