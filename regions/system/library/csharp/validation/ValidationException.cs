namespace K1s0.System.Validation;

public class ValidationException : Exception
{
    public string Field { get; }
    public string Code { get; }

    public ValidationException(string field, string message, string? code = null)
        : base(message)
    {
        Field = field;
        Code = code ?? $"INVALID_{field.ToUpperInvariant()}";
    }
}

public class ValidationErrors
{
    private readonly List<ValidationException> _errors = [];

    public bool HasErrors() => _errors.Count > 0;

    public IReadOnlyList<ValidationException> GetErrors() => _errors.AsReadOnly();

    public void Add(ValidationException error) => _errors.Add(error);
}
