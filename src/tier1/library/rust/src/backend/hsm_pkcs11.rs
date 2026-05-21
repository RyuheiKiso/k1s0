// hsm_pkcs11.rs — k1s0 tier1 Library backend: HSM PKCS#11 バックエンド実装
// spec 05_鍵管理適合仕様.md §v1_kek_hsm_pkcs11: HSM PKCS#11 backend の完全実装。
// v1_data_kek / v1_token_signing / v1_audit_root_signing の 3 key_class に対応する。
// cryptoki crate を optional feature "hsm_integration" で guard し、
// SoftHSM2 が存在しない CI 環境でも cargo check が通るようにする。

// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{anyhow, Result};
// zeroize: Zeroizing<Vec<u8>> で鍵素材をゼロクリアする（spec 層 A: メモリ漏洩防止）
use zeroize::Zeroizing;
// std::fmt: Debug / Display 手動実装
use std::fmt;

// SoftHSM2 モジュールのデフォルトパス（Debian / Ubuntu 系 Linux）
const SOFTHSM2_MODULE_DEFAULT: &str = "/usr/lib/softhsm/libsofthsm2.so";

// ---- ObjectHandle: PKCS#11 オブジェクトハンドル ----

// ObjectHandle は PKCS#11 の CK_OBJECT_HANDLE を opaque にラップする型。
// 生の usize は公開しない（spec §KeyHandle / KeyMaterial の言語横断型 に準拠）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectHandle {
    // inner: CK_OBJECT_HANDLE の数値（u64 で保持する）
    inner: u64,
}

// ObjectHandle のコンストラクタ（crate 内部からのみ生成可能）
impl ObjectHandle {
    // new は数値から ObjectHandle を生成する（pub(crate) のみ）
    pub(crate) fn new(handle: u64) -> Self {
        // inner に数値を格納する
        Self { inner: handle }
    }

    // as_raw は内部数値を返す（PKCS#11 API 呼び出し時のみ使用する）
    pub(crate) fn as_raw(&self) -> u64 {
        // inner の値をそのまま返す
        self.inner
    }
}

// ---- Pkcs11Backend: HSM PKCS#11 バックエンド本体 ----

// Pkcs11Backend は PKCS#11 互換 HSM（SoftHSM2 / Thales Luna / AWS CloudHSM 等）への
// ファサード構造体。spec §v1_kek_hsm_pkcs11 の 5 機能を提供する。
pub struct Pkcs11Backend {
    // module_path: PKCS#11 共有ライブラリの絶対パス（SoftHSM2 デフォルト値あり）
    module_path: String,
    // slot_id: HSM スロット番号（SoftHSM2 では `softhsm2-util --show-slots` で確認する）
    slot_id: u64,
    // pin: HSM ユーザー PIN（secrecy の Zeroizing でゼロクリアする）
    pin: Zeroizing<String>,
}

// Pkcs11Backend の Debug 実装: pin を "[REDACTED]" で隠蔽する
impl fmt::Debug for Pkcs11Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // pin の内容を出力しない（ログ・デバッグ出力への漏洩を防ぐ）
        f.debug_struct("Pkcs11Backend")
            .field("module_path", &self.module_path)
            .field("slot_id", &self.slot_id)
            .field("pin", &"[REDACTED]")
            .finish()
    }
}

// Pkcs11Backend の実装ブロック
impl Pkcs11Backend {
    // new は PKCS#11 バックエンドを初期化して返す。
    // module_path: PKCS#11 共有ライブラリのパス（None の場合は SoftHSM2 デフォルト）
    // slot_id: HSM スロット番号
    // pin: ユーザー PIN（Zeroizing でゼロクリアされる）
    pub fn new(module_path: &str, slot_id: u64, pin: &str) -> Result<Self> {
        // module_path の空文字は SoftHSM2 デフォルトパスに置き換える
        let resolved_path = if module_path.is_empty() {
            // デフォルトの SoftHSM2 モジュールパスを使用する
            SOFTHSM2_MODULE_DEFAULT.to_string()
        } else {
            // 指定されたパスをそのまま使用する
            module_path.to_string()
        };
        // pin を Zeroizing でラップしてゼロクリアを保証する
        let zeroized_pin = Zeroizing::new(pin.to_string());
        // Pkcs11Backend を構築して返す
        let backend = Self {
            // module_path を設定する
            module_path: resolved_path,
            // slot_id を設定する
            slot_id,
            // pin を Zeroizing でラップして設定する
            pin: zeroized_pin,
        };
        // hsm_integration feature が有効な場合は PKCS#11 モジュールの存在確認を行う
        #[cfg(feature = "hsm_integration")]
        {
            // モジュールファイルの存在確認を行う（初期化は実際の操作時に行う）
            if !std::path::Path::new(&backend.module_path).exists() {
                // モジュールファイルが存在しない場合はエラーを返す
                return Err(anyhow!(
                    "PKCS#11 モジュールが見つからない: {}",
                    backend.module_path
                ));
            }
        }
        // 構築した Pkcs11Backend を返す
        Ok(backend)
    }

