class PageRequest {
  final int page;
  final int perPage;

  const PageRequest({required this.page, required this.perPage});
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
}
