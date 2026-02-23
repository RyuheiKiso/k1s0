namespace K1s0.System.SchedulerClient;

public class SchedulerException : Exception
{
    public string Code { get; }

    public SchedulerException(string message, string code)
        : base(message)
    {
        Code = code;
    }
}
