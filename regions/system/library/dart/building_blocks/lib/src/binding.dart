import 'dart:typed_data';
import 'component.dart';

class BindingData {
  final Uint8List data;
  final Map<String, String> metadata;

  const BindingData({required this.data, required this.metadata});
}

class BindingResponse {
  final Uint8List data;
  final Map<String, String> metadata;

  const BindingResponse({required this.data, required this.metadata});
}

abstract class InputBinding implements Component {
  Future<BindingData> read();
}

abstract class OutputBinding implements Component {
  Future<BindingResponse> invoke(String operation, Uint8List data, {Map<String, String>? metadata});
}
