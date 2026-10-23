use aes_gcm::aead::{Aead, Generate, Key, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

/// 暗号化済みの値に付けるプレフィックス（これがない値は旧形式の平文として扱う）
const PREFIX: &str = "enc:v1:";
const NONCE_LEN: usize = 12;

/// キーチェーンに鍵を保存するときのサービス名
const KEYCHAIN_SERVICE: &str = "genpasswd_ex";

/// キーチェーンのアカウント名（debug ビルドは本番の鍵と分ける）
fn keychain_account() -> &'static str {
    if cfg!(debug_assertions) { "db-key-dev" } else { "db-key" }
}

pub struct Cipher {
    aead: Aes256Gcm,
}

impl Cipher {
    /// 鍵を読み込む。環境変数 `GENPASSWD_EX_KEY`（base64）があれば優先し、
    /// なければキーチェーンから取得する（未登録なら生成して登録する）
    pub fn load() -> Result<Self, String> {
        let key = match std::env::var("GENPASSWD_EX_KEY") {
            Ok(b64) => B64
                .decode(b64.trim())
                .map_err(|e| format!("invalid GENPASSWD_EX_KEY: {}", e))?,
            Err(_) => keychain::load_or_create(KEYCHAIN_SERVICE, keychain_account(), || {
                Key::<Aes256Gcm>::generate().to_vec()
            })?,
        };
        let key = Key::<Aes256Gcm>::try_from(key.as_slice())
            .map_err(|_| format!("invalid key length: {} bytes (expected 32)", key.len()))?;
        Ok(Self { aead: Aes256Gcm::new(&key) })
    }

    pub fn encrypt(&self, plaintext: &str) -> String {
        let nonce = Nonce::generate();
        let ciphertext = self
            .aead
            .encrypt(&nonce, plaintext.as_bytes())
            .expect("AES-GCM encryption failed");
        let mut buf = nonce.to_vec();
        buf.extend_from_slice(&ciphertext);
        format!("{}{}", PREFIX, B64.encode(buf))
    }

    /// 暗号化済みの値を復号する。プレフィックスがない値（旧形式の平文）はそのまま返す
    pub fn decrypt(&self, stored: &str) -> Result<String, String> {
        let Some(b64) = stored.strip_prefix(PREFIX) else {
            return Ok(stored.to_string());
        };
        let buf = B64.decode(b64).map_err(|e| format!("invalid base64: {}", e))?;
        if buf.len() < NONCE_LEN {
            return Err("ciphertext too short".to_string());
        }
        let (nonce, ciphertext) = buf.split_at(NONCE_LEN);
        let nonce = Nonce::try_from(nonce).expect("nonce length checked");
        let plaintext = self
            .aead
            .decrypt(&nonce, ciphertext)
            .map_err(|_| "decryption failed (wrong key?)".to_string())?;
        String::from_utf8(plaintext).map_err(|e| format!("invalid UTF-8: {}", e))
    }
}

pub fn is_encrypted(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

#[cfg(target_os = "macos")]
mod keychain {
    use security_framework::passwords::{get_generic_password, set_generic_password};
    use security_framework_sys::base::errSecItemNotFound;

    pub fn load_or_create(
        service: &str,
        account: &str,
        generate: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        match get_generic_password(service, account) {
            Ok(key) => Ok(key),
            // 未登録の場合のみ新規作成する（拒否などのエラーで既存の鍵を上書きしないため）
            Err(e) if e.code() == errSecItemNotFound => {
                let key = generate();
                set_generic_password(service, account, &key)
                    .map_err(|e| format!("failed to save key to keychain: {}", e))?;
                eprintln!("Created encryption key in keychain ({} / {}).", service, account);
                Ok(key)
            }
            Err(e) => Err(format!("failed to read key from keychain: {}", e)),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod keychain {
    pub fn load_or_create(
        _service: &str,
        _account: &str,
        _generate: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        Err("keychain is only supported on macOS; set GENPASSWD_EX_KEY instead".to_string())
    }
}
