class Baggage {
  final _entries = <String, String>{};

  void set(String key, String value) => _entries[key] = value;

  String? get(String key) => _entries[key];

  Map<String, String> get entries => Map.unmodifiable(_entries);

  String toHeader() =>
      _entries.entries.map((e) => '${e.key}=${e.value}').join(',');

  static Baggage fromHeader(String s) {
    final baggage = Baggage();
    if (s.isEmpty) return baggage;
    for (final pair in s.split(',')) {
      final idx = pair.indexOf('=');
      if (idx > 0) {
        baggage.set(pair.substring(0, idx).trim(), pair.substring(idx + 1).trim());
      }
    }
    return baggage;
  }
}
