import 'dart:convert';

String encodeCursor(String id) => base64Url.encode(utf8.encode(id));

String decodeCursor(String cursor) => utf8.decode(base64Url.decode(cursor));
