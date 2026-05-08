use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub is_admin: bool,
}

#[derive(Clone, Debug)]
struct UserCredentials {
    id: u64,
    email: String,
    password_hash: String,
    is_admin: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Session {
    pub token: String,
    pub user: User,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    InvalidEmail,
    InvalidPassword,
    EmailAlreadyExists,
    InvalidCredentials,
    PasswordHashFailure,
}

pub struct AuthService {
    next_id: AtomicU64,
    users_by_email: RwLock<HashMap<String, UserCredentials>>,
    sessions: RwLock<HashMap<String, u64>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            users_by_email: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, input: RegisterInput) -> Result<Session, AuthError> {
        let normalized_email = normalize_email(&input.email).ok_or(AuthError::InvalidEmail)?;
        validate_password(&input.password)?;

        let password_hash = hash_password(&input.password)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let user_creds = UserCredentials {
            id,
            email: normalized_email.clone(),
            password_hash,
            is_admin: false,
        };

        let mut users = self
            .users_by_email
            .write()
            .expect("users write lock poisoned");

        if users.contains_key(&normalized_email) {
            return Err(AuthError::EmailAlreadyExists);
        }

        users.insert(normalized_email.clone(), user_creds.clone());
        drop(users);

        self.issue_session_for_user(&user_creds)
    }

    pub fn login(&self, input: LoginInput) -> Result<Session, AuthError> {
        let normalized_email =
            normalize_email(&input.email).ok_or(AuthError::InvalidCredentials)?;

        let users = self
            .users_by_email
            .read()
            .expect("users read lock poisoned");
        let user_creds = users
            .get(&normalized_email)
            .ok_or(AuthError::InvalidCredentials)?;

        verify_password(&input.password, &user_creds.password_hash)?;

        self.issue_session_for_user(user_creds)
    }

    pub fn get_user_by_token(&self, token: &str) -> Option<User> {
        let user_id = {
            let sessions = self.sessions.read().expect("sessions read lock poisoned");
            sessions.get(token).copied()?
        };

        let users = self
            .users_by_email
            .read()
            .expect("users read lock poisoned");

        users
            .values()
            .find(|user| user.id == user_id)
            .map(|user| User {
                id: user.id,
                email: user.email.clone(),
                is_admin: user.is_admin,
            })
    }

    fn issue_session_for_user(&self, user: &UserCredentials) -> Result<Session, AuthError> {
        let token = Uuid::new_v4().to_string();
        let mut sessions = self.sessions.write().expect("sessions write lock poisoned");
        sessions.insert(token.clone(), user.id);

        Ok(Session {
            token,
            user: User {
                id: user.id,
                email: user.email.clone(),
                is_admin: user.is_admin,
            },
        })
    }
}

fn normalize_email(email: &str) -> Option<String> {
    let value = email.trim().to_ascii_lowercase();
    if value.is_empty() || !value.contains('@') || value.len() > 320 {
        return None;
    }

    let mut split = value.split('@');
    let local = split.next().unwrap_or_default();
    let domain = split.next().unwrap_or_default();

    if local.is_empty() || domain.is_empty() || split.next().is_some() || !domain.contains('.') {
        return None;
    }

    Some(value)
}

fn validate_password(password: &str) -> Result<(), AuthError> {
    let value = password.trim();
    if value.len() < 8 || value.len() > 128 {
        return Err(AuthError::InvalidPassword);
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::PasswordHashFailure)?;

    Ok(hash.to_string())
}

fn verify_password(password: &str, hashed: &str) -> Result<(), AuthError> {
    let parsed_hash = PasswordHash::new(hashed).map_err(|_| AuthError::InvalidCredentials)?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidCredentials)
}

#[cfg(test)]
mod tests {
    use super::{AuthError, AuthService, LoginInput, RegisterInput};

    #[test]
    fn register_and_login_work() {
        let auth = AuthService::new();

        let register = auth.register(RegisterInput {
            email: "User@Example.com".to_string(),
            password: "password123".to_string(),
        });
        assert!(register.is_ok());

        let login = auth.login(LoginInput {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        });
        assert!(login.is_ok());
    }

    #[test]
    fn duplicate_email_rejected() {
        let auth = AuthService::new();

        let _ = auth.register(RegisterInput {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        });

        let second = auth.register(RegisterInput {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        });

        assert!(matches!(second, Err(AuthError::EmailAlreadyExists)));
    }

    #[test]
    fn wrong_password_rejected() {
        let auth = AuthService::new();

        let _ = auth.register(RegisterInput {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        });

        let login = auth.login(LoginInput {
            email: "user@example.com".to_string(),
            password: "not-the-password".to_string(),
        });

        assert!(matches!(login, Err(AuthError::InvalidCredentials)));
    }
}