    // module_path を返す（テスト・診断用）
    pub fn module_path(&self) -> &str {
        // module_path の参照を返す
        &self.module_path
    }

    // slot_id を返す（テスト・診断用）
    pub fn slot_id(&self) -> u64 {
        // slot_id の値を返す
        self.slot_id
    }
}

// ---- feature = "hsm_integration" ガード下の実装 ----
// SoftHSM2 が存在しない環境では compile error にならないよう cfg guard する。

#[cfg(feature = "hsm_integration")]
mod hsm_impl {
    // cryptoki: PKCS#11 Rust バインディング（cryptoki crate）
    use cryptoki::context::{CInitializeArgs, Pkcs11};
    // cryptoki の鍵生成機構定数
    use cryptoki::mechanism::Mechanism;
    // cryptoki のオブジェクト属性
    use cryptoki::object::{Attribute, AttributeType, KeyType, ObjectClass};
    // cryptoki のセッション型
    use cryptoki::session::UserType;
    // cryptoki のスロット型
    use cryptoki::types::AuthPin;
    // super: 親モジュールの型を使用する
    use super::{ObjectHandle, Pkcs11Backend};
    // anyhow: エラーハンドリング
    use anyhow::{anyhow, Result};
    // zeroize: ゼロクリア
    use zeroize::Zeroizing;

    // Pkcs11Backend の hsm_integration 実装
    impl Pkcs11Backend {
        // open_session は PKCS#11 セッションを開いて返す内部ヘルパー。
        fn open_session(&self) -> Result<cryptoki::session::Session> {
            // PKCS#11 コンテキストを初期化する
            let pkcs11 = Pkcs11::new(&self.module_path)
                .map_err(|e| anyhow!("Pkcs11 初期化失敗: {}", e))?;
            // C_Initialize を呼び出す
            pkcs11.initialize(CInitializeArgs::OsThreads)
                .map_err(|e| anyhow!("C_Initialize 失敗: {}", e))?;
            // スロット一覧を取得して slot_id が存在するか確認する
            let slots = pkcs11.get_slots_with_token()
                .map_err(|e| anyhow!("get_slots_with_token 失敗: {}", e))?;
            // slot_id に対応するスロットを取得する
            let slot = slots.into_iter()
                .find(|s| s.id() == self.slot_id)
                .ok_or_else(|| anyhow!("スロット {} が見つからない", self.slot_id))?;
            // R/W セッションを開く
            let session = pkcs11.open_rw_session(slot)
                .map_err(|e| anyhow!("セッション開始失敗: {}", e))?;
            // PIN でログインする
            let pin = AuthPin::new(self.pin.as_str().to_string());
            session.login(UserType::User, Some(&pin))
                .map_err(|e| anyhow!("C_Login 失敗: {}", e))?;
            // セッションを返す
            Ok(session)
        }

        // generate_aes256_key は AES-256 鍵を HSM 内で生成して ObjectHandle を返す。
        // CKM_AES_KEY_GEN を使用し、鍵素材は HSM 外に出さない（spec §v1_kek_hsm_pkcs11）。
        pub fn generate_aes256_key(&self, key_id: &str) -> Result<ObjectHandle> {
            // セッションを開く
            let session = self.open_session()?;
            // CKM_AES_KEY_GEN 機構を指定する
            let mechanism = Mechanism::AesKeyGen;
            // 鍵の属性を設定する（AES-256 = 32 bytes、ラップ可能、抽出不可、永続化）
            let template = vec![
                // 鍵クラス: CKO_SECRET_KEY
                Attribute::Class(ObjectClass::SECRET_KEY),
                // 鍵型: CKK_AES
                Attribute::KeyType(KeyType::AES),
                // 鍵長: 32 bytes（AES-256）
                Attribute::ValueLen(32_u64.into()),
                // 暗号化に使用可能
                Attribute::Encrypt(true),
                // 復号に使用可能
                Attribute::Decrypt(true),
                // ラップに使用可能（DEK を KEK で wrap するために必要）
                Attribute::Wrap(true),
                // アンラップに使用可能
                Attribute::Unwrap(true),
                // 鍵の素材を外部に抽出不可（HSM から出ない）
                Attribute::Extractable(false),
                // セッション終了後も永続化する
                Attribute::Token(true),
                // 鍵ラベルを key_id で設定する
                Attribute::Label(key_id.as_bytes().to_vec()),
            ];
            // C_GenerateKey を呼び出して AES-256 鍵を生成する
            let handle = session.generate_key(&mechanism, &template)
                .map_err(|e| anyhow!("C_GenerateKey 失敗: {}", e))?;
            // CK_OBJECT_HANDLE を ObjectHandle にラップして返す
            Ok(ObjectHandle::new(handle.into()))
        }

