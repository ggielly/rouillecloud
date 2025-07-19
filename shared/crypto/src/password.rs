use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use zeroize::Zeroize;

pub struct PasswordManager {
    argon2: Argon2<'static>,
}

impl PasswordManager {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        let salt = SaltString::generate(OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| PasswordError::HashingFailed)?;
        
        Ok(password_hash.to_string())
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| PasswordError::InvalidHash)?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    pub fn generate_secure_password(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let mut rng = OsRng;
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    pub fn check_password_strength(password: &str) -> PasswordStrength {
        let mut score = 0;
        let mut feedback = Vec::new();
        
        // Length check
        if password.len() >= 12 {
            score += 2;
        } else if password.len() >= 8 {
            score += 1;
        } else {
            feedback.push("Password should be at least 8 characters long".to_string());
        }
        
        // Character variety checks
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        } else {
            feedback.push("Include uppercase letters".to_string());
        }
        
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        } else {
            feedback.push("Include lowercase letters".to_string());
        }
        
        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        } else {
            feedback.push("Include numbers".to_string());
        }
        
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        } else {
            feedback.push("Include special characters".to_string());
        }
        
        // Common patterns check
        if Self::contains_common_patterns(password) {
            score -= 1;
            feedback.push("Avoid common patterns".to_string());
        }
        
        let strength = match score {
            0..=2 => PasswordStrengthLevel::Weak,
            3..=4 => PasswordStrengthLevel::Medium,
            5..=6 => PasswordStrengthLevel::Strong,
            _ => PasswordStrengthLevel::VeryStrong,
        };
        
        PasswordStrength {
            level: strength,
            score,
            feedback,
        }
    }
    
    fn contains_common_patterns(password: &str) -> bool {
        let common_patterns = [
            "123456", "password", "qwerty", "abc123", "admin",
            "letmein", "welcome", "monkey", "dragon", "password123",
        ];
        
        let lower_password = password.to_lowercase();
        common_patterns.iter().any(|&pattern| lower_password.contains(pattern))
    }
}

impl Default for PasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }
    
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { data: bytes }
    }
    
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl Zeroize for SecureString {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

#[derive(Debug, Clone)]
pub struct PasswordStrength {
    pub level: PasswordStrengthLevel,
    pub score: i32,
    pub feedback: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrengthLevel {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password hashing failed")]
    HashingFailed,
    #[error("Invalid password hash")]
    InvalidHash,
    #[error("Password verification failed")]
    VerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_hashing() {
        let manager = PasswordManager::new();
        let password = "test_password_123";
        
        let hash = manager.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        
        assert!(manager.verify_password(password, &hash).unwrap());
        assert!(!manager.verify_password("wrong_password", &hash).unwrap());
    }
    
    #[test]
    fn test_password_generation() {
        let password = PasswordManager::generate_secure_password(16);
        assert_eq!(password.len(), 16);
        
        // Should contain varied characters
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        
        assert!(has_upper || has_lower || has_digit);
    }
    
    #[test]
    fn test_password_strength() {
        let weak = PasswordManager::check_password_strength("123");
        assert_eq!(weak.level, PasswordStrengthLevel::Weak);
        
        let strong = PasswordManager::check_password_strength("MyStr0ng!P@ssw0rd");
        assert!(matches!(strong.level, PasswordStrengthLevel::Strong | PasswordStrengthLevel::VeryStrong));
    }
    
    #[test]
    fn test_secure_string() {
        let mut secure = SecureString::new("secret".to_string());
        assert_eq!(secure.as_str().unwrap(), "secret");
        assert_eq!(secure.len(), 6);
        
        secure.zeroize();
        // Data should be zeroed but we can't easily test this without unsafe code
    }
}
