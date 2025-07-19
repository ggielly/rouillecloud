use rand::RngCore;
use rand::rngs::OsRng;
use blake3::derive_key;
use zeroize::Zeroize;

pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key from a password using BLAKE3
    pub fn derive_key_from_password(
        password: &str,
        salt: &[u8],
        context: &str,
    ) -> [u8; 32] {
        let mut key_material = Vec::with_capacity(password.len() + salt.len());
        key_material.extend_from_slice(password.as_bytes());
        key_material.extend_from_slice(salt);
        
        derive_key(context, &key_material)
    }
    
    /// Generate a random salt
    pub fn generate_salt(size: usize) -> Vec<u8> {
        let mut salt = vec![0u8; size];
        let mut rng = OsRng;
        rng.fill_bytes(&mut salt);
        salt
    }
    
    /// Derive a subkey from a master key
    pub fn derive_subkey(master_key: &[u8], context: &str, info: &str) -> [u8; 32] {
        let full_context = format!("{}:{}", context, info);
        derive_key(&full_context, master_key)
    }
    
    /// Key stretching using multiple rounds of BLAKE3
    pub fn stretch_key(key: &[u8], rounds: u32) -> [u8; 32] {
        let mut result = [0u8; 32];
        result[..key.len().min(32)].copy_from_slice(&key[..key.len().min(32)]);
        
        for i in 0..rounds {
            let context = format!("stretch:{}", i);
            result = derive_key(&context, &result);
        }
        
        result
    }
}

/// Secure key storage with automatic zeroization
#[derive(Clone)]
pub struct SecureKey {
    key: [u8; 32],
    context: String,
}

impl SecureKey {
    pub fn new(key: [u8; 32], context: String) -> Self {
        Self { key, context }
    }
    
    pub fn from_password(password: &str, salt: &[u8], context: &str) -> Self {
        let key = KeyDerivation::derive_key_from_password(password, salt, context);
        Self {
            key,
            context: context.to_string(),
        }
    }
    
    pub fn generate(context: String) -> Self {
        let mut key = [0u8; 32];
        let mut rng = OsRng;
        rng.fill_bytes(&mut key);
        Self { key, context }
    }
    
    pub fn derive_subkey(&self, info: &str) -> SecureKey {
        let subkey = KeyDerivation::derive_subkey(&self.key, &self.context, info);
        let sub_context = format!("{}:{}", self.context, info);
        SecureKey::new(subkey, sub_context)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
    
    pub fn context(&self) -> &str {
        &self.context
    }
    
    /// Export key with additional derivation for storage
    pub fn export_for_storage(&self, storage_context: &str) -> [u8; 32] {
        KeyDerivation::derive_subkey(&self.key, &self.context, storage_context)
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl Zeroize for SecureKey {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.context.zeroize();
    }
}

/// Key derivation for different purposes
pub enum KeyPurpose {
    FileEncryption,
    DatabaseEncryption,
    UserAuthentication,
    SessionToken,
    ApiKey,
    BackupEncryption,
}

impl KeyPurpose {
    pub fn context(&self) -> &'static str {
        match self {
            KeyPurpose::FileEncryption => "fileshare.file.encryption",
            KeyPurpose::DatabaseEncryption => "fileshare.database.encryption",
            KeyPurpose::UserAuthentication => "fileshare.user.auth",
            KeyPurpose::SessionToken => "fileshare.session.token",
            KeyPurpose::ApiKey => "fileshare.api.key",
            KeyPurpose::BackupEncryption => "fileshare.backup.encryption",
        }
    }
}

/// Key derivation factory for creating purpose-specific keys
pub struct KeyDerivationFactory {
    master_key: SecureKey,
}

impl KeyDerivationFactory {
    pub fn new(master_key: SecureKey) -> Self {
        Self { master_key }
    }
    
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let master_key = SecureKey::from_password(
            password, 
            salt, 
            "fileshare.master"
        );
        Self { master_key }
    }
    
    pub fn derive_key(&self, purpose: KeyPurpose, additional_info: Option<&str>) -> SecureKey {
        let info = match additional_info {
            Some(info) => format!("{}:{}", purpose.context(), info),
            None => purpose.context().to_string(),
        };
        
        self.master_key.derive_subkey(&info)
    }
    
    pub fn derive_user_key(&self, user_id: &str) -> SecureKey {
        self.derive_key(KeyPurpose::UserAuthentication, Some(user_id))
    }
    
    pub fn derive_file_key(&self, file_id: &str) -> SecureKey {
        self.derive_key(KeyPurpose::FileEncryption, Some(file_id))
    }
    
    pub fn derive_api_key(&self, api_key_id: &str) -> SecureKey {
        self.derive_key(KeyPurpose::ApiKey, Some(api_key_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_derivation_from_password() {
        let password = "test_password";
        let salt = KeyDerivation::generate_salt(16);
        let context = "test_context";
        
        let key1 = KeyDerivation::derive_key_from_password(password, &salt, context);
        let key2 = KeyDerivation::derive_key_from_password(password, &salt, context);
        
        assert_eq!(key1, key2);
        
        // Different salt should produce different key
        let salt2 = KeyDerivation::generate_salt(16);
        let key3 = KeyDerivation::derive_key_from_password(password, &salt2, context);
        assert_ne!(key1, key3);
    }
    
    #[test]
    fn test_secure_key() {
        let password = "test_password";
        let salt = KeyDerivation::generate_salt(16);
        let context = "test";
        
        let key = SecureKey::from_password(password, &salt, context);
        assert_eq!(key.context(), context);
        assert_eq!(key.as_bytes().len(), 32);
        
        let subkey = key.derive_subkey("sub");
        assert_ne!(key.as_bytes(), subkey.as_bytes());
        assert_eq!(subkey.context(), "test:sub");
    }
    
    #[test]
    fn test_key_factory() {
        let master_key = SecureKey::generate("master".to_string());
        let factory = KeyDerivationFactory::new(master_key);
        
        let file_key1 = factory.derive_file_key("file1");
        let file_key2 = factory.derive_file_key("file2");
        let user_key = factory.derive_user_key("user1");
        
        // Different purposes/IDs should produce different keys
        assert_ne!(file_key1.as_bytes(), file_key2.as_bytes());
        assert_ne!(file_key1.as_bytes(), user_key.as_bytes());
        
        // Same input should produce same key
        let file_key1_again = factory.derive_file_key("file1");
        assert_eq!(file_key1.as_bytes(), file_key1_again.as_bytes());
    }
    
    #[test]
    fn test_key_stretching() {
        let original_key = b"short_key";
        let stretched1 = KeyDerivation::stretch_key(original_key, 1000);
        let stretched2 = KeyDerivation::stretch_key(original_key, 1000);
        let stretched_different = KeyDerivation::stretch_key(original_key, 1001);
        
        assert_eq!(stretched1, stretched2);
        assert_ne!(stretched1, stretched_different);
    }
}
