import 'package:dio/dio.dart';

import 'config_types.dart';

class ConfigData {
  const ConfigData({
    required this.schema,
    required this.values,
  });

  final ConfigEditorSchema schema;
  final Map<String, dynamic> values;
}

class ConfigInterpreter {
  const ConfigInterpreter({required this.dio});

  final Dio dio;

  Future<ConfigData> build(String serviceName) async {
    final results = await Future.wait([
      dio.get<Map<String, dynamic>>(
        '/api/v1/config-schema/$serviceName',
      ),
      dio.get<Map<String, dynamic>>(
        '/api/v1/config/services/$serviceName',
      ),
    ]);

    final schemaResponse = results[0].data!;
    final valuesResponse = results[1].data!;

    final schema = ConfigEditorSchema.fromJson(schemaResponse);
    final values = valuesResponse['values'] as Map<String, dynamic>? ?? {};

    return ConfigData(schema: schema, values: values);
  }
}
