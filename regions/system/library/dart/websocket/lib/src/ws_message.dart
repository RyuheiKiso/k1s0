import 'dart:typed_data';

enum MessageType { text, binary, ping, pong, close }

class WsMessage {
  final MessageType type;
  final Object payload;

  const WsMessage({required this.type, required this.payload});

  String get textPayload => payload as String;
  Uint8List get binaryPayload => payload as Uint8List;
}
