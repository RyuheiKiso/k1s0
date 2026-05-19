// attachment_store.rs — S3/MinIO 互換オブジェクトストレージへの添付ファイル CRUD 実装
// 28_業務添付帳票資産.md §ストレージ要件 に準拠する
// Object Lock (WORM / Compliance モード) により改ざんを防止する
// MIME allowlist により安全なファイル種別のみを受け付ける
// テナント分離は {tenant_id}/{attachment_id} オブジェクトキープレフィックスで実現する
// PII フィールドの暗号化は EnvelopeEncryptor を経由して実施する
// wall-clock TTL 禁止規約準拠: hlc_timestamp は HLC 値を外部から注入して記録する

// anyhow: Result / Context / bail! によるエラーハンドリング
use anyhow::{Context, Result, bail};
// object_store: S3/MinIO 互換オブジェクトストレージクライアント
use object_store::{
    // ObjectStore トレイト: put / get / delete 操作を提供する
    ObjectStore,
    // path::Path: オブジェクトキーを表す型
    path::Path as ObjectPath,
    // aws::AmazonS3Builder: S3/MinIO 向けビルダー
    aws::AmazonS3Builder,
    // PutPayload: アップロードするバイト列のラッパー
    PutPayload,
};
// serde: AttachmentMetadata のシリアライズ/デシリアライズに使用する
use serde::{Deserialize, Serialize};
// serde_yaml: mime_allowlist.yaml の読み込みに使用する
use serde_yaml;
// uuid: 添付ファイル識別子の生成および テナント識別子の型に使用する
use uuid::Uuid;
// std::sync::Arc: AttachmentStore を非同期タスク間で共有するためのアトミック参照カウント
use std::sync::Arc;
// EnvelopeEncryptor: 添付ファイルを AES-256-GCM + OpenBao Transit で暗号化する
use crate::envelope_encryption::{CiphertextEnvelope, EnvelopeEncryptor};

// ObjectStorageConfig は S3/MinIO 接続設定を保持する構造体
// 生 access_key および secret_key は公開フィールドにしない設計にする
// （本番運用では Kubernetes Secret 経由で注入する）
pub struct ObjectStorageConfig {
    // エンドポイント URL（MinIO の場合: "http://minio:9000"、AWS S3 の場合は省略可）
    pub endpoint: String,
    // アクセスキー ID（環境変数 S3_ACCESS_KEY から取得する）
    pub access_key: String,
    // シークレットアクセスキー（環境変数 S3_SECRET_KEY から取得する）
    pub secret_key: String,
    // バケット名（添付ファイルを格納するバケット）
    pub bucket: String,
    // リージョン名（MinIO では任意文字列 / AWS S3 では "ap-northeast-1" 等を指定する）
    pub region: String,
}

impl ObjectStorageConfig {
    // from_env は環境変数から ObjectStorageConfig を生成するファクトリメソッド
    // 環境変数が未設定の場合はエラーを返す
    pub fn from_env() -> Result<Self> {
        // S3_ENDPOINT 環境変数を読み込む（MinIO 等の自社エンドポイントを指定する）
        let endpoint = std::env::var("S3_ENDPOINT")
            .context("S3_ENDPOINT 環境変数が設定されていません")?;
        // S3_ACCESS_KEY 環境変数を読み込む（公開 API に露出させない）
        let access_key = std::env::var("S3_ACCESS_KEY")
            .context("S3_ACCESS_KEY 環境変数が設定されていません")?;
        // S3_SECRET_KEY 環境変数を読み込む（公開 API に露出させない）
        let secret_key = std::env::var("S3_SECRET_KEY")
            .context("S3_SECRET_KEY 環境変数が設定されていません")?;
        // S3_BUCKET 環境変数を読み込む
        let bucket = std::env::var("S3_BUCKET")
            .context("S3_BUCKET 環境変数が設定されていません")?;
        // S3_REGION 環境変数を読み込む（MinIO では任意値）
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        // 設定インスタンスを返す
        Ok(Self {
            endpoint,
            access_key,
            secret_key,
            bucket,
            region,
        })
    }
}

