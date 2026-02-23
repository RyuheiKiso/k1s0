namespace K1s0.System.Validation;

public class ValidationException : Exception
{
    public string Field { get; }

    public ValidationException(string field, string message)
        : base(message)
    {
        Field = field;
    }
}
