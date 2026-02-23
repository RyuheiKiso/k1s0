import 'event.dart';

abstract class AuditClient {
  Future<void> record(AuditEvent event);
  Future<List<AuditEvent>> flush();
}

class BufferedAuditClient implements AuditClient {
  final List<AuditEvent> _buffer = [];

  @override
  Future<void> record(AuditEvent event) async => _buffer.add(event);

  @override
  Future<List<AuditEvent>> flush() async {
    final result = List<AuditEvent>.of(_buffer);
    _buffer.clear();
    return result;
  }
}
