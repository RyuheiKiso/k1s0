const int minPerPage = 1;
const int maxPerPage = 100;

class PerPageValidationException implements Exception {
  final int value;
  PerPageValidationException(this.value);

  @override
  String toString() =>
      'PerPageValidationException: invalid perPage: $value (must be between $minPerPage and $maxPerPage)';
}

int validatePerPage(int perPage) {
  if (perPage < minPerPage || perPage > maxPerPage) {
    throw PerPageValidationException(perPage);
  }
  return perPage;
}

class PageRequest {
  final int page;
  final int perPage;

  const PageRequest({required this.page, required this.perPage});

  factory PageRequest.defaultRequest() => const PageRequest(page: 1, perPage: 20);

  int get offset => (page - 1) * perPage;

  bool hasNext(int total) => page * perPage < total;
}

class PaginationMeta {
  final int total;
  final int page;
  final int perPage;
  final int totalPages;

  const PaginationMeta({
    required this.total,
    required this.page,
    required this.perPage,
    required this.totalPages,
  });
}

class PageResponse<T> {
  final List<T> items;
  final int total;
  final int page;
  final int perPage;
  final int totalPages;

  const PageResponse({
    required this.items,
    required this.total,
    required this.page,
    required this.perPage,
    required this.totalPages,
  });

  factory PageResponse.create(List<T> items, int total, PageRequest req) {
    final totalPages = req.perPage > 0 ? (total + req.perPage - 1) ~/ req.perPage : 0;
    return PageResponse(
      items: items,
      total: total,
      page: req.page,
      perPage: req.perPage,
      totalPages: totalPages,
    );
  }

  PaginationMeta get meta => PaginationMeta(
        total: total,
        page: page,
        perPage: perPage,
        totalPages: totalPages,
      );
}
