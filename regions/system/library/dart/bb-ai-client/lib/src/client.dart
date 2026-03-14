import 'types.dart';

// AI ゲートウェイへのアクセスを抽象化する抽象クラス
abstract class AiClient {
  // チャット補完を実行する
  Future<CompleteResponse> complete(CompleteRequest req);
  // テキスト埋め込みを生成する
  Future<EmbedResponse> embed(EmbedRequest req);
  // 利用可能なモデル一覧を返す
  Future<List<ModelInfo>> listModels();
}
