class Filter {
  final String field;
  final String operator;
  final dynamic value;
  final dynamic valueTo;

  const Filter({
    required this.field,
    required this.operator,
    required this.value,
    this.valueTo,
  });

  factory Filter.eq(String field, dynamic value) =>
      Filter(field: field, operator: 'eq', value: value);

  factory Filter.lt(String field, dynamic value) =>
      Filter(field: field, operator: 'lt', value: value);

  factory Filter.gt(String field, dynamic value) =>
      Filter(field: field, operator: 'gt', value: value);

  factory Filter.range(String field, dynamic from, dynamic to) =>
      Filter(field: field, operator: 'range', value: from, valueTo: to);
}

class FacetBucket {
  final String value;
  final int count;

  const FacetBucket({required this.value, required this.count});
}

class SearchQuery {
  final String query;
  final List<Filter> filters;
  final List<String> facets;
  final int page;
  final int size;

  const SearchQuery({
    required this.query,
    this.filters = const [],
    this.facets = const [],
    this.page = 0,
    this.size = 20,
  });
}

class SearchResult<T> {
  final List<T> hits;
  final int total;
  final Map<String, List<FacetBucket>> facets;
  final int tookMs;

  const SearchResult({
    required this.hits,
    required this.total,
    required this.facets,
    required this.tookMs,
  });
}
