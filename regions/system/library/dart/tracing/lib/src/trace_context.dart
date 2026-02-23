class TraceContext {
  final String traceId;
  final String parentId;
  final int flags;

  const TraceContext({
    required this.traceId,
    required this.parentId,
    required this.flags,
  });

  String toTraceparent() =>
      '00-$traceId-$parentId-${flags.toRadixString(16).padLeft(2, '0')}';

  static TraceContext? fromTraceparent(String s) {
    final parts = s.split('-');
    if (parts.length != 4) return null;
    if (parts[0] != '00') return null;
    if (parts[1].length != 32) return null;
    if (parts[2].length != 16) return null;
    if (parts[3].length != 2) return null;

    final flags = int.tryParse(parts[3], radix: 16);
    if (flags == null) return null;

    return TraceContext(
      traceId: parts[1],
      parentId: parts[2],
      flags: flags,
    );
  }
}
