enum SearchErrorCode {
  indexNotFound,
  invalidQuery,
  serverError,
  timeout,
}

class SearchError implements Exception {
  final String message;
  final SearchErrorCode code;

  const SearchError(this.message, this.code);

  @override
  String toString() => 'SearchError($code): $message';
}
