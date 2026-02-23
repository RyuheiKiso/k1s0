class VersionConflictError implements Exception {
  final int expected;
  final int actual;

  const VersionConflictError(this.expected, this.actual);

  @override
  String toString() =>
      'VersionConflictError: expected=$expected, actual=$actual';
}
