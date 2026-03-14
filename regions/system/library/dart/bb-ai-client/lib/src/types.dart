// AI クライアントエラー: API 通信エラーを表す例外クラス
class AiClientError implements Exception {
  final String message;
  final int? statusCode;

  const AiClientError(this.message, {this.statusCode});

  @override
  String toString() =>
      statusCode != null ? 'AiClientError($statusCode): $message' : 'AiClientError: $message';
}

// AI との会話メッセージ: role は "user" / "assistant" / "system" のいずれか
class ChatMessage {
  final String role;
  final String content;

  const ChatMessage({required this.role, required this.content});

  Map<String, dynamic> toJson() => {'role': role, 'content': content};

  factory ChatMessage.fromJson(Map<String, dynamic> json) =>
      ChatMessage(role: json['role'] as String, content: json['content'] as String);
}

// テキスト補完リクエストのパラメータ
class CompleteRequest {
  final String model;
  final List<ChatMessage> messages;
  final int? maxTokens;
  final double? temperature;
  final bool? stream;

  const CompleteRequest({
    required this.model,
    required this.messages,
    this.maxTokens,
    this.temperature,
    this.stream,
  });

  Map<String, dynamic> toJson() => {
        'model': model,
        'messages': messages.map((m) => m.toJson()).toList(),
        if (maxTokens != null) 'max_tokens': maxTokens,
        if (temperature != null) 'temperature': temperature,
        if (stream != null) 'stream': stream,
      };
}

// トークン使用量
class Usage {
  final int inputTokens;
  final int outputTokens;

  const Usage({required this.inputTokens, required this.outputTokens});

  factory Usage.fromJson(Map<String, dynamic> json) => Usage(
        inputTokens: json['input_tokens'] as int,
        outputTokens: json['output_tokens'] as int,
      );
}

// テキスト補完レスポンス
class CompleteResponse {
  final String id;
  final String model;
  final String content;
  final Usage usage;

  const CompleteResponse({
    required this.id,
    required this.model,
    required this.content,
    required this.usage,
  });

  factory CompleteResponse.fromJson(Map<String, dynamic> json) => CompleteResponse(
        id: json['id'] as String,
        model: json['model'] as String,
        content: json['content'] as String,
        usage: Usage.fromJson(json['usage'] as Map<String, dynamic>),
      );
}

// テキスト埋め込みリクエストのパラメータ
class EmbedRequest {
  final String model;
  final List<String> texts;

  const EmbedRequest({required this.model, required this.texts});

  Map<String, dynamic> toJson() => {'model': model, 'texts': texts};
}

// テキスト埋め込みレスポンス
class EmbedResponse {
  final String model;
  final List<List<double>> embeddings;

  const EmbedResponse({required this.model, required this.embeddings});

  factory EmbedResponse.fromJson(Map<String, dynamic> json) => EmbedResponse(
        model: json['model'] as String,
        embeddings: (json['embeddings'] as List)
            .map((e) => (e as List).cast<double>())
            .toList(),
      );
}

// モデルの基本情報
class ModelInfo {
  final String id;
  final String name;
  final String description;

  const ModelInfo({required this.id, required this.name, required this.description});

  factory ModelInfo.fromJson(Map<String, dynamic> json) => ModelInfo(
        id: json['id'] as String,
        name: json['name'] as String,
        description: json['description'] as String,
      );
}