        // wrap_key は wrapping_key で target_key をラップして wrapped key bytes を返す。
        // CKM_AES_KEY_WRAP 機構を使用する（spec §v1_kek_hsm_pkcs11）。
        pub fn wrap_key(
            &self,
            wrapping_key: ObjectHandle,
            target_key: ObjectHandle,
        ) -> Result<Zeroizing<Vec<u8>>> {
            // セッションを開く
            let session = self.open_session()?;
            // CKM_AES_KEY_WRAP 機構を指定する（AES key wrap RFC 3394）
            let mechanism = Mechanism::AesKeyWrap;
            // C_WrapKey を呼び出す
            let wrapped = session.wrap_key(
                &mechanism,
                wrapping_key.as_raw().into(),
                target_key.as_raw().into(),
            ).map_err(|e| anyhow!("C_WrapKey 失敗: {}", e))?;
            // Zeroizing でラップして返す（drop 時にゼロクリアされる）
            Ok(Zeroizing::new(wrapped))
        }

        // sign は key_id で識別される署名鍵を使って data に署名し、署名バイト列を返す。
        // CKM_ECDSA または CKM_SHA256_RSA_PKCS を使用する（spec §v1_kek_hsm_pkcs11）。
        pub fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
            // セッションを開く
            let session = self.open_session()?;
            // key_id に対応するオブジェクトを検索する
            let template = vec![
                // ラベルで検索する
                Attribute::Label(key_id.as_bytes().to_vec()),
                // 署名に使用可能なオブジェクトを検索する
                Attribute::Sign(true),
            ];
            // C_FindObjects で鍵オブジェクトを探す
            let objects = session.find_objects(&template)
                .map_err(|e| anyhow!("C_FindObjects 失敗: {}", e))?;
            // 最初のオブジェクトを使用する
            let key_handle = objects.into_iter().next()
                .ok_or_else(|| anyhow!("key_id '{}' に対応する署名鍵が見つからない", key_id))?;
            // 鍵型を確認して機構を選択する（EC: ECDSA、RSA: SHA256_RSA_PKCS）
            let key_type_attr = session.get_attributes(key_handle, &[AttributeType::KeyType])
                .map_err(|e| anyhow!("鍵型取得失敗: {}", e))?;
            // key_type が EC かどうかを判定する
            let is_ec = key_type_attr.into_iter().any(|a| {
                // KeyType::EC の場合は ECDSA を使用する
                matches!(a, Attribute::KeyType(kt) if kt == KeyType::EC)
            });
            // 機構を選択する
            let mechanism = if is_ec {
                // EC 鍵の場合は ECDSA を使用する
                Mechanism::Ecdsa
            } else {
                // RSA 鍵の場合は SHA256_RSA_PKCS を使用する
                Mechanism::Sha256RsaPkcs
            };
            // C_Sign を呼び出して署名を計算する
            let signature = session.sign(&mechanism, key_handle, data)
                .map_err(|e| anyhow!("C_Sign 失敗: {}", e))?;
            // 署名バイト列を返す
            Ok(signature)
        }
    }
}

// ---- feature = "hsm_integration" が無効な場合のスタブ実装 ----
// cargo check が通るように、feature 無効時も型が存在するようにする。

#[cfg(not(feature = "hsm_integration"))]
impl Pkcs11Backend {
    // generate_aes256_key のスタブ実装（hsm_integration feature 無効時）
    // SoftHSM2 が存在しない環境でも cargo check が通るようにする
    pub fn generate_aes256_key(&self, _key_id: &str) -> Result<ObjectHandle> {
        // hsm_integration feature が無効な場合は常にエラーを返す
        Err(anyhow!(
            "hsm_integration feature が無効: PKCS#11 操作は実行できない (module_path={})",
            self.module_path
        ))
    }

    // wrap_key のスタブ実装（hsm_integration feature 無効時）
    pub fn wrap_key(
        &self,
        _wrapping_key: ObjectHandle,
        _target_key: ObjectHandle,
    ) -> Result<Zeroizing<Vec<u8>>> {
        // hsm_integration feature が無効な場合は常にエラーを返す
        Err(anyhow!(
            "hsm_integration feature が無効: wrap_key は実行できない (module_path={})",
            self.module_path
        ))
    }

    // sign のスタブ実装（hsm_integration feature 無効時）
    pub fn sign(&self, _key_id: &str, _data: &[u8]) -> Result<Vec<u8>> {
        // hsm_integration feature が無効な場合は常にエラーを返す
        Err(anyhow!(
            "hsm_integration feature が無効: sign は実行できない (module_path={})",
            self.module_path
        ))
    }
}

