import 'dart:convert';

class CursorRequest {
  final String? cursor;
  final int limit;

  const CursorRequest({this.cursor, required this.limit});
}

class CursorMeta {
  final String? nextCursor;
  final bool hasMore;

  const CursorMeta({this.nextCursor, required this.hasMore});
}

const String _cursorSeparator = '|';

String encodeCursor(String sortKey, String id) =>
    base64Url.encode(utf8.encode('$sortKey$_cursorSeparator$id')).replaceAll('=', '');

String _padBase64(String s) {
  final mod = s.length % 4;
  if (mod == 0) return s;
  return s + '=' * (4 - mod);
}

({String sortKey, String id}) decodeCursor(String cursor) {
  List<int> bytes;
  try {
    bytes = base64Url.decode(_padBase64(cursor));
  } catch (_) {
    bytes = base64.decode(_padBase64(cursor));
  }

  final decoded = utf8.decode(bytes);
  final idx = decoded.indexOf(_cursorSeparator);
  if (idx < 0) {
    throw FormatException('invalid cursor: missing separator');
  }
  return (sortKey: decoded.substring(0, idx), id: decoded.substring(idx + 1));
}
