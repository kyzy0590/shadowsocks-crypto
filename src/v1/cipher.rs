#[cfg(feature = "v1-aead")]
use super::aeadcipher::AeadCipher;
#[cfg(feature = "v1-stream")]
use super::streamcipher::StreamCipher;
use super::{dummy::DummyCipher, CipherCategory, CipherKind};

/// Get available ciphers in string representation
///
/// Commonly used for checking users' configuration input
pub const fn available_ciphers() -> &'static [&'static str] {
    &[
        "plain",
        "none",
        #[cfg(feature = "v1-stream")]
        "table",
        #[cfg(feature = "v1-stream")]
        "rc4-md5",
        // Stream Ciphers
        #[cfg(feature = "v1-stream")]
        "aes-128-ctr",
        #[cfg(feature = "v1-stream")]
        "aes-192-ctr",
        #[cfg(feature = "v1-stream")]
        "aes-256-ctr",
        #[cfg(feature = "v1-stream")]
        "aes-128-cfb",
        #[cfg(feature = "v1-stream")]
        "aes-128-cfb1",
        #[cfg(feature = "v1-stream")]
        "aes-128-cfb8",
        #[cfg(feature = "v1-stream")]
        "aes-128-cfb128",
        #[cfg(feature = "v1-stream")]
        "aes-192-cfb",
        #[cfg(feature = "v1-stream")]
        "aes-192-cfb1",
        #[cfg(feature = "v1-stream")]
        "aes-192-cfb8",
        #[cfg(feature = "v1-stream")]
        "aes-192-cfb128",
        #[cfg(feature = "v1-stream")]
        "aes-256-cfb",
        #[cfg(feature = "v1-stream")]
        "aes-256-cfb1",
        #[cfg(feature = "v1-stream")]
        "aes-256-cfb8",
        #[cfg(feature = "v1-stream")]
        "aes-256-cfb128",
        #[cfg(feature = "v1-stream")]
        "aes-128-ofb",
        #[cfg(feature = "v1-stream")]
        "aes-192-ofb",
        #[cfg(feature = "v1-stream")]
        "aes-256-ofb",
        #[cfg(feature = "v1-stream")]
        "camellia-128-ctr",
        #[cfg(feature = "v1-stream")]
        "camellia-192-ctr",
        #[cfg(feature = "v1-stream")]
        "camellia-256-ctr",
        #[cfg(feature = "v1-stream")]
        "camellia-128-cfb",
        #[cfg(feature = "v1-stream")]
        "camellia-128-cfb1",
        #[cfg(feature = "v1-stream")]
        "camellia-128-cfb8",
        #[cfg(feature = "v1-stream")]
        "camellia-128-cfb128",
        #[cfg(feature = "v1-stream")]
        "camellia-192-cfb",
        #[cfg(feature = "v1-stream")]
        "camellia-192-cfb1",
        #[cfg(feature = "v1-stream")]
        "camellia-192-cfb8",
        #[cfg(feature = "v1-stream")]
        "camellia-192-cfb128",
        #[cfg(feature = "v1-stream")]
        "camellia-256-cfb",
        #[cfg(feature = "v1-stream")]
        "camellia-256-cfb1",
        #[cfg(feature = "v1-stream")]
        "camellia-256-cfb8",
        #[cfg(feature = "v1-stream")]
        "camellia-256-cfb128",
        #[cfg(feature = "v1-stream")]
        "camellia-128-ofb",
        #[cfg(feature = "v1-stream")]
        "camellia-192-ofb",
        #[cfg(feature = "v1-stream")]
        "camellia-256-ofb",
        #[cfg(feature = "v1-stream")]
        "rc4",
        #[cfg(feature = "v1-stream")]
        "chacha20-ietf",
        // AEAD Ciphers
        #[cfg(feature = "v1-aead")]
        "aes-128-gcm",
        #[cfg(feature = "v1-aead")]
        "aes-256-gcm",
        #[cfg(feature = "v1-aead")]
        "chacha20-ietf-poly1305",
        #[cfg(feature = "v1-aead-extra")]
        "aes-128-ccm",
        #[cfg(feature = "v1-aead-extra")]
        "aes-256-ccm",
        #[cfg(feature = "v1-aead-extra")]
        "aes-128-gcm-siv",
        #[cfg(feature = "v1-aead-extra")]
        "aes-256-gcm-siv",
        #[cfg(feature = "v1-aead-extra")]
        "xchacha20-ietf-poly1305",
        // #[cfg(feature = "v1-aead-extra")]
        // "sm4-gcm",
        // #[cfg(feature = "v1-aead-extra")]
        // "sm4-ccm",
    ]
}