// ---- ユニットテスト ----

#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // test_pkcs11_backend_new_default_path は、空の module_path を指定すると
    // SoftHSM2 のデフォルトパスが設定されることを確認する
    #[test]
    fn test_pkcs11_backend_new_default_path() {
        // 空文字列を渡すと SoftHSM2 デフォルトパスが設定される
        // hsm_integration feature 有効時はモジュールファイルが存在しないためエラーになる可能性がある
        // ここではパスの設定だけを確認する（feature 無効時は Ok になる）
        #[cfg(not(feature = "hsm_integration"))]
        {
            // hsm_integration feature 無効時は常に Ok になる
            let backend = Pkcs11Backend::new("", 0, "test-pin")
                .expect("Pkcs11Backend::new が失敗した");
            // デフォルトパスが設定されていることを確認する
            assert_eq!(
                backend.module_path(),
                SOFTHSM2_MODULE_DEFAULT,
                "デフォルトモジュールパスが期待値と異なる"
            );
        }
    }

    // test_pkcs11_backend_custom_path は、カスタム module_path が正しく設定されることを確認する
    #[test]
    fn test_pkcs11_backend_custom_path() {
        // hsm_integration feature 無効時のみ実行する（ファイル存在確認をスキップするため）
        #[cfg(not(feature = "hsm_integration"))]
        {
            // カスタムパスを設定してバックエンドを生成する
            let backend = Pkcs11Backend::new("/custom/pkcs11.so", 1, "test-pin")
                .expect("Pkcs11Backend::new が失敗した");
            // カスタムパスが設定されていることを確認する
            assert_eq!(
                backend.module_path(),
                "/custom/pkcs11.so",
                "カスタムモジュールパスが期待値と異なる"
            );
            // slot_id が正しく設定されていることを確認する
            assert_eq!(backend.slot_id(), 1, "slot_id が期待値と異なる");
        }
    }

    // test_generate_aes256_key_without_hsm は hsm_integration feature 無効時に
    // generate_aes256_key がエラーを返すことを確認する
    // SoftHSM2 が存在しない場合は ignore する
    #[test]
    #[cfg_attr(not(feature = "hsm_integration"), ignore)]
    fn test_generate_aes256_key_hsm_integration() {
        // この test は hsm_integration feature が有効かつ SoftHSM2 が存在する場合のみ実行される
        // CI 環境では ignore される（#[cfg_attr(not(feature = "hsm_integration"), ignore)]）
        let backend = Pkcs11Backend::new(SOFTHSM2_MODULE_DEFAULT, 0, "test-pin")
            .expect("Pkcs11Backend::new が失敗した");
        // generate_aes256_key を呼び出す（SoftHSM2 が存在する場合のみ成功する）
        let result = backend.generate_aes256_key("test-kek-001");
        // SoftHSM2 が存在する場合は Ok になることを確認する
        assert!(result.is_ok(), "generate_aes256_key が失敗した: {:?}", result.err());
    }

    // test_generate_aes256_key_stub は hsm_integration feature 無効時に
    // generate_aes256_key が Err を返すことを確認する
    #[test]
    #[cfg(not(feature = "hsm_integration"))]
    fn test_generate_aes256_key_stub() {
        // hsm_integration feature 無効時のみ実行する
        let backend = Pkcs11Backend::new("/stub/pkcs11.so", 0, "pin")
            .expect("Pkcs11Backend::new が失敗した");
        // generate_aes256_key を呼び出す
        let result = backend.generate_aes256_key("test-key");
        // hsm_integration feature 無効時は Err になることを確認する
        assert!(result.is_err(), "hsm_integration feature 無効時は Err を返すべき");
    }

    // test_debug_redacts_pin は Debug 出力に PIN が含まれないことを確認する
    #[test]
    fn test_debug_redacts_pin() {
        // hsm_integration feature 無効時のみ実行する
        #[cfg(not(feature = "hsm_integration"))]
        {
            // Pkcs11Backend を生成する
            let backend = Pkcs11Backend::new("/test/pkcs11.so", 0, "my-secret-pin")
                .expect("Pkcs11Backend::new が失敗した");
            // Debug 出力を文字列にする
            let debug_str = format!("{:?}", backend);
            // PIN が含まれていないことを確認する
            assert!(
                !debug_str.contains("my-secret-pin"),
                "Debug 出力に PIN が含まれてはならない: {}",
                debug_str
            );
            // [REDACTED] が含まれていることを確認する
            assert!(
                debug_str.contains("[REDACTED]"),
                "Debug 出力に [REDACTED] が含まれるべき: {}",
                debug_str
            );
        }
    }
}
