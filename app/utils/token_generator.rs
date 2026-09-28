use crate::entity::admin_user;
use crate::error::{AppError, AppResult};
use anyhow::anyhow;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

const _ALGORITHM: Algorithm = Algorithm::HS512;
pub const TOKEN_VALIDITY: time::Duration = time::Duration::hours(4);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
    name: String,
    access: String,
}

pub struct LoginTokenGenerator {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    db: DatabaseConnection,
}

impl LoginTokenGenerator {
    #[must_use]
    pub fn from_secret(secret: &str, db: DatabaseConnection) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            db,
        }
    }

    pub fn user_to_token(&self, user: &admin_user::Model) -> AppResult<String> {
        let exp = std::time::SystemTime::now() + TOKEN_VALIDITY;

        let claims = Claims {
            sub: user.email.clone(),
            exp: exp
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| AppError::Internal(anyhow!("Time went backwards")))?
                .as_secs(),
            name: user.name.clone(),
            access: match user.role {
                admin_user::UserRole::Admin => "write",
                admin_user::UserRole::Observer => "read",
            }
            .into(),
        };
        encode(&Header::new(_ALGORITHM), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(e.into()))
    }

    pub async fn token_to_user(&self, token: &str) -> AppResult<admin_user::Model> {
        let token = decode::<Claims>(&token, &self.decoding_key, &Validation::new(_ALGORITHM))
            .map_err(|_| AppError::Unauthorized)?;

        admin_user::Entity::find_by_email(&token.claims.sub)
            .one(&self.db)
            .await?
            .ok_or(AppError::Unauthorized)
    }

    pub async fn refresh(&self, token: &str) -> AppResult<String> {
        self.user_to_token(&self.token_to_user(token).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::admin_user;
    use crate::utils::test_utils::tests::setup_test;
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn test_refresh() {
        let test_app = setup_test().await.with_admin("admin@lio.lu", "test").await;
        let user = admin_user::Entity::find_by_email("admin@lio.lu")
            .one(&test_app.state.db)
            .await
            .unwrap()
            .unwrap();

        let token = test_app.state.token_generator.user_to_token(&user);
        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(token.len() > 0);

        sleep(std::time::Duration::from_secs(1)).await;

        let refreshed_token = test_app.state.token_generator.refresh(&token).await;
        assert!(refreshed_token.is_ok());
        let refreshed_token = refreshed_token.unwrap();
        assert!(refreshed_token.len() > 0);
        assert_ne!(token, refreshed_token);
    }
}
