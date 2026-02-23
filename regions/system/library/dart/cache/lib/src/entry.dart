class CacheEntry<T> {
  final T value;
  final DateTime expiresAt;

  const CacheEntry({
    required this.value,
    required this.expiresAt,
  });

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}
