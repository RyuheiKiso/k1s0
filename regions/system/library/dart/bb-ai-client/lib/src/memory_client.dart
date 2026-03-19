import 'client.dart';
import 'types.dart';

// インメモリ AI クライアント: テスト用のモック実装
class InMemoryAiClient implements AiClient {
  // complete のカスタムハンドラ（未設定時はデフォルトダミーレスポンス）
  final Future<CompleteResponse> Function(CompleteRequest)? _completeFn;
  // embed のカスタムハンドラ
  final Future<EmbedResponse> Function(EmbedRequest)? _embedFn;
  // listModels のカスタムハンドラ
  final Future<List<ModelInfo>> Function()? _listModelsFn;

  InMemoryAiClient({
    Future<CompleteResponse> Function(CompleteRequest)? complete,
    Future<EmbedResponse> Function(EmbedRequest)? embed,
    Future<List<ModelInfo>> Function()? listModels,
  })  : _completeFn = complete,
        _embedFn = embed,
        _listModelsFn = listModels;

  @override
  Future<CompleteResponse> complete(CompleteRequest req) async {
    if (_completeFn != null) return _completeFn(req);
    // デフォルトはダミーレスポンスを返す
    return CompleteResponse(
      id: 'mock-id',
      model: req.model,
      content: 'mock response',
      usage: const Usage(inputTokens: 0, outputTokens: 0),
    );
  }

  @override
  Future<EmbedResponse> embed(EmbedRequest req) async {
    if (_embedFn != null) return _embedFn(req);
    // デフォルトは空の埋め込みを返す
    return EmbedResponse(
      model: req.model,
      embeddings: req.texts.map((_) => <double>[]).toList(),
    );
  }

  @override
  Future<List<ModelInfo>> listModels() async {
    if (_listModelsFn != null) return _listModelsFn();
    // デフォルトは空リストを返す
    return [];
  }
}
