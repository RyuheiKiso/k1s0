// attachment_integration_test.rs — エンベロープ暗号化および AttachmentStore 統合テスト
// 28_業務添付帳票資産.md §暗号化要件 および §ストレージ要件 の検証を行う
// OpenBao / MinIO が存在しない CI 環境では #[ignore] により自動スキップする
// wall-clock TTL 禁止規約準拠: HLC タイムスタンプを使用する（0 は統合テスト専用プレースホルダ）

// k1s0_tier2_attachment クレートの公開 API をインポートする
use k1s0_tier2_attachment::{
    // CiphertextEnvelope: 暗号化結果を保持する構造体
    CiphertextEnvelope,
    // EnvelopeEncryptor: AES-256-GCM + OpenBao Transit の暗号化実装
    EnvelopeEncryptor,
    // ObjectAttachmentStore: S3/MinIO 添付ファイルストア実装
    ObjectAttachmentStore,
    // ObjectStorageConfig: S3/MinIO 接続設定
    ObjectStorageConfig,
};

// test_envelope_encrypt_decrypt_roundtrip は AES-256-GCM の暗号化・復号ラウンドトリップを検証する
// OpenBao が存在しない環境では自動スキップする（#[ignore] attribute を使用する）
#[tokio::test]
#[ignore = "OpenBao Transit API が必要（OPENBAO_ADDR / OPENBAO_TOKEN 環境変数を設定して実行する）"]
async fn test_envelope_encrypt_decrypt_roundtrip() {
    // テスト用平文（PII を想定したバイト列）
    let plaintext = b"test-pii-data: user@example.com / phone: 090-0000-0000";

    // EnvelopeEncryptor を環境変数から初期化する
    // OPENBAO_ADDR と OPENBAO_TOKEN が未設定の場合は panic する
    let encryptor = EnvelopeEncryptor::new()
        .expect("EnvelopeEncryptor の初期化に失敗しました（OPENBAO_ADDR / OPENBAO_TOKEN を確認してください）");

    // テスト用 Transit 鍵名を設定する
    let key_name = "test-attachment-key";

    // 暗号化を実行する（DEK は自動生成・自動消去される）
    let envelope: CiphertextEnvelope = encryptor
        .encrypt(plaintext, key_name)
        .await
        .expect("暗号化に失敗しました");

    // 暗号文が平文と異なることを検証する
    assert_ne!(
        envelope.ciphertext,
        plaintext.to_vec(),
        "暗号文は平文と異なるべきです"
    );

    // wrapped_dek が "vault:" プレフィックスで始まることを検証する（OpenBao Transit 形式）
    assert!(
        envelope.wrapped_dek.starts_with("vault:"),
        "wrapped_dek は 'vault:' で始まるべきです: {}",
        envelope.wrapped_dek
    );

    // nonce_b64 が空でないことを検証する
    assert!(
        !envelope.nonce_b64.is_empty(),
        "nonce_b64 が空です"
    );

    // key_name が正しく保持されていることを検証する
    assert_eq!(
        envelope.key_name, key_name,
        "key_name が一致しません"
    );

    // 復号を実行する
    let decrypted = encryptor
        .decrypt(&envelope)
        .await
        .expect("復号に失敗しました");

    // 復号結果が元の平文と一致することを検証する（ラウンドトリップ検証）
    assert_eq!(
        decrypted,
        plaintext.to_vec(),
        "復号結果が元の平文と一致しません"
    );
}

