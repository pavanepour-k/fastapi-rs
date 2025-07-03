use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;
use crate::security::utils::{generate_secure_random_string, constant_time_compare};

#[derive(Error, Debug)]
pub enum OAuth2Error {
    #[error("Invalid grant type: {0}")]
    InvalidGrantType(String),
    #[error("Invalid client credentials")]
    InvalidClient,
    #[error("Invalid authorization code")]
    InvalidAuthorizationCode,
    #[error("Invalid access token")]
    InvalidAccessToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid scope: {0}")]
    InvalidScope(String),
    #[error("Insufficient scope")]
    InsufficientScope,
    #[error("Invalid redirect URI")]
    InvalidRedirectUri,
    #[error("PKCE verification failed")]
    PkceVerificationFailed,
}

pub type Result<T> = std::result::Result<T, OAuth2Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Client {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub grant_types: Vec<String>,
    pub is_confidential: bool,
}

impl OAuth2Client {
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uris: Vec::new(),
            scopes: Vec::new(),
            grant_types: vec!["authorization_code".to_string()],
            is_confidential: client_secret.is_some(),
        }
    }
    
    pub fn with_redirect_uris(mut self, uris: Vec<String>) -> Self {
        self.redirect_uris = uris;
        self
    }
    
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }
    
    pub fn verify_secret(&self, provided_secret: &str) -> bool {
        match &self.client_secret {
            Some(secret) => constant_time_compare(secret, provided_secret),
            None => false,
        }
    }
    
    pub fn is_redirect_uri_valid(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|allowed_uri| allowed_uri == uri)
    }
    
    pub fn supports_grant_type(&self, grant_type: &str) -> bool {
        self.grant_types.contains(&grant_type.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub user_id: Option<String>,
}

impl AuthorizationCode {
    pub fn new(
        client_id: String,
        redirect_uri: String,
        scopes: Vec<String>,
        expires_in_seconds: i64,
    ) -> Self {
        Self {
            code: generate_secure_random_string(32, None),
            client_id,
            redirect_uri,
            scopes,
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            code_challenge: None,
            code_challenge_method: None,
            user_id: None,
        }
    }
    
    pub fn with_pkce(mut self, challenge: String, method: String) -> Self {
        self.code_challenge = Some(challenge);
        self.code_challenge_method = Some(method);
        self
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    pub fn verify_pkce(&self, verifier: &str) -> bool {
        match (&self.code_challenge, &self.code_challenge_method) {
            (Some(challenge), Some(method)) => {
                match method.as_str() {
                    "plain" => constant_time_compare(challenge, verifier),
                    "S256" => {
                        let computed_challenge = base64_url_encode(&sha256(verifier.as_bytes()));
                        constant_time_compare(challenge, &computed_challenge)
                    }
                    _ => false,
                }
            }
            _ => true, // No PKCE required
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
    pub client_id: String,
    pub user_id: Option<String>,
    pub refresh_token: Option<String>,
}

impl AccessToken {
    pub fn new(
        client_id: String,
        scopes: Vec<String>,
        expires_in_seconds: i64,
    ) -> Self {
        Self {
            token: generate_secure_random_string(64, None),
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            scopes,
            client_id,
            user_id: None,
            refresh_token: Some(generate_secure_random_string(64, None)),
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scopes.iter().any(|scope| scope == required_scope)
    }
    
    pub fn has_any_scope(&self, required_scopes: &[String]) -> bool {
        required_scopes.iter().any(|scope| self.has_scope(scope))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

impl From<AccessToken> for TokenResponse {
    fn from(token: AccessToken) -> Self {
        let expires_in = (token.expires_at - Utc::now()).num_seconds();
        Self {
            access_token: token.token,
            token_type: token.token_type,
            expires_in,
            refresh_token: token.refresh_token,
            scope: if token.scopes.is_empty() {
                None
            } else {
                Some(token.scopes.join(" "))
            },
        }
    }
}

pub struct OAuth2Server {
    clients: HashMap<String, OAuth2Client>,
    authorization_codes: HashMap<String, AuthorizationCode>,
    access_tokens: HashMap<String, AccessToken>,
}

impl OAuth2Server {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            authorization_codes: HashMap::new(),
            access_tokens: HashMap::new(),
        }
    }
    
    pub fn register_client(&mut self, client: OAuth2Client) {
        self.clients.insert(client.client_id.clone(), client);
    }
    
    pub fn create_authorization_code(
        &mut self,
        client_id: &str,
        redirect_uri: &str,
        scopes: Vec<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        user_id: Option<String>,
    ) -> Result<String> {
        let client = self.clients.get(client_id)
            .ok_or(OAuth2Error::InvalidClient)?;
        
        if !client.is_redirect_uri_valid(redirect_uri) {
            return Err(OAuth2Error::InvalidRedirectUri);
        }
        
        let mut auth_code = AuthorizationCode::new(
            client_id.to_string(),
            redirect_uri.to_string(),
            scopes,
            600, // 10 minutes
        );
        
        if let (Some(challenge), Some(method)) = (code_challenge, code_challenge_method) {
            auth_code = auth_code.with_pkce(challenge, method);
        }
        
        if let Some(uid) = user_id {
            auth_code = auth_code.with_user(uid);
        }
        
        let code = auth_code.code.clone();
        self.authorization_codes.insert(code.clone(), auth_code);
        
        Ok(code)
    }
    
    pub fn exchange_authorization_code(
        &mut self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<TokenResponse> {
        let auth_code = self.authorization_codes.remove(code)
            .ok_or(OAuth2Error::InvalidAuthorizationCode)?;
        
        if auth_code.is_expired() {
            return Err(OAuth2Error::InvalidAuthorizationCode);
        }
        
        if auth_code.client_id != client_id {
            return Err(OAuth2Error::InvalidClient);
        }
        
        if auth_code.redirect_uri != redirect_uri {
            return Err(OAuth2Error::InvalidRedirectUri);
        }
        
        let client = self.clients.get(client_id)
            .ok_or(OAuth2Error::InvalidClient)?;
        
        // Verify client credentials for confidential clients
        if client.is_confidential {
            match (client_secret, &client.client_secret) {
                (Some(provided), Some(expected)) => {
                    if !constant_time_compare(provided, expected) {
                        return Err(OAuth2Error::InvalidClient);
                    }
                }
                _ => return Err(OAuth2Error::InvalidClient),
            }
        }
        
        // Verify PKCE if required
        if let Some(verifier) = code_verifier {
            if !auth_code.verify_pkce(verifier) {
                return Err(OAuth2Error::PkceVerificationFailed);
            }
        } else if auth_code.code_challenge.is_some() {
            return Err(OAuth2Error::PkceVerificationFailed);
        }
        
        let mut access_token = AccessToken::new(
            client_id.to_string(),
            auth_code.scopes,
            3600, // 1 hour
        );
        
        if let Some(uid) = auth_code.user_id {
            access_token = access_token.with_user(uid);
        }
        
        let token_response = TokenResponse::from(access_token.clone());
        self.access_tokens.insert(access_token.token.clone(), access_token);
        
        Ok(token_response)
    }
    
    pub fn validate_access_token(&self, token: &str) -> Result<&AccessToken> {
        let access_token = self.access_tokens.get(token)
            .ok_or(OAuth2Error::InvalidAccessToken)?;
        
        if access_token.is_expired() {
            return Err(OAuth2Error::TokenExpired);
        }
        
        Ok(access_token)
    }
    
    pub fn refresh_access_token(
        &mut self,
        refresh_token: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<TokenResponse> {
        // Find the access token with the matching refresh token
        let (old_token, old_access_token) = self.access_tokens.iter()
            .find(|(_, token)| {
                token.refresh_token.as_ref() == Some(&refresh_token.to_string()) &&
                token.client_id == client_id
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .ok_or(OAuth2Error::InvalidAccessToken)?;
        
        let client = self.clients.get(client_id)
            .ok_or(OAuth2Error::InvalidClient)?;
        
        // Verify client credentials
        if client.is_confidential {
            match (client_secret, &client.client_secret) {
                (Some(provided), Some(expected)) => {
                    if !constant_time_compare(provided, expected) {
                        return Err(OAuth2Error::InvalidClient);
                    }
                }
                _ => return Err(OAuth2Error::InvalidClient),
            }
        }
        
        // Remove old token
        self.access_tokens.remove(&old_token);
        
        // Create new token
        let mut new_token = AccessToken::new(
            client_id.to_string(),
            old_access_token.scopes,
            3600, // 1 hour
        );
        
        if let Some(uid) = old_access_token.user_id {
            new_token = new_token.with_user(uid);
        }
        
        let token_response = TokenResponse::from(new_token.clone());
        self.access_tokens.insert(new_token.token.clone(), new_token);
        
        Ok(token_response)
    }
    
    pub fn revoke_token(&mut self, token: &str) -> bool {
        self.access_tokens.remove(token).is_some()
    }
    
    pub fn cleanup_expired_tokens(&mut self) {
        let now = Utc::now();
        
        self.authorization_codes.retain(|_, code| code.expires_at > now);
        self.access_tokens.retain(|_, token| token.expires_at > now);
    }
}

impl Default for OAuth2Server {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions
fn sha256(data: &[u8]) -> Vec<u8> {
    // Simplified SHA-256 implementation for demo
    // In production, use a proper crypto library
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    
    hash.to_be_bytes().to_vec()
}

fn base64_url_encode(data: &[u8]) -> String {
    base64::encode_config(data, base64::URL_SAFE_NO_PAD)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oauth2_client_creation() {
        let client = OAuth2Client::new("test_client".to_string(), Some("secret".to_string()))
            .with_redirect_uris(vec!["http://localhost:8000/callback".to_string()])
            .with_scopes(vec!["read".to_string(), "write".to_string()]);
        
        assert_eq!(client.client_id, "test_client");
        assert!(client.is_confidential);
        assert!(client.verify_secret("secret"));
        assert!(!client.verify_secret("wrong_secret"));
        assert!(client.is_redirect_uri_valid("http://localhost:8000/callback"));
        assert!(!client.is_redirect_uri_valid("http://evil.com/callback"));
    }
    
    #[test]
    fn test_authorization_code_flow() {
        let mut server = OAuth2Server::new();
        
        let client = OAuth2Client::new("test_client".to_string(), Some("secret".to_string()))
            .with_redirect_uris(vec!["http://localhost:8000/callback".to_string()]);
        server.register_client(client);
        
        // Create authorization code
        let code = server.create_authorization_code(
            "test_client",
            "http://localhost:8000/callback",
            vec!["read".to_string()],
            None,
            None,
            Some("user123".to_string()),
        ).unwrap();
        
        // Exchange code for token
        let token_response = server.exchange_authorization_code(
            &code,
            "test_client",
            Some("secret"),
            "http://localhost:8000/callback",
            None,
        ).unwrap();
        
        assert_eq!(token_response.token_type, "Bearer");
        assert!(token_response.access_token.len() > 0);
        assert!(token_response.refresh_token.is_some());
        
        // Validate the token
        let access_token = server.validate_access_token(&token_response.access_token).unwrap();
        assert_eq!(access_token.client_id, "test_client");
        assert_eq!(access_token.user_id, Some("user123".to_string()));
        assert!(access_token.has_scope("read"));
    }
    
    #[test]
    fn test_pkce_flow() {
        let mut server = OAuth2Server::new();
        
        let client = OAuth2Client::new("public_client".to_string(), None)
            .with_redirect_uris(vec!["http://localhost:8000/callback".to_string()]);
        server.register_client(client);
        
        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let code_challenge = base64_url_encode(&sha256(code_verifier.as_bytes()));
        
        // Create authorization code with PKCE
        let code = server.create_authorization_code(
            "public_client",
            "http://localhost:8000/callback",
            vec!["read".to_string()],
            Some(code_challenge),
            Some("S256".to_string()),
            None,
        ).unwrap();
        
        // Exchange code with correct verifier
        let token_response = server.exchange_authorization_code(
            &code,
            "public_client",
            None,
            "http://localhost:8000/callback",
            Some(code_verifier),
        ).unwrap();
        
        assert_eq!(token_response.token_type, "Bearer");
    }
    
    #[test]
    fn test_token_refresh() {
        let mut server = OAuth2Server::new();
        
        let client = OAuth2Client::new("test_client".to_string(), Some("secret".to_string()))
            .with_redirect_uris(vec!["http://localhost:8000/callback".to_string()]);
        server.register_client(client);
        
        // Get initial token
        let code = server.create_authorization_code(
            "test_client",
            "http://localhost:8000/callback",
            vec!["read".to_string()],
            None,
            None,
            None,
        ).unwrap();
        
        let initial_token = server.exchange_authorization_code(
            &code,
            "test_client",
            Some("secret"),
            "http://localhost:8000/callback",
            None,
        ).unwrap();
        
        // Refresh the token
        let refresh_token = initial_token.refresh_token.unwrap();
        let new_token = server.refresh_access_token(
            &refresh_token,
            "test_client",
            Some("secret"),
        ).unwrap();
        
        assert_ne!(initial_token.access_token, new_token.access_token);
        assert_eq!(new_token.token_type, "Bearer");
        
        // Old token should be invalid
        assert!(server.validate_access_token(&initial_token.access_token).is_err());
    }
}