/// Generate random bytes into `iv_or_salt`
pub fn random_iv_or_salt(iv_or_salt: &mut [u8]) {
    // Gen IV or Gen Salt by KEY-LEN
    if iv_or_salt.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    loop {
        rand::Rng::fill(&mut rng, iv_or_salt);
        let is_zeros = iv_or_salt.iter().all(|&x| x == 0);
        if !is_zeros {
            break;
        }
    }
}

/// Key derivation of OpenSSL's [EVP_BytesToKey](https://wiki.openssl.org/index.php/Manual:EVP_BytesToKey(3))
pub fn openssl_bytes_to_key(password: &[u8], key: &mut [u8]) {
    use md5::{
        digest::{
            generic_array::{typenum::Unsigned, GenericArray},
            OutputSizeUser,
        },
        Digest,
        Md5,
    };

    let key_len = key.len();
    let digest_len = <Md5 as OutputSizeUser>::OutputSize::to_usize();

    let mut last_digest: Option<GenericArray<u8, <Md5 as OutputSizeUser>::OutputSize>> = None;

    let mut offset = 0usize;
    while offset < key_len {
        let mut m = Md5::new();
        if let Some(digest) = last_digest {
            m.update(&digest);
        }

        m.update(password);

        let digest = m.finalize();

        let amt = std::cmp::min(key_len - offset, digest_len);
        key[offset..offset + amt].copy_from_slice(&digest[..amt]);

        offset += digest_len;
        last_digest = Some(digest);
    }
}

trait CipherInner {
    fn ss_kind(&self) -> CipherKind;
    fn ss_category(&self) -> CipherCategory;
    fn ss_tag_len(&self) -> usize;
    fn ss_encrypt_slice(&mut self, plaintext_in_ciphertext_out: &mut [u8]);
    fn ss_decrypt_slice(&mut self, ciphertext_in_plaintext_out: &mut [u8]) -> bool;
}

impl CipherInner for DummyCipher {
    fn ss_kind(&self) -> CipherKind {
        CipherKind::NONE
    }

    fn ss_category(&self) -> CipherCategory {
        CipherCategory::None
    }

    fn ss_tag_len(&self) -> usize {
        0
    }

    fn ss_encrypt_slice(&mut self, _plaintext_in_ciphertext_out: &mut [u8]) {}

    fn ss_decrypt_slice(&mut self, _ciphertext_in_plaintext_out: &mut [u8]) -> bool {
        true
    }
}

#[cfg(feature = "v1-stream")]
impl CipherInner for StreamCipher {
    fn ss_kind(&self) -> CipherKind {
        self.kind()
    }

    fn ss_category(&self) -> CipherCategory {
        CipherCategory::Stream
    }

    fn ss_tag_len(&self) -> usize {
        0
    }

    fn ss_encrypt_slice(&mut self, plaintext_in_ciphertext_out: &mut [u8]) {
        self.encrypt(plaintext_in_ciphertext_out)
    }

    fn ss_decrypt_slice(&mut self, ciphertext_in_plaintext_out: &mut [u8]) -> bool {
        self.decrypt(ciphertext_in_plaintext_out);
        true
    }
}

#[cfg(feature = "v1-aead")]
impl CipherInner for AeadCipher {
    fn ss_kind(&self) -> CipherKind {
        self.kind()
    }

    fn ss_category(&self) -> CipherCategory {
        CipherCategory::Aead
    }

    fn ss_tag_len(&self) -> usize {
        self.tag_len()
    }

    fn ss_encrypt_slice(&mut self, plaintext_in_ciphertext_out: &mut [u8]) {
        self.encrypt(plaintext_in_ciphertext_out)
    }

    fn ss_decrypt_slice(&mut self, ciphertext_in_plaintext_out: &mut [u8]) -> bool {
        self.decrypt(ciphertext_in_plaintext_out)
    }
}

/// Unified interface of Ciphers
#[allow(clippy::large_enum_variant)]
pub enum Cipher {
    Dummy(DummyCipher),
    #[cfg(feature = "v1-stream")]
    Stream(StreamCipher),
    #[cfg(feature = "v1-aead")]
    Aead(AeadCipher),
}