// test_mime_allowlist_validation は MIME allowlist によるファイル種別検証を単体テストする
// この テストは外部サービス不要で実行できる
#[tokio::test]
#[ignore = "S3_ENDPOINT / S3_ACCESS_KEY / S3_SECRET_KEY / S3_BUCKET 環境変数と OPENBAO_ADDR / OPENBAO_TOKEN が必要"]
async fn test_attachment_store_put_get_roundtrip() {
    // ObjectStorageConfig を環境変数から生成する
    let config = ObjectStorageConfig::from_env()
        .expect("ObjectStorageConfig の生成に失敗しました（S3_ 環境変数を確認してください）");

    // mime_allowlist.yaml のパスを設定する
    // 統合テストは src/tier2/attachment/ ディレクトリで実行されることを前提とする
    let mime_allowlist_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/mime_allowlist.yaml"
    );

    // ObjectAttachmentStore を初期化する
    let store = ObjectAttachmentStore::new(
        config,
        "test-attachment-key".to_string(),
        mime_allowlist_path,
    )
    .await
    .expect("ObjectAttachmentStore の初期化に失敗しました");

    // テスト用 tenant_id を生成する
    let tenant_id = uuid::Uuid::new_v4();
    // テスト用平文データ（PDF のヘッダバイト列 + ダミーデータ）
    let test_data = b"test attachment content for roundtrip validation";
    // MIME タイプを設定する（allowlist に含まれる "application/pdf" を使用する）
    let content_type = "application/pdf";
    // HLC タイムスタンプのプレースホルダ（統合テスト専用）
    // 本番コードでは hlc_lib から実際の HLC 値を取得して渡すこと
    let hlc_timestamp: u64 = 0;

    // アップロードを実行する
    let metadata = store
        .put(tenant_id, content_type, test_data.to_vec(), hlc_timestamp)
        .await
        .expect("アップロードに失敗しました");

    // メタデータを検証する
    assert_eq!(
        metadata.tenant_id, tenant_id,
        "tenant_id が一致しません"
    );
    // MIME タイプが正しく記録されていることを検証する
    assert_eq!(
        metadata.content_type, content_type,
        "content_type が一致しません"
    );
    // サイズが正しく記録されていることを検証する
    assert_eq!(
        metadata.size_bytes,
        test_data.len() as u64,
        "size_bytes が一致しません"
    );
    // オブジェクトキーがテナント分離形式であることを検証する
    assert!(
        metadata.object_key.starts_with(&tenant_id.to_string()),
        "object_key がテナント分離形式ではありません: {}",
        metadata.object_key
    );
    // dek_handle が生 DEK ではなく OpenBao Transit 形式であることを検証する
    assert!(
        metadata.dek_handle.starts_with("vault:"),
        "dek_handle は 'vault:' で始まるべきです"
    );

    // ダウンロードと復号を実行する
    let downloaded = store
        .get(&metadata)
        .await
        .expect("ダウンロード・復号に失敗しました");

    // 復号結果が元のデータと一致することを検証する（ラウンドトリップ検証）
    assert_eq!(
        downloaded,
        test_data.to_vec(),
        "復号結果が元のデータと一致しません"
    );

    // クリーンアップ: テスト用オブジェクトを削除する
    // Object Lock Compliance 期間内の場合はエラーになるが、テスト用バケットでは許容する
    let _ = store
        .delete_by_id(tenant_id, metadata.attachment_id)
        .await;
}

// test_mime_allowlist_deny は allowlist 外の MIME タイプを拒否することを検証する
// この テストは外部サービス不要で実行できる（ObjectAttachmentStore の内部ロジックのみテスト）
#[tokio::test]
#[ignore = "S3 接続設定と OPENBAO 設定が必要"]
async fn test_mime_allowlist_deny_invalid_mime() {
    // ObjectStorageConfig を環境変数から生成する
    let config = ObjectStorageConfig::from_env()
        .expect("ObjectStorageConfig の生成に失敗しました");

    // mime_allowlist.yaml のパスを設定する
    let mime_allowlist_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/mime_allowlist.yaml"
    );

    // ObjectAttachmentStore を初期化する
    let store = ObjectAttachmentStore::new(
        config,
        "test-attachment-key".to_string(),
        mime_allowlist_path,
    )
    .await
    .expect("ObjectAttachmentStore の初期化に失敗しました");

    // テスト用 tenant_id を生成する
    let tenant_id = uuid::Uuid::new_v4();
    // allowlist に含まれない MIME タイプ（実行可能ファイル）を指定する
    let forbidden_mime = "application/x-executable";
    // ダミーデータ
    let data = b"MZ\x90\x00".to_vec();
    // HLC タイムスタンプのプレースホルダ
    let hlc_timestamp: u64 = 0;

    // put が Err を返すことを検証する
    let result = store
        .put(tenant_id, forbidden_mime, data, hlc_timestamp)
        .await;
    // allowlist 外の MIME タイプは拒否されるべき
    assert!(
        result.is_err(),
        "allowlist 外の MIME タイプが受け入れられました: mime={}",
        forbidden_mime
    );
}