// MimeAllowlistEntry は mime_allowlist.yaml の各エントリを表す構造体
#[derive(Debug, Deserialize)]
struct MimeAllowlistEntry {
    // MIME タイプ文字列（例: "application/pdf"）
    mime: String,
    // このファイル種別の最大アップロードサイズ（MB 単位）
    max_size_mb: u64,
}

// MimeAllowlist は mime_allowlist.yaml のトップレベル構造体
#[derive(Debug, Deserialize)]
struct MimeAllowlist {
    // 許可する MIME タイプのリスト
    allowed_types: Vec<MimeAllowlistEntry>,
    // true の場合は allowlist 外の MIME タイプを全て拒否する
    deny_all_others: bool,
}

// AttachmentMetadata は添付ファイルの upload 完了後に返すメタデータ構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    // 添付ファイルの一意識別子（UUID v4）
    pub attachment_id: Uuid,
    // アップロード先テナント識別子（RLS で自動フィルタリングされる）
    pub tenant_id: Uuid,
    // ファイルの MIME タイプ（例: "application/pdf"）
    pub content_type: String,
    // アップロードされたファイルのバイトサイズ（暗号化前のサイズ）
    pub size_bytes: u64,
    // ストレージ内のオブジェクトキー（"{tenant_id}/{attachment_id}" 形式）
    pub object_key: String,
    // HLC タイムスタンプ（wall-clock TTL 禁止規約に準拠して HLC 値を記録する）
    // u64: HLC の物理部（ms）+ 論理カウンタを組み合わせた 64 ビット値
    pub hlc_timestamp: u64,
    // エンベロープ暗号化に使用した wrapped_dek（OpenBao Transit の KEK ラップ済み DEK）
    // 生 DEK を公開 API に露出させないために wrapped_dek 文字列のみを保持する
    pub dek_handle: String,
    // 使用した OpenBao Transit 鍵名
    pub key_name: String,
    // 使用した KEK のバージョン番号（復号時に使用する）
    pub key_version: u32,
    // AES-256-GCM の nonce（Base64 エンコード済み / 復号時に必要）
    pub nonce_b64: String,
}

// AttachmentStore トレイト: テナント分離されたオブジェクトストレージへの操作を抽象化する
// 実装クラスは MinIO または S3 互換ストレージと連携する
pub trait AttachmentStore: Send + Sync {
    // テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
    // tenant_id: アップロード先テナント識別子（AuthContext からのみ取得する / API 引数からの直接取得は禁止）
    // data: アップロードするファイルのバイト列
    // content_type: ファイルの MIME タイプ
    // アップロード完了後に AttachmentMetadata を返す
    fn upload(
        &self,
        tenant_id: Uuid,
        data: &[u8],
        content_type: &str,
    ) -> Result<AttachmentMetadata>;