macro_rules! cipher_method_forward {
    (ref $self:expr, $method:ident $(, $param:expr),*) => {
        match *$self {
            Cipher::Dummy(ref c) => c.$method($($param),*),
            #[cfg(feature = "v1-stream")]
            Cipher::Stream(ref c) => c.$method($($param),*),
            #[cfg(feature = "v1-aead")]
            Cipher::Aead(ref c) => c.$method($($param),*),
        }
    };

    (mut $self:expr, $method:ident $(, $param:expr),*) => {
        match *$self {
            Cipher::Dummy(ref mut c) => c.$method($($param),*),
            #[cfg(feature = "v1-stream")]
            Cipher::Stream(ref mut c) => c.$method($($param),*),
            #[cfg(feature = "v1-aead")]
            Cipher::Aead(ref mut c) => c.$method($($param),*),
        }
    };
}

impl Cipher {
    /// Create a new Cipher of `kind`
    ///
    /// - Stream Ciphers initialize with IV
    /// - AEAD Ciphers initialize with SALT
    pub fn new(kind: CipherKind, key: &[u8], iv_or_salt: &[u8]) -> Cipher {
        let category = kind.category();

        match category {
            CipherCategory::None => {
                let _ = key;
                let _ = iv_or_salt;

                Cipher::Dummy(DummyCipher::new())
            }
            #[cfg(feature = "v1-stream")]
            CipherCategory::Stream => Cipher::Stream(StreamCipher::new(kind, key, iv_or_salt)),
            #[cfg(feature = "v1-aead")]
            CipherCategory::Aead => {
                use hkdf::Hkdf;
                use sha1::Sha1;

                const MAX_KEY_LEN: usize = 64;
                const SUBKEY_INFO: &'static [u8] = b"ss-subkey";

                let ikm = key;
                let hk = Hkdf::<Sha1>::new(Some(iv_or_salt), ikm);
                let mut okm = [0u8; MAX_KEY_LEN];
                hk.expand(SUBKEY_INFO, &mut okm).expect("HKDF-SHA1");

                let subkey = &okm[..ikm.len()];
                Cipher::Aead(AeadCipher::new(kind, subkey))
            }
        }
    }

    /// Get the `CipherCategory` of the current cipher
    pub fn category(&self) -> CipherCategory {
        cipher_method_forward!(ref self, ss_category)
    }

    /// Get the `CipherKind` of the current cipher
    pub fn kind(&self) -> CipherKind {
        cipher_method_forward!(ref self, ss_kind)
    }

    /// Get the TAG length of AEAD ciphers
    pub fn tag_len(&self) -> usize {
        cipher_method_forward!(ref self, ss_tag_len)
    }

    /// Encrypt a packet. Encrypted result will be written in `pkt`
    ///
    /// - Stream Ciphers: the size of input and output packets are the same
    /// - AEAD Ciphers: the size of output must be at least `input.len() + TAG_LEN`
    pub fn encrypt_packet(&mut self, pkt: &mut [u8]) {
        cipher_method_forward!(mut self, ss_encrypt_slice, pkt)
    }

    /// Decrypt a packet. Decrypted result will be written in `pkt`
    ///
    /// - Stream Ciphers: the size of input and output packets are the same
    /// - AEAD Ciphers: the size of output is `input.len() - TAG_LEN`
    #[must_use]
    pub fn decrypt_packet(&mut self, pkt: &mut [u8]) -> bool {
        cipher_method_forward!(mut self, ss_decrypt_slice, pkt)
    }
}

#[test]
fn test_cipher_new_none() {
    let key = [2u8; 16];
    let salt = [1u8; 16];
    let kind = CipherKind::NONE;

    let cipher = Cipher::new(kind, &key, &salt);
    assert_eq!(cipher.tag_len(), 0);
}

#[cfg(feature = "v1-aead")]
#[test]
fn test_cipher_new_aead() {
    let key = [2u8; 16];
    let salt = [1u8; 16];
    let kind = CipherKind::AES_128_GCM;

    let cipher = Cipher::new(kind, &key, &salt);
    assert_eq!(cipher.tag_len(), 16);
}

#[cfg(feature = "v1-stream")]
#[test]
fn test_cipher_new_stream() {
    let key = [2u8; 32];
    let iv = [1u8; 12];
    let kind = CipherKind::CHACHA20;

    let cipher = Cipher::new(kind, &key, &iv);
    assert_eq!(cipher.tag_len(), 0);
}

#[test]
fn test_send() {
    fn test<C: Send>() {}
    test::<Cipher>();
}

#[test]
fn test_sync() {
    fn test<C: Sync>() {}
    test::<Cipher>();
}