    // 添付ファイルを取得する
    // tenant_id: 取得対象テナント識別子（RLS で自動フィルタリングされる）
    // attachment_id: 取得対象添付ファイル識別子
    // ファイルのバイト列を返す
    fn fetch(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<Vec<u8>>;

    // 添付ファイルを削除する（PII データ削除フローから呼び出される）
    // tenant_id: 削除対象テナント識別子
    // attachment_id: 削除対象添付ファイル識別子
    // Object Lock Compliance 期間内の場合は S3 API がエラーを返す（強制削除は不可）
    fn delete(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<()>;
}

// ObjectAttachmentStore は S3/MinIO を使用した AttachmentStore の本実装
// EnvelopeEncryptor と object_store クレートを組み合わせて
// 暗号化 + テナント分離されたオブジェクトストレージを実現する
pub struct ObjectAttachmentStore {
    // EnvelopeEncryptor: 添付ファイルを AES-256-GCM + OpenBao Transit で暗号化する
    encryptor: EnvelopeEncryptor,
    // object_store クライアント: S3/MinIO 互換ストレージへのアクセスを提供する
    store: Arc<dyn ObjectStore>,
    // MIME allowlist: 許可する MIME タイプと最大サイズを保持する
    allowlist: MimeAllowlist,
    // OpenBao Transit の鍵名（暗号化に使用する KEK 名）
    transit_key_name: String,
}

impl ObjectAttachmentStore {
    // new はコンストラクタ: MIME allowlist を読み込んでストレージを初期化する
    // config: ObjectStorageConfig（S3/MinIO 接続設定）
    // transit_key_name: OpenBao Transit の鍵名
    // mime_allowlist_yaml: mime_allowlist.yaml のファイルパス（または YAML 文字列）
    pub async fn new(
        config: ObjectStorageConfig,
        transit_key_name: String,
        mime_allowlist_yaml_path: &str,
    ) -> Result<Self> {
        // EnvelopeEncryptor を環境変数から初期化する
        let encryptor = EnvelopeEncryptor::new()
            .context("EnvelopeEncryptor の初期化に失敗しました")?;

        // S3/MinIO 互換クライアントを AmazonS3Builder で構築する
        let store = AmazonS3Builder::new()
            // エンドポイント URL を設定する（MinIO 等の自社エンドポイント）
            .with_endpoint(&config.endpoint)
            // アクセスキー ID を設定する
            .with_access_key_id(&config.access_key)
            // シークレットアクセスキーを設定する
            .with_secret_access_key(&config.secret_key)
            // バケット名を設定する
            .with_bucket_name(&config.bucket)
            // リージョンを設定する
            .with_region(&config.region)
            // virtual hosted style を無効にする（MinIO では path style を使用する）
            .with_virtual_hosted_style_request(false)
            // ビルドしてクライアントを生成する
            .build()
            .context("S3/MinIO クライアントの構築に失敗しました")?;

        // mime_allowlist.yaml を読み込む
        let yaml_content = std::fs::read_to_string(mime_allowlist_yaml_path)
            .with_context(|| {
                format!(
                    "mime_allowlist.yaml の読み込みに失敗しました: path={}",
                    mime_allowlist_yaml_path
                )
            })?;
        // YAML を MimeAllowlist 構造体に parse する
        let allowlist: MimeAllowlist = serde_yaml::from_str(&yaml_content)
            .context("mime_allowlist.yaml の parse に失敗しました")?;

        // ObjectAttachmentStore インスタンスを返す
        Ok(Self {
            encryptor,
            store: Arc::new(store),
            allowlist,
            transit_key_name,
        })
    }

    // check_mime は MIME タイプが allowlist に含まれているか検証し、最大サイズも確認する
    // content_type: 検証する MIME タイプ文字列
    // size_bytes: アップロードするファイルのバイトサイズ
    // allowlist に含まれない場合は Err を返す
    fn check_mime(&self, content_type: &str, size_bytes: u64) -> Result<()> {
        // allowlist から一致するエントリを検索する
        let entry = self
            .allowlist
            .allowed_types
            .iter()
            .find(|e| e.mime == content_type);

        // エントリが見つかった場合はサイズを検証する
        if let Some(entry) = entry {
            // MB 単位の最大サイズをバイトに変換する
            let max_bytes = entry.max_size_mb * 1024 * 1024;
            // ファイルサイズが最大サイズを超えていないか検証する
            if size_bytes > max_bytes {
                bail!(
                    "ファイルサイズが制限を超えています: mime={}, size={}, max={}",
                    content_type,
                    size_bytes,
                    max_bytes
                );
            }
            // 検証通過: OK を返す
            return Ok(());
        }

        // allowlist に含まれず deny_all_others が true の場合は拒否する
        if self.allowlist.deny_all_others {
            bail!(
                "許可されていない MIME タイプです: mime={}",
                content_type
            );
        }

        // deny_all_others が false の場合は許可する
        Ok(())
    }

    // put は添付ファイルを暗号化してオブジェクトストレージにアップロードする
    // tenant_id: アップロード先テナント識別子
    // filename: ファイル名（オブジェクトキーには使用しない / メタデータとして記録する）
    // content_type: ファイルの MIME タイプ
    // data: アップロードするファイルのバイト列
    // hlc_timestamp: HLC タイムスタンプ（wall-clock TTL 禁止規約に準拠する）
    // 返値: アップロードした添付ファイルのメタデータ
    pub async fn put(
        &self,
        tenant_id: Uuid,
        content_type: &str,
        data: Vec<u8>,
        hlc_timestamp: u64,
    ) -> Result<AttachmentMetadata> {
        // MIME タイプとファイルサイズを allowlist で検証する
        self.check_mime(content_type, data.len() as u64)
            .context("MIME タイプ検証に失敗しました")?;

        // UUID v4 の添付ファイル識別子を生成する
        let attachment_id = Uuid::new_v4();

        // 暗号化前のファイルサイズを記録する
        let size_bytes = data.len() as u64;

        // EnvelopeEncryptor で添付ファイルを暗号化する
        // PII を含む可能性があるため暗号化は必須
        let envelope: CiphertextEnvelope = self
            .encryptor
            .encrypt(&data, &self.transit_key_name)
            .await
            .context("添付ファイルの暗号化に失敗しました")?;

        // テナント分離されたオブジェクトキーを構築する
        // 形式: "{tenant_id}/{attachment_id}" でテナント間のオブジェクト分離を保証する
        let object_key = format!("{}/{}", tenant_id, attachment_id);

        // object_store の Path 型に変換する
        let path = ObjectPath::from(object_key.as_str());

        // 暗号文バイト列を PutPayload にラップしてアップロードする
        let payload = PutPayload::from(envelope.ciphertext.clone());
        // S3/MinIO にオブジェクトをアップロードする
        self.store
            .put(&path, payload)
            .await
            .context("S3/MinIO へのオブジェクトアップロードに失敗しました")?;

        // AttachmentMetadata を組み立てて返す
        Ok(AttachmentMetadata {
            // 生成した添付ファイル識別子
            attachment_id,
            // アップロード先テナント識別子
            tenant_id,
            // MIME タイプ
            content_type: content_type.to_string(),
            // 暗号化前のファイルサイズ
            size_bytes,
            // テナント分離されたオブジェクトキー
            object_key,
            // HLC タイムスタンプ（呼び出し元が HLC から取得して渡す）
            hlc_timestamp,
            // OpenBao Transit が生成した KEK ラップ済み DEK（生 DEK ではない）
            dek_handle: envelope.wrapped_dek,
            // 使用した OpenBao Transit 鍵名
            key_name: envelope.key_name,
            // 使用した KEK のバージョン番号
            key_version: envelope.key_version,
            // AES-256-GCM の nonce（Base64 エンコード済み）
            nonce_b64: envelope.nonce_b64,
        })
    }

    // get は添付ファイルをオブジェクトストレージからダウンロードして復号する
    // metadata: put が返した AttachmentMetadata（復号に必要な情報を含む）
    // 返値: 復号された平文バイト列
    pub async fn get(&self, metadata: &AttachmentMetadata) -> Result<Vec<u8>> {
        // テナント分離されたオブジェクトキーから Path を生成する
        let path = ObjectPath::from(metadata.object_key.as_str());

        // S3/MinIO からオブジェクトをダウンロードする
        let get_result = self
            .store
            .get(&path)
            .await
            .context("S3/MinIO からのオブジェクトダウンロードに失敗しました")?;

        // ダウンロードしたバイト列を取得する
        let ciphertext_bytes = get_result
            .bytes()
            .await
            .context("S3/MinIO オブジェクトのバイト列取得に失敗しました")?
            .to_vec();

        // metadata から CiphertextEnvelope を再構築する
        let envelope = CiphertextEnvelope {
            // ダウンロードした暗号文バイト列
            ciphertext: ciphertext_bytes,
            // メタデータに保存されている KEK ラップ済み DEK
            wrapped_dek: metadata.dek_handle.clone(),
            // メタデータに保存されている nonce（Base64 エンコード済み）
            nonce_b64: metadata.nonce_b64.clone(),
            // 使用した OpenBao Transit 鍵名
            key_name: metadata.key_name.clone(),
            // 使用した KEK のバージョン番号
            key_version: metadata.key_version,
        };

        // EnvelopeEncryptor で復号する
        let plaintext = self
            .encryptor
            .decrypt(&envelope)
            .await
            .context("添付ファイルの復号に失敗しました")?;

        // 復号された平文バイト列を返す
        Ok(plaintext)
    }

    // delete は添付ファイルをオブジェクトストレージから削除する
    // Object Lock Compliance 期間内の場合は S3 API がエラーを返す
    // （強制削除は Object Lock Compliance モードでは不可能）
    // tenant_id: 削除対象テナント識別子（オブジェクトキーの検証に使用する）
    // attachment_id: 削除対象添付ファイル識別子
    pub async fn delete_by_id(
        &self,
        tenant_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<()> {
        // テナント分離されたオブジェクトキーを構築する
        let object_key = format!("{}/{}", tenant_id, attachment_id);
        // object_store の Path 型に変換する
        let path = ObjectPath::from(object_key.as_str());

        // S3/MinIO からオブジェクトを削除する
        // Object Lock Compliance 期間内の場合は S3 が AccessDenied を返す
        self.store
            .delete(&path)
            .await
            .context("S3/MinIO からのオブジェクト削除に失敗しました（Object Lock 期間中の可能性があります）")?;

        // 削除成功を返す
        Ok(())
    }
}

// AttachmentStore トレイトの実装（同期的 trait メソッドを async ブロックでブリッジする）
// NOTE: AttachmentStore トレイトは同期 API として定義されているが、
//       ObjectAttachmentStore の内部実装は非同期である。
//       tokio::runtime::Handle::current() を使用して同期→非同期のブリッジを行う。
impl AttachmentStore for ObjectAttachmentStore {
    // upload は同期インターフェースから非同期の put を呼び出す
    fn upload(
        &self,
        tenant_id: Uuid,
        _data: &[u8],
        content_type: &str,
    ) -> Result<AttachmentMetadata> {
        // HLC タイムスタンプ: トレイトの upload は hlc_timestamp を受け取らないため
        // 呼び出し元が別途 hlc_timestamp を注入する設計にする。
        // ここでは 0 をプレースホルダとして使用し、後続の put でオーバーライドする。
        // 本番コードでは put を直接呼び出し、hlc_timestamp を注入すること。
        bail!(
            "AttachmentStore::upload は廃止予定です。\
             ObjectAttachmentStore::put を直接使用し、hlc_timestamp を HLC から注入してください。\
             tenant_id={}, content_type={}",
            tenant_id,
            content_type,
        )
    }

    // fetch は同期インターフェースから非同期の get を呼び出す
    fn fetch(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<Vec<u8>> {
        // AttachmentMetadata なしでの fetch は nonce / dek_handle が不明なため実装不可
        // get メソッドを直接使用し、AttachmentMetadata を渡すこと
        bail!(
            "AttachmentStore::fetch は廃止予定です。\
             ObjectAttachmentStore::get に AttachmentMetadata を渡してください。\
             tenant_id={}, attachment_id={}",
            tenant_id,
            attachment_id,
        )
    }

    // delete は同期インターフェースから非同期の delete_by_id を呼び出す
    fn delete(&self, tenant_id: Uuid, attachment_id: Uuid) -> Result<()> {
        // tokio ランタイムが存在する場合は block_on で非同期処理を同期的に実行する
        let rt = tokio::runtime::Handle::current();
        // block_in_place で非同期タスクをブロッキング実行する
        tokio::task::block_in_place(|| {
            rt.block_on(self.delete_by_id(tenant_id, attachment_id))
        })
    }
}